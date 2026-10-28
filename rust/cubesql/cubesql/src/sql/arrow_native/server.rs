use std::{collections::HashMap, sync::Arc, time::SystemTime};

use async_trait::async_trait;
use datafusion::{dataframe::DataFrame as DFDataFrame, physical_plan::SendableRecordBatchStream};
use futures::{FutureExt, StreamExt};
use log::{error, trace};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    sync::{watch, RwLock},
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{
    ipc,
    protocol::{
        is_supported_version, read_message, write_message, Message, MAX_AUTHENTICATED_FRAME_LEN,
        MAX_UNAUTHENTICATED_FRAME_LEN, PROTOCOL_VERSION_1,
    },
};
use crate::{
    compile::{
        convert_statement_to_cube_query, parser::parse_sql_to_statement, DatabaseProtocol,
        QueryPlan,
    },
    config::processing_loop::{ProcessingLoop, ShutdownMode},
    sql::{
        redact_error_message, redact_sql_literals, redacted_query_key, Session, SessionManager,
        SqlAuthServiceAuthenticateRequest,
    },
    telemetry::{ContextLogger, SessionLogger},
    transport::SpanId,
    CubeError,
};

/// Serves the SQL API over the Arrow Native protocol (see `protocol`): the
/// same sessions, authentication and query planning as the Postgres endpoint,
/// with results sent as Arrow IPC instead of Postgres rows.
pub struct ArrowNativeServer {
    address: String,
    close_socket_rx: RwLock<watch::Receiver<Option<ShutdownMode>>>,
    close_socket_tx: watch::Sender<Option<ShutdownMode>>,
    session_manager: Arc<SessionManager>,
}

crate::di_service!(ArrowNativeServer, []);

#[async_trait]
impl ProcessingLoop for ArrowNativeServer {
    async fn processing_loop(&self) -> Result<(), CubeError> {
        let listener = TcpListener::bind(self.address.clone()).await?;

        println!("🔗 Cube SQL (arrow) is listening on {}", self.address);

        let fast_shutdown_interruptor = CancellationToken::new();
        let semifast_shutdown_interruptor = CancellationToken::new();

        let mut joinset = tokio::task::JoinSet::new();
        let mut active_shutdown_mode: Option<ShutdownMode> = None;

        loop {
            let mut stop_receiver = self.close_socket_rx.write().await;
            let (socket, _) = tokio::select! {
                _ = stop_receiver.changed() => {
                    let mode = *stop_receiver.borrow();
                    if mode > active_shutdown_mode {
                        active_shutdown_mode = mode;
                        match active_shutdown_mode {
                            Some(ShutdownMode::Fast) => {
                                trace!("[arrow] Stopping processing_loop via channel, fast mode");

                                fast_shutdown_interruptor.cancel();
                                break;
                            }
                            Some(ShutdownMode::SemiFast) => {
                                trace!("[arrow] Stopping processing_loop via channel, semifast mode");

                                semifast_shutdown_interruptor.cancel();
                                break;
                            }
                            Some(ShutdownMode::Smart) => {
                                trace!("[arrow] Stopping processing_loop via interruptor, smart mode");
                                break;
                            }
                            None => {
                                unreachable!("mode compared greater than something; it can't be None");
                            }
                        }
                    } else {
                        continue;
                    }
                }
                Some(_) = joinset.join_next() => {
                    continue;
                }
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok(res) => res,
                        Err(err) => {
                            error!("Network error: {}", err);
                            continue;
                        }
                    }
                }
            };

            // A response is several small frames written back to back; with
            // Nagle's algorithm on, the client's delayed ACK stalls them ~40ms
            if let Err(err) = socket.set_nodelay(true) {
                error!("[arrow] Unable to set TCP_NODELAY: {}", err);
            }

            let (client_addr, client_port) = match socket.peer_addr() {
                Ok(peer_addr) => (peer_addr.ip().to_string(), peer_addr.port()),
                Err(e) => {
                    error!(
                        "[arrow] Error while calling peer_addr() on TcpStream: {}",
                        e
                    );

                    ("127.0.0.1".to_string(), 0000_u16)
                }
            };

