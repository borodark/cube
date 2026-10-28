use std::sync::Arc;

use async_trait::async_trait;
use datafusion::arrow::{array::Int64Array, ipc::reader::StreamReader, record_batch::RecordBatch};
use tokio::io::{duplex, DuplexStream};
use tokio_util::sync::CancellationToken;

use super::{
    protocol::{
        read_message, write_message, Message, MAX_AUTHENTICATED_FRAME_LEN, PROTOCOL_VERSION_1,
        PROTOCOL_VERSION_2,
    },
    server::ArrowNativeConnection,
};
use crate::{
    compile::test::{get_test_tenant_ctx, TestConnectionTransport},
    config::{ConfigObj, ConfigObjImpl},
    sql::{
        compiler_cache::CompilerCacheImpl, pg_auth_service::PostgresAuthServiceDefaultImpl,
        session::SessionStatActivity, AuthenticateResponse, HttpAuthContext, ServerManager,
        SessionManager, SqlAuthService, SqlAuthServiceAuthenticateRequest,
    },
    CubeError,
};

/// Accepts user "alice" with password "secret", and a request without a user
/// (protocol v1) with password "token". The access token of the resulting
/// context names whom it was issued to.
#[derive(Debug)]
struct TestAuth;

#[async_trait]
impl SqlAuthService for TestAuth {
    async fn authenticate(
        &self,
        request: SqlAuthServiceAuthenticateRequest,
        user: Option<String>,
        _password: Option<String>,
    ) -> Result<AuthenticateResponse, CubeError> {
        assert_eq!(request.protocol, "arrow_native");

        let (password, issued_to) = match user.as_deref() {
            Some("alice") => ("secret", "alice"),
            None => ("token", "anonymous"),
            Some(other) => return Err(CubeError::user(format!("Unknown user {}", other))),
        };

        Ok(AuthenticateResponse {
            context: Arc::new(HttpAuthContext {
                access_token: issued_to.to_string(),
                base_path: "test".to_string(),
            }),
            password: Some(password.to_string()),
            skip_password_check: false,
        })
    }
}

struct TestServer {
    session_manager: Arc<SessionManager>,
    transport: Arc<TestConnectionTransport>,
}

impl TestServer {
    fn new() -> Self {
        Self::with_config(ConfigObjImpl::default())
    }

    fn with_config(config_obj: ConfigObjImpl) -> Self {
        let config_obj: Arc<dyn ConfigObj> = Arc::new(config_obj);
        let transport = Arc::new(TestConnectionTransport::new(get_test_tenant_ctx()));
        let server = Arc::new(ServerManager::new(
            Arc::new(TestAuth),
            transport.clone(),
            Arc::new(PostgresAuthServiceDefaultImpl::new()),
            Arc::new(CompilerCacheImpl::new(
                config_obj.clone(),
                transport.clone(),
            )),
            None,
            config_obj,
        ));

        Self {
            session_manager: Arc::new(SessionManager::new(server)),
            transport,
        }
    }

    /// Starts serving one connection and returns the client's end of it.
    fn connect(&self) -> Client {
        let (client, server) = duplex(1024 * 1024);
        let connection = ArrowNativeConnection::new(
            server,
            self.session_manager.clone(),
            "127.0.0.1".to_string(),
            1234,
        );
        let handle = tokio::spawn(async move {
            connection
                .run(CancellationToken::new(), CancellationToken::new())
                .await
        });

        Client {
            socket: client,
            version: PROTOCOL_VERSION_1,
            handle,
        }
    }

    async fn session_count(&self) -> usize {
        self.session_manager
            .map_sessions::<SessionStatActivity>()
            .await
            .len()
    }
}

struct Client {
    socket: DuplexStream,
    version: u32,
    handle: tokio::task::JoinHandle<Result<(), CubeError>>,
}

impl Client {
    async fn send(&mut self, message: Message) {
        write_message(&mut self.socket, &message, self.version)
            .await
            .unwrap();
    }

    async fn recv(&mut self) -> Message {
        read_message(&mut self.socket, self.version, MAX_AUTHENTICATED_FRAME_LEN)
            .await
            .unwrap()
            .expect("the server closed the connection")
    }

