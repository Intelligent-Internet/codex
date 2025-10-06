//! HTTP Server Library with SSE Support
//!
//! This library implements an HTTP transport with support for both standard responses
//! and Server-Sent Events (SSE) streaming.
//!
//! # Features
//!
//! - **HTTP Messages**: Simple message wrapper using Codex EventMsg protocol
//! - **SSE Streaming**: Server-Sent Events for real-time streaming responses
//! - **Keep-alive**: Automatic ping messages every 15 seconds for SSE connections
//!
//! # Example
//!
//! ```no_run
//! use codex_http_server::{HttpServer, MessageHandler, HandlerResponse, HttpMessage};
//! use codex_protocol::protocol::EventMsg;
//! use std::net::SocketAddr;
//! use anyhow::Result;
//!
//! struct MyHandler;
//!
//! #[async_trait::async_trait]
//! impl MessageHandler for MyHandler {
//!     async fn handle_request(&self, request: HttpMessage) -> Result<HandlerResponse> {
//!         // Echo the request back
//!         let response = HttpMessage {
//!             id: request.id,
//!             work_dir: request.work_dir,
//!             event: request.event,
//!         };
//!         Ok(HandlerResponse::Standard(response))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let addr: SocketAddr = "127.0.0.1:8080".parse()?;
//!     let server = HttpServer::new(addr, MyHandler);
//!     server.run().await
//! }
//! ```

pub mod agent_handler;
pub mod message;
pub mod server;

// Re-export main types for convenience
pub use agent_handler::AgentHandler;
pub use codex_protocol::protocol::{Event, EventMsg};
pub use message::HttpMessage;
pub use server::{HandlerResponse, HttpServer, MessageHandler};
