use anyhow::Result;
use clap::Parser;
use codex_arg0::arg0_dispatch_or_else;
use codex_common::CliConfigOverrides;
use codex_core::{
    AuthManager, ConversationManager,
    config::{Config as CodexConfig, ConfigOverrides},
};
use codex_http_server::{AgentHandler, HttpServer};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "sse-http-server")]
#[command(about = "HTTP Server with Codex Agent Handler")]
struct Args {
    /// Server bind address
    #[arg(short, long, default_value = "0.0.0.0:8081")]
    addr: String,

    /// Dangerously bypass approvals and sandbox
    #[arg(long, default_value = "true")]
    dangerously_bypass_approvals_and_sandbox: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    arg0_dispatch_or_else(|codex_linux_sandbox_exe| async move {
        run_main(
            codex_linux_sandbox_exe,
            CliConfigOverrides::default(),
            args.addr,
            args.dangerously_bypass_approvals_and_sandbox,
        )
        .await?;
        Ok(())
    })
}

async fn run_main(
    codex_linux_sandbox_exe: Option<PathBuf>,
    cli_config_overrides: CliConfigOverrides,
    addr_str: String,
    dangerously_bypass_approvals_and_sandbox: bool,
) -> Result<()> {
    // Initialize tracing with stderr output (like MCP server)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let addr: SocketAddr = addr_str.parse()?;

    // Parse CLI overrides (following MCP server pattern)
    let cli_kv_overrides = cli_config_overrides
        .parse_overrides()
        .map_err(|e| anyhow::anyhow!("Error parsing -c overrides: {e}"))?;

    // Load config with CLI overrides and sandbox exe path
    let config = CodexConfig::load_with_cli_overrides(
        cli_kv_overrides,
        ConfigOverrides {
            codex_linux_sandbox_exe,
            ..ConfigOverrides::default()
        },
    )
    .map_err(|e| anyhow::anyhow!("Error loading config: {e}"))?;

    // Initialize AuthManager and ConversationManager (following MCP server pattern)
    let auth_manager = AuthManager::shared(config.codex_home.clone());
    let conversation_manager = Arc::new(ConversationManager::new(auth_manager));

    // Create the RealHandler with ConversationManager and Config
    let handler = AgentHandler::new(
        conversation_manager,
        config,
        dangerously_bypass_approvals_and_sandbox,
    );

    // Create and run the server
    let server = HttpServer::new(addr, handler);

    server.run().await
}