    async fn handshake(&mut self, version: u32) {
        self.send(Message::HandshakeRequest { version }).await;
        match self.recv().await {
            Message::HandshakeResponse { version: v, .. } => assert_eq!(v, version),
            other => panic!("expected a handshake response, got {:?}", other),
        }
        self.version = version;
    }

    async fn login(&mut self, version: u32, user: Option<&str>, password: &str) -> Message {
        self.handshake(version).await;
        self.send(Message::AuthRequest {
            user: user.map(str::to_string),
            password: password.to_string(),
            database: None,
        })
        .await;
        self.recv().await
    }

    /// Runs `sql`, returning the decoded batches or the error message.
    async fn query(&mut self, sql: &str) -> Result<Vec<RecordBatch>, (String, String)> {
        self.send(Message::QueryRequest {
            sql: sql.to_string(),
        })
        .await;

        let mut batches = vec![];
        loop {
            match self.recv().await {
                Message::QueryResponseSchema { arrow_ipc_schema } => {
                    let reader = StreamReader::try_new(&arrow_ipc_schema[..], None).unwrap();
                    assert_eq!(reader.into_iter().count(), 0);
                }
                Message::QueryResponseBatch { arrow_ipc_batch } => {
                    // Every batch is a self-contained IPC stream
                    let reader = StreamReader::try_new(&arrow_ipc_batch[..], None).unwrap();
                    batches.extend(reader.map(|b| b.unwrap()));
                }
                Message::QueryComplete { rows_affected } => {
                    let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
                    assert_eq!(rows_affected as usize, rows);
                    return Ok(batches);
                }
                Message::Error { code, message } => return Err((code, message)),
                other => panic!("unexpected message {:?}", other),
            }
        }
    }

    /// Closes the client's end and waits for the server's handler to finish.
    async fn close(self) -> Result<(), CubeError> {
        drop(self.socket);
        self.handle.await.unwrap()
    }
}

fn int_column(batches: &[RecordBatch], column: usize) -> Vec<i64> {
    batches
        .iter()
        .flat_map(|batch| {
            batch
                .column(column)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .values()
                .to_vec()
        })
        .collect()
}

#[tokio::test]
async fn test_query_returns_arrow_batches() {
    let server = TestServer::new();
    let mut client = server.connect();

    assert!(matches!(
        client.login(PROTOCOL_VERSION_1, None, "token").await,
        Message::AuthResponse { success: true, .. }
    ));

    let batches = client
        .query("SELECT 1 AS a UNION ALL SELECT 2 AS a")
        .await
        .unwrap();
    let mut values = int_column(&batches, 0);
    values.sort();
    assert_eq!(values, vec![1, 2]);
    assert_eq!(batches[0].schema().field(0).name(), "a");

    client.close().await.unwrap();
}

#[tokio::test]
async fn test_protocol_v2_authenticates_a_user() {
    let server = TestServer::new();
    let mut client = server.connect();

    assert!(matches!(
        client
            .login(PROTOCOL_VERSION_2, Some("alice"), "secret")
            .await,
        Message::AuthResponse { success: true, .. }
    ));
    assert!(client.query("SELECT 1").await.is_ok());
}

#[tokio::test]
async fn test_wrong_password_is_refused_and_the_session_released() {
    let server = TestServer::new();
    let mut client = server.connect();

    match client
        .login(PROTOCOL_VERSION_2, Some("alice"), "wrong")
        .await
    {
        Message::AuthResponse {
            success: false,
            session_id: reason,
        } => assert!(reason.contains("alice"), "{}", reason),
        other => panic!("expected a refusal, got {:?}", other),
    }

    client.close().await.unwrap();
    assert_eq!(server.session_count().await, 0);
}

#[tokio::test]
async fn test_unknown_user_is_refused() {
    let server = TestServer::new();
    let mut client = server.connect();

    assert!(matches!(
        client
            .login(PROTOCOL_VERSION_2, Some("mallory"), "secret")
            .await,
        Message::AuthResponse { success: false, .. }
    ));
}

#[tokio::test]
async fn test_queries_run_with_the_authenticated_security_context() {
    let server = TestServer::new();
    let mut client = server.connect();
    client
        .login(PROTOCOL_VERSION_2, Some("alice"), "secret")
        .await;

    // The test transport has no data for it, so the query fails, but the
    // request it receives must carry the context alice authenticated with
    let _ = client
        .query("SELECT COUNT(*) FROM KibanaSampleDataEcommerce")
        .await;

    let calls = server.transport.load_calls().await;
    assert!(!calls.is_empty(), "the query never reached the transport");
    for call in calls {
        let ctx = call.ctx.as_any().downcast_ref::<HttpAuthContext>().unwrap();
        assert_eq!(ctx.access_token, "alice");
    }
}

