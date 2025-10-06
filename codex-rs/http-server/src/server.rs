use crate::message::HttpMessage;
use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::post,
};
use codex_protocol::protocol::EventMsg;
use futures::stream::{Stream, StreamExt};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info};

/// HTTP server with SSE support
pub struct HttpServer {
    /// Address to bind the server to
    addr: SocketAddr,
    /// Message handler callback
    message_handler: Arc<dyn MessageHandler>,
}

/// Response type that handler can return
pub enum HandlerResponse {
    /// Standard HTTP response (non-streaming)
    Standard(HttpMessage),
    /// Streaming response with SSE
    Stream(Pin<Box<dyn Stream<Item = EventMsg> + Send>>),
}

/// Trait for handling incoming HTTP requests
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming HTTP message
    /// Returns either a standard response or a stream
    async fn handle_request(&self, request: HttpMessage) -> Result<HandlerResponse>;
}

/// Shared application state
#[derive(Clone)]
struct AppState {
    handler: Arc<dyn MessageHandler>,
}

impl HttpServer {
    /// Create a new HTTP server
    pub fn new<H>(addr: SocketAddr, handler: H) -> Self
    where
        H: MessageHandler + 'static,
    {
        Self {
            addr,
            message_handler: Arc::new(handler),
        }
    }

    /// Start the HTTP server with graceful shutdown
    pub async fn run(self) -> Result<()> {
        let state = AppState {
            handler: Arc::clone(&self.message_handler),
        };

        // Configure CORS to allow all origins for development
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/messages", post(handle_messages))
            .route("/health", axum::routing::get(health_check))
            .layer(cors)
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(self.addr)
            .await
            .context("Failed to bind to address")?;

        info!("MCP HTTP server listening on {}", self.addr);
        info!("Endpoint: POST /messages");

        // Set up graceful shutdown signal
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        // Spawn a task to listen for Ctrl+C
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            info!("Shutdown signal received, starting graceful shutdown...");
            shutdown_tx.send(()).ok();
        });

        // Run server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .context("Server error")?;

        info!("Server stopped gracefully");
        Ok(())
    }
}

/// Handle POST /messages - HTTP endpoint
async fn handle_messages(
    State(state): State<AppState>,
    Json(request): Json<HttpMessage>,
) -> Response {
    debug!("Received HTTP request: id={:?}", request.id);
    debug!("Event type: {:?}", request.event);

    // Handle the request
    match state.handler.handle_request(request.clone()).await {
        Ok(HandlerResponse::Standard(response)) => {
            // Return standard JSON response
            Json(response).into_response()
        }
        Ok(HandlerResponse::Stream(stream)) => {
            // Return SSE stream
            let request_id = request.id.clone();
            create_sse_response(stream, request_id).into_response()
        }
        Err(e) => {
            // Handler failed
            error!("Handler error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Create SSE response with keep-alive pings and client disconnect detection
fn create_sse_response(
    mut data_stream: Pin<Box<dyn Stream<Item = EventMsg> + Send>>,
    request_id: Option<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let event_stream = async_stream::stream! {
        let mut ping_interval = tokio::time::interval(Duration::from_secs(15));
        ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        debug!("Starting SSE stream for request: {:?}", request_id);

        loop {
            tokio::select! {
                // Send data from the handler stream
                item = data_stream.next() => {
                    match item {
                        Some(event_msg) => {
                            let msg = HttpMessage {
                                id: request_id.clone(),
                                work_dir: None,
                                event: event_msg,
                            };

                            match msg.to_json() {
                                Ok(json) => {
                                    // Try to send the event, if it fails the client disconnected
                                    yield Ok(Event::default().data(json));
                                }
                                Err(e) => {
                                    error!("Failed to serialize response: {}", e);
                                    break;
                                }
                            }
                        }
                        None => {
                            // Stream ended normally
                            debug!("Data stream ended for request: {:?}", request_id);
                            break;
                        }
                    }
                }
                // Send keep-alive ping every 15 seconds
                _ = ping_interval.tick() => {
                    // Ping helps detect client disconnects
                    yield Ok(Event::default().event("ping").data("ping"));
                }
            }
        }

        info!("SSE stream closed for request: {:?}", request_id);
    };

    Sse::new(event_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    )
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}
