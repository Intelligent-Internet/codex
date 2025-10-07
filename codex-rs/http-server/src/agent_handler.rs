use crate::{AGENT_MD_CONTENT, HandlerResponse, MessageHandler, message::HttpMessage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use codex_core::{CodexConversation, ConversationManager, config::Config as CodexConfig};
use codex_protocol::protocol::{AskForApproval, EventMsg, InputItem, Op, SandboxPolicy};
use futures::stream::Stream;
use std::fs;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub struct AgentHandler {
    conversation_manager: Arc<ConversationManager>,
    config: CodexConfig,
    dangerously_bypass_approvals_and_sandbox: bool,
}

impl AgentHandler {
    /// Create a new real Codex handler
    pub fn new(
        conversation_manager: Arc<ConversationManager>,
        config: CodexConfig,
        dangerously_bypass_approvals_and_sandbox: bool,
    ) -> Self {
        Self {
            conversation_manager,
            config,
            dangerously_bypass_approvals_and_sandbox,
        }
    }

    /// Run a Codex session and stream events
    async fn run_codex_session(
        conversation: Arc<CodexConversation>,
    ) -> Pin<Box<dyn Stream<Item = EventMsg> + Send>> {
        let stream = async_stream::stream! {
            loop {
                match conversation.next_event().await {
                    Ok(event) => {
                        let event_msg = event.msg.clone();

                        // Filter out some event types
                        let should_yield = !matches!(
                            event_msg,
                            EventMsg::AgentMessageDelta(_) | EventMsg::AgentReasoningDelta(_) | EventMsg::AgentReasoningRawContentDelta(_) | EventMsg::TokenCount(_)
                        );

                        // Yield the event if not filtered
                        if should_yield {
                            yield event_msg.clone();
                        }

                        // Check if we should stop streaming
                        match event_msg {
                            EventMsg::TaskComplete(_) | EventMsg::Error(_) => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("Codex runtime error: {}", e);
                        yield EventMsg::Error(codex_protocol::protocol::ErrorEvent {
                            message: format!("Codex runtime error: {e}"),
                        });
                        break;
                    }
                }
            }
        };

        Box::pin(stream)
    }
}

#[async_trait]
impl MessageHandler for AgentHandler {
    async fn handle_request(&self, request: HttpMessage) -> Result<HandlerResponse> {
        info!(
            "Running real Codex session for request: id={:?}",
            request.id
        );
        debug!("Received event type: {:?}", request.event);

        // Extract the prompt from the request event
        let prompt = match &request.event {
            EventMsg::UserMessage(msg) => {
                info!("Received UserMessage: {}", msg.message);
                msg.message.clone()
            }
            EventMsg::AgentMessage(msg) => {
                info!("Received AgentMessage: {}", msg.message);
                msg.message.clone()
            }
            other => {
                error!("Invalid request event type: {:?}", other);
                return Err(anyhow::anyhow!(
                    "Invalid request: expected UserMessage or AgentMessage event, got {other:?}"
                ));
            }
        };

        // Apply request-specific configuration overrides
        let mut config = self.config.clone();

        // Override working directory if provided
        if let Some(work_dir) = &request.work_dir {
            info!("Using working directory: {}", work_dir);
            config.cwd = PathBuf::from(work_dir);
        }

        // Override approval and sandbox policies based on server flags
        if self.dangerously_bypass_approvals_and_sandbox {
            info!("Bypassing approvals and sandbox (dangerous mode enabled)");
            config.approval_policy = AskForApproval::Never;
            config.sandbox_policy = SandboxPolicy::DangerFullAccess;
        }

        // Create AGENTS.md if it doesn't exist in the working directory
        let agents_file = config.cwd.join("AGENTS.md");
        if !agents_file.exists() {
            match fs::write(&agents_file, AGENT_MD_CONTENT) {
                Ok(_) => info!("Created AGENTS.md at {:?}", agents_file),
                Err(e) => warn!("Warning: Could not create AGENTS.md: {}", e),
            }
        }

        // Create codex_context.md if it doesn't exist in the working directory
        let context_file = config.cwd.join("codex_context.md");
        if !context_file.exists() {
            match fs::write(&context_file, "") {
                Ok(_) => info!("Created codex_context.md at {:?}", context_file),
                Err(e) => warn!("Warning: Could not create codex_context.md: {}", e),
            }
        }

        // Create a new Codex conversation
        let new_conv = self
            .conversation_manager
            .new_conversation(config)
            .await
            .context("Failed to create Codex conversation")?;

        let conversation = new_conv.conversation;

        // Submit the initial prompt
        conversation
            .submit(Op::UserInput {
                items: vec![InputItem::Text { text: prompt }],
            })
            .await
            .context("Failed to submit initial prompt")?;

        // Create and return the event stream
        let stream = Self::run_codex_session(conversation).await;

        Ok(HandlerResponse::Stream(stream))
    }
}