#[tokio::test]
async fn test_a_failed_query_leaves_the_connection_usable() {
    let server = TestServer::new();
    let mut client = server.connect();
    client.login(PROTOCOL_VERSION_1, None, "token").await;

    let (code, message) = client.query("SELECT * FORM nowhere").await.unwrap_err();
    assert_eq!(code, "QUERY_ERROR");
    assert!(!message.is_empty());

    let batches = client.query("SELECT 3").await.unwrap();
    assert_eq!(int_column(&batches, 0), vec![3]);
}

#[tokio::test]
async fn test_session_statements_complete_without_results() {
    let server = TestServer::new();
    let mut client = server.connect();
    client.login(PROTOCOL_VERSION_1, None, "token").await;

    assert_eq!(
        client.query("SET application_name = 'test'").await,
        Ok(vec![])
    );
    assert_eq!(
        client
            .query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
            .await,
        Ok(vec![])
    );

    // Planned as a select, so it is answered like one
    let batches = client.query("SHOW application_name").await.unwrap();
    assert_eq!(batches[0].schema().field(0).name(), "setting");
}

#[tokio::test]
async fn test_unsupported_statements_are_reported() {
    let server = TestServer::new();
    let mut client = server.connect();
    client.login(PROTOCOL_VERSION_1, None, "token").await;

    let (code, _) = client
        .query("CREATE TEMPORARY TABLE t AS SELECT 1 AS a")
        .await
        .unwrap_err();
    assert_eq!(code, "UNSUPPORTED");

    assert!(client.query("SELECT 1").await.is_ok());
}

#[tokio::test]
async fn test_queries_before_authentication_are_refused() {
    let server = TestServer::new();
    let mut client = server.connect();
    client.handshake(PROTOCOL_VERSION_1).await;

    client
        .send(Message::QueryRequest {
            sql: "SELECT 1".to_string(),
        })
        .await;
    match client.recv().await {
        Message::Error { code, .. } => assert_eq!(code, "PROTOCOL_ERROR"),
        other => panic!("expected an error, got {:?}", other),
    }

    client.close().await.unwrap();
    assert_eq!(server.session_count().await, 0);
}

#[tokio::test]
async fn test_unsupported_version_is_refused_before_a_session_exists() {
    let server = TestServer::new();
    let mut client = server.connect();

    client.send(Message::HandshakeRequest { version: 99 }).await;
    match client.recv().await {
        Message::Error { code, .. } => assert_eq!(code, "UNSUPPORTED_VERSION"),
        other => panic!("expected an error, got {:?}", other),
    }
    assert_eq!(server.session_count().await, 0);
}

#[tokio::test]
async fn test_large_frame_before_authentication_is_refused_unread() {
    let server = TestServer::new();
    let client = server.connect();
    let Client {
        mut socket, handle, ..
    } = client;

    // A 1 MiB length prefix and no payload: the server must refuse it on the
    // length alone instead of waiting for (or allocating) the payload
    use tokio::io::AsyncWriteExt;
    socket
        .write_all(&(1024u32 * 1024).to_be_bytes())
        .await
        .unwrap();

    let err = handle.await.unwrap().unwrap_err();
    assert!(err.message.contains("exceeds the limit"), "{}", err.message);
}

#[tokio::test]
async fn test_session_limit_is_reported_to_the_client() {
    let server = TestServer::with_config(ConfigObjImpl {
        max_sessions: 1,
        ..ConfigObjImpl::default()
    });

    let mut first = server.connect();
    first.login(PROTOCOL_VERSION_1, None, "token").await;

    let mut second = server.connect();
    second
        .send(Message::HandshakeRequest {
            version: PROTOCOL_VERSION_1,
        })
        .await;
    assert!(matches!(
        second.recv().await,
        Message::HandshakeResponse { .. }
    ));
    match second.recv().await {
        Message::Error { code, .. } => assert_eq!(code, "TOO_MANY_CONNECTIONS"),
        other => panic!("expected an error, got {:?}", other),
    }
}
