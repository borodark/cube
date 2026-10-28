//! The Arrow Native endpoint: the SQL API with results returned as Arrow IPC.

mod ipc;
pub mod protocol;
mod server;

#[cfg(test)]
mod tests;

pub use server::ArrowNativeServer;