            let connection = ArrowNativeConnection::new(
                socket,
                self.session_manager.clone(),
                client_addr,
                client_port,
            );
            let fast_shutdown_interruptor = fast_shutdown_interruptor.clone();
            let semifast_shutdown_interruptor = semifast_shutdown_interruptor.clone();

            joinset.spawn(async move {
                if let Err(e) = connection
                    .run(fast_shutdown_interruptor, semifast_shutdown_interruptor)
                    .await
                {
                    trace!("[arrow] Connection closed with error: {}", e);
                }
            });
        }

        // Close the listening socket (so we _visibly_ stop accepting incoming connections) before
        // we wait for the outstanding connection tasks finish.
        drop(listener);

        // Now that we've had the stop signal, wait for outstanding connection tasks to finish
        // cleanly.
        loop {
            let mut stop_receiver = self.close_socket_rx.write().await;
            tokio::select! {
                _ = stop_receiver.changed() => {
                    let mode = *stop_receiver.borrow();
                    if mode > active_shutdown_mode {
                        active_shutdown_mode = mode;
                        match active_shutdown_mode {
                            Some(ShutdownMode::Fast) => {
                                fast_shutdown_interruptor.cancel();
                            }
                            Some(ShutdownMode::SemiFast) => {
                                semifast_shutdown_interruptor.cancel();
                            }
                            _ => {}
                        }
                    }
                }
                res = joinset.join_next() => {
                    if res.is_none() {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn stop_processing(&self, mode: ShutdownMode) -> Result<(), CubeError> {
        self.close_socket_tx.send(Some(mode))?;
        Ok(())
    }
}

impl ArrowNativeServer {
    pub fn new(address: String, session_manager: Arc<SessionManager>) -> Arc<Self> {
        let (close_socket_tx, close_socket_rx) = watch::channel(None::<ShutdownMode>);
        Arc::new(Self {
            address,
            session_manager,
            close_socket_rx: RwLock::new(close_socket_rx),
            close_socket_tx,
        })
    }
}

/// An error sent to the client as an Error message.
struct ClientError {
    code: &'static str,
    message: String,
}

impl ClientError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub(crate) struct ArrowNativeConnection<S> {
    socket: S,
    session_manager: Arc<SessionManager>,
    client_addr: String,
    client_port: u16,
    version: u32,
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> ArrowNativeConnection<S> {
    pub(crate) fn new(
        socket: S,
        session_manager: Arc<SessionManager>,
        client_addr: String,
        client_port: u16,
    ) -> Self {
        Self {
            socket,
            session_manager,
            client_addr,
            client_port,
            version: PROTOCOL_VERSION_1,
        }
    }

    /// Serves the connection until the client disconnects or the server shuts
    /// down. The session exists from the handshake to the end of this call.
    pub(crate) async fn run(
        mut self,
        fast_shutdown_interruptor: CancellationToken,
        semifast_shutdown_interruptor: CancellationToken,
    ) -> Result<(), CubeError> {
        let Some(message) = self.read(MAX_UNAUTHENTICATED_FRAME_LEN).await? else {
            return Ok(());
        };
        match message {
            Message::HandshakeRequest { version } if is_supported_version(version) => {
                self.version = version;
                self.write(&Message::HandshakeResponse {
                    version,
                    server_version: env!("CARGO_PKG_VERSION").to_string(),
                })
                .await?;
            }
            Message::HandshakeRequest { version } => {
                return self
                    .write_error(ClientError::new(
                        "UNSUPPORTED_VERSION",
                        format!("Unsupported protocol version {}", version),
                    ))
                    .await;
            }
            other => return self.unexpected(other).await,
        }

        // Postgres sessions: queries are parsed, planned and authorized exactly
        // as they are for the SQL API's Postgres endpoint
        let session = match self
            .session_manager
            .create_session(
                DatabaseProtocol::PostgreSQL,
                self.client_addr.clone(),
                self.client_port,
                None,
            )
            .await
        {
            Ok(session) => session,
            Err(err) => {
                error!("Session creation error: {}", err);
                return self
                    .write_error(ClientError::new("TOO_MANY_CONNECTIONS", err.message))
                    .await;
            }
        };

        trace!("[arrow] New connection {}", session.state.connection_id);
        let connection_id = session.state.connection_id;

        // Like the Postgres endpoint, a panic while serving the connection
        // must not leak its session
        let result = std::panic::AssertUnwindSafe(self.serve(
            session,
            fast_shutdown_interruptor,
            semifast_shutdown_interruptor,
        ))
        .catch_unwind()
        .await
        .unwrap_or_else(|err| Err(CubeError::panic(err)));

        trace!("[arrow] Removing connection {}", connection_id);
        self.session_manager.drop_session(connection_id).await;

        result
    }

    async fn serve(
        &mut self,
        session: Arc<Session>,
        fast_shutdown_interruptor: CancellationToken,
        semifast_shutdown_interruptor: CancellationToken,
    ) -> Result<(), CubeError> {
        if !self.authenticate(&session).await? {
            return Ok(());
        }

        let logger: Arc<dyn ContextLogger> = Arc::new(SessionLogger::new(session.state.clone()));

        loop {
            let message = tokio::select! {
                _ = fast_shutdown_interruptor.cancelled() => return Ok(()),
                _ = semifast_shutdown_interruptor.cancelled() => return Ok(()),
                message = self.read(MAX_AUTHENTICATED_FRAME_LEN) => match message? {
                    Some(message) => message,
                    None => return Ok(()),
                },
            };

            match message {
                Message::QueryRequest { sql } => {
                    tokio::select! {
                        // Semifast shutdown lets the running query finish
                        _ = fast_shutdown_interruptor.cancelled() => return Ok(()),
                        res = self.process_query(&session, &logger, sql) => res?,
                    }
                }
                other => return self.unexpected(other).await,
            }
        }
    }

    /// Authenticates like the Postgres endpoint: the SQL auth service returns
    /// the expected password, unless it skips the check. Returns whether the
    /// client may continue.
    async fn authenticate(&mut self, session: &Arc<Session>) -> Result<bool, CubeError> {
        let Some(message) = self.read(MAX_UNAUTHENTICATED_FRAME_LEN).await? else {
            return Ok(false);
        };
        let (user, password, database) = match message {
            Message::AuthRequest {
                user,
                password,
                database,
            } => (user, password, database),
            other => {
                self.unexpected(other).await?;
                return Ok(false);
            }
        };

        let request = SqlAuthServiceAuthenticateRequest {
            protocol: "arrow_native".to_string(),
            method: "password".to_string(),
        };
        let response = session
            .server
            .auth
            .authenticate(request, user.clone(), Some(password.clone()))
            .await;

        let auth_context = match response {
            Ok(response)
                if response.skip_password_check
                    || response.password.as_deref() == Some(password.as_str()) =>
            {
                response.context
            }
            _ => {
                let reason = match &user {
                    Some(user) => format!("password authentication failed for user \"{}\"", user),
                    None => "password authentication failed".to_string(),
                };
                // A failed AuthResponse carries the reason in its session id field
                self.write(&Message::AuthResponse {
                    success: false,
                    session_id: reason,
                })
                .await?;
                return Ok(false);
            }
        };

        let user = user.or_else(|| auth_context.user().cloned());
        session
            .state
            .set_database(Some(database.unwrap_or("db".to_string())));
        session.state.set_user(user.clone());
        session.state.set_original_user(user);
        session.state.set_auth_context(Some(auth_context));

        self.write(&Message::AuthResponse {
            success: true,
            session_id: session.state.connection_id.to_string(),
        })
        .await?;

        Ok(true)
    }

    /// Runs one query and answers it. A failed query is answered with an
    /// Error message and leaves the connection usable; only an I/O error is
    /// returned.
    async fn process_query(
        &mut self,
        session: &Arc<Session>,
        logger: &Arc<dyn ContextLogger>,
        sql: String,
    ) -> Result<(), CubeError> {
        let log_redaction = session.server.config_obj.log_redaction();
        let span_id = Arc::new(
            SpanId::new(
                Uuid::new_v4().to_string(),
                serde_json::json!({ "sql": sql }),
            )
            .with_redacted_query_key(redacted_query_key(&sql, log_redaction)),
        );
        let start_time = SystemTime::now();

        self.log_load_state(
            session,
            &span_id,
            "Load Request",
            serde_json::json!({ "query": span_id.query_key.clone() }),
        )
        .await?;

        let stream = match Self::plan_and_execute(session, &sql, span_id.clone()).await {
            Ok(stream) => stream,
            Err(err) => {
                return self
                    .query_failed(session, logger, &span_id, &sql, err)
                    .await
            }
        };

        if let Err(err) = self.write_results(stream).await {
            return self
                .query_failed(session, logger, &span_id, &sql, err)
                .await;
        }

        self.log_load_state(
            session,
            &span_id,
            "Load Request Success",
            serde_json::json!({
                "query": span_id.query_key.clone(),
                "apiType": "sql",
                "duration": start_time.elapsed().unwrap().as_millis() as u64,
                "isDataQuery": span_id.is_data_query().await,
                "lastRefreshTime": span_id.last_refresh_time().await,
            }),
        )
        .await
    }

    async fn plan_and_execute(
        session: &Arc<Session>,
        sql: &str,
        span_id: Arc<SpanId>,
    ) -> Result<Option<SendableRecordBatchStream>, ClientError> {
        let query_error =
            |e: &dyn std::fmt::Display| ClientError::new("QUERY_ERROR", e.to_string());

        let auth_context = session
            .state
            .auth_context()
            .ok_or_else(|| ClientError::new("INTERNAL_ERROR", "must be auth"))?;
        let compiler_cache = &session.session_manager.server.compiler_cache;
        let cache_entry = compiler_cache
            .get_cache_entry(auth_context, session.state.protocol.clone())
            .await
            .map_err(|e| query_error(&e))?;
        let meta = compiler_cache
            .meta(cache_entry)
            .await
            .map_err(|e| query_error(&e))?;

        let statement = parse_sql_to_statement(sql, DatabaseProtocol::PostgreSQL, &mut None)
            .map_err(|e| query_error(&e))?;

        let plan = std::panic::AssertUnwindSafe(convert_statement_to_cube_query(
            statement,
            meta,
            session.clone(),
            &mut None,
            Some(span_id),
        ))
        .catch_unwind()
        .await
        .map_err(|e| query_error(&CubeError::panic(e)))?
        .map_err(|e| query_error(&e))?;

        match plan {
            QueryPlan::DataFusionSelect(plan, ctx) => {
                let df = DFDataFrame::new(ctx.state.clone(), &plan);
                let stream = std::panic::AssertUnwindSafe(df.execute_stream())
                    .catch_unwind()
                    .await
                    .map_err(|e| query_error(&CubeError::panic(e)))?
                    .map_err(|e| query_error(&e))?;
                Ok(Some(stream))
            }
            // SET, BEGIN and other statements that only change session state
            QueryPlan::MetaOk(_, _) => Ok(None),
            QueryPlan::MetaTabular(_, frame) if frame.get_columns().is_empty() => Ok(None),
            QueryPlan::MetaTabular(_, _)
            | QueryPlan::CreateTempTable(..)
            | QueryPlan::CreateEmptyTempTable(_)
            | QueryPlan::CopyFrom(_) => Err(ClientError::new(
                "UNSUPPORTED",
                "This statement is not supported by the Arrow Native endpoint; use the Postgres endpoint",
            )),
        }
    }

    /// Streams the results as a schema, one batch per record batch, and a
    /// completion. Each batch is encoded as it arrives.
    async fn write_results(
        &mut self,
        stream: Option<SendableRecordBatchStream>,
    ) -> Result<(), ClientError> {
        let io_error = |e: CubeError| ClientError::new("IO_ERROR", e.message);

        let Some(mut stream) = stream else {
            return self
                .write(&Message::QueryComplete { rows_affected: 0 })
                .await
                .map_err(io_error);
        };

        let schema = ipc::encode_schema(&stream.schema())
            .map_err(|e| ClientError::new("INTERNAL_ERROR", e.message))?;
        self.write(&Message::QueryResponseSchema {
            arrow_ipc_schema: schema,
        })
        .await
        .map_err(io_error)?;

        let mut rows = 0i64;
        while let Some(batch) = stream.next().await {
            let batch = batch.map_err(|e| ClientError::new("QUERY_ERROR", e.to_string()))?;
            rows += batch.num_rows() as i64;
            let arrow_ipc_batch = ipc::encode_batch(&batch)
                .map_err(|e| ClientError::new("INTERNAL_ERROR", e.message))?;
            self.write(&Message::QueryResponseBatch { arrow_ipc_batch })
                .await
                .map_err(io_error)?;
        }

        self.write(&Message::QueryComplete {
            rows_affected: rows,
        })
        .await
        .map_err(io_error)
    }

    /// Reports a failed query to the client, the log and the transport, as
    /// the Postgres endpoint does. A broken socket is returned as an error.
    async fn query_failed(
        &mut self,
        session: &Arc<Session>,
        logger: &Arc<dyn ContextLogger>,
        span_id: &Arc<SpanId>,
        sql: &str,
        err: ClientError,
    ) -> Result<(), CubeError> {
        if err.code == "IO_ERROR" {
            return Err(CubeError::internal(err.message));
        }

        let mut properties = serde_json::json!({
            "query": span_id.query_key.clone(),
            "error": err.message.clone(),
            "duration": span_id.duration(),
        });
        let mut log_props = HashMap::from([("query".to_string(), sql.to_string())]);
        if session.server.config_obj.log_redaction() {
            let redacted_error = redact_error_message(&err.message, sql);
            properties["redactedError"] = serde_json::json!(redacted_error);
            log_props.insert("redactedQuery".to_string(), redact_sql_literals(sql));
            log_props.insert("redactedError".to_string(), redacted_error);
        }

        self.log_load_state(session, span_id, "SQL API Error", properties)
            .await?;
        logger.error(err.message.as_str(), Some(log_props));

        self.write_error(err).await
    }

    async fn log_load_state(
        &self,
        session: &Arc<Session>,
        span_id: &Arc<SpanId>,
        event: &str,
        properties: serde_json::Value,
    ) -> Result<(), CubeError> {
        let Some(auth_context) = session.state.auth_context() else {
            return Ok(());
        };

        session
            .session_manager
            .server
            .transport
            .log_load_state(
                Some(span_id.clone()),
                auth_context,
                session.state.get_load_request_meta("sql"),
                event.to_string(),
                properties,
            )
            .await
    }

    async fn unexpected(&mut self, message: Message) -> Result<(), CubeError> {
        self.write_error(ClientError::new(
            "PROTOCOL_ERROR",
            format!("Unexpected message: {:?}", MessageName(&message)),
        ))
        .await
    }

    async fn write_error(&mut self, err: ClientError) -> Result<(), CubeError> {
        self.write(&Message::Error {
            code: err.code.to_string(),
            message: err.message,
        })
        .await
    }

    async fn read(&mut self, max_len: u32) -> Result<Option<Message>, CubeError> {
        read_message(&mut self.socket, self.version, max_len).await
    }

    async fn write(&mut self, message: &Message) -> Result<(), CubeError> {
        write_message(&mut self.socket, message, self.version).await
    }
}

/// Names a message without printing its contents, which may be a password.
struct MessageName<'a>(&'a Message);

impl std::fmt::Debug for MessageName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.0 {
            Message::HandshakeRequest { .. } => "HandshakeRequest",
            Message::HandshakeResponse { .. } => "HandshakeResponse",
            Message::AuthRequest { .. } => "AuthRequest",
            Message::AuthResponse { .. } => "AuthResponse",
            Message::QueryRequest { .. } => "QueryRequest",
            Message::QueryResponseSchema { .. } => "QueryResponseSchema",
            Message::QueryResponseBatch { .. } => "QueryResponseBatch",
            Message::QueryComplete { .. } => "QueryComplete",
            Message::Error { .. } => "Error",
        };
        f.write_str(name)
    }
}
