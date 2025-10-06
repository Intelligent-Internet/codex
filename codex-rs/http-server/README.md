# Codex HTTP Server

**HTTP Server with Codex Agent Handler** - A production-ready HTTP server that executes Codex conversations via Server-Sent Events (SSE).

## Features

- ✅ **Real Codex Integration**: Executes actual Codex conversation sessions
- ✅ **SSE Streaming**: Real-time event streaming for conversation responses
- ✅ **Agent Handler**: Built-in support for Codex `EventMsg` protocol
- ✅ **Health Monitoring**: Health check endpoint for service monitoring
- ✅ **Configurable**: Command-line arguments for flexible deployment

## Architecture

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/messages` | POST | Execute Codex conversations (returns SSE stream) |
| `/health` | GET | Health check |

### How It Works

1. **Message Handling**: Client sends Codex event messages to `/messages`
2. **Conversation Execution**: Server creates a real Codex conversation session
3. **Event Streaming**: Server streams Codex events back via SSE
4. **Session Management**: Each request creates an independent conversation session

## Installation

Build the server binary:

```bash
cargo build --release --bin sse-http-server
```

## Usage

### Starting the Server

**Default configuration** (binds to `0.0.0.0:8081`):

```bash
./target/release/sse-http-server
```

**Custom address**:

```bash
./target/release/sse-http-server --addr 127.0.0.1:3000
```

**Custom address (short form)**:

```bash
./target/release/sse-http-server -a 0.0.0.0:9000
```

**Disable dangerous bypass mode**:

```bash
./target/release/sse-http-server --dangerously-bypass-approvals-and-sandbox false
```

**Help**:

```bash
./target/release/sse-http-server --help
```

### Command-Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--addr` | `-a` | Server bind address | `0.0.0.0:8081` |
| `--dangerously-bypass-approvals-and-sandbox` | | Dangerously bypass approvals and sandbox | `true` |

### Server Output

When started, the server displays:

```
╔═══════════════════════════════════════════════════════════════╗
║  HTTP Server - Codex Agent Handler                           ║
╚═══════════════════════════════════════════════════════════════╝

Server listening on: http://localhost:8081

Endpoints:
  POST /messages - HTTP endpoint (supports streaming)
  GET  /health   - Health check

RealHandler executes actual Codex conversations:
  - Receives messages via POST /messages
  - Creates Codex conversation session
  - Streams real Codex events back via SSE
```

## API Reference

### POST /messages

Execute a Codex conversation and receive streamed events.

**Request Format**:

```json
{
  "type": "user_message",
  "message": "Hello, can you help me?",
  "work_dir": "/path/to/project"
}
```

**Request Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | Event type (user_message, agent_message, etc.) |
| `message` | string | Yes (for user_message) | The message content |
| `work_dir` | string | No | Custom working directory for this request |
| `id` | string | No | Optional request ID |

**Response**: Server-Sent Events (SSE) stream

**Examples**:

Basic request:

```bash
curl -N -X POST http://localhost:8081/messages \
  -H "Content-Type: application/json" \
  -d '{"type":"user_message","message":"Hello, can you help me?"}'
```

Request with custom working directory:

```bash
curl -N -X POST http://localhost:8081/messages \
  -H "Content-Type: application/json" \
  -d '{"type":"user_message","message":"List files","work_dir":"/path/to/project"}'
```

### GET /health

Health check endpoint for monitoring.

**Response**:

```json
{
  "status": "ok"
}
```

**Example**:

```bash
curl http://localhost:8081/health
```

## Event Message Types

The server accepts Codex `EventMsg` types (in snake_case):

- `user_message` - User input message
- `user_confirm` - User confirmation
- `user_cancel` - User cancellation
- `user_edit` - User edit request
- Additional types as defined in `codex-protocol`

## Configuration

### Environment Variables

The server respects standard environment variables:

- `RUST_LOG` - Logging level (e.g., `info`, `debug`, `trace`)

**Example**:

```bash
RUST_LOG=debug ./target/release/sse-http-server
```

### Codex Configuration

The server uses the standard Codex configuration system:

- Config file: `~/.codex/config.json`
- Settings can be overridden via CLI flags (same as MCP server)

## Development

### Building

```bash
cargo build --bin sse-http-server
```

### Running in Development

```bash
cargo run --bin sse-http-server -- --addr 127.0.0.1:8081
```

### Testing

Test the server with curl:

```bash
curl -N -X POST http://localhost:8081/messages \
  -H "Content-Type: application/json" \
  -d '{"type":"user_message","message":"What is 2+2?"}'
```

### Logging

Enable debug logging:

```bash
RUST_LOG=debug cargo run --bin sse-http-server
```

Enable trace logging:

```bash
RUST_LOG=trace cargo run --bin sse-http-server
```

## Production Deployment

### Systemd Service

Example systemd unit file (`/etc/systemd/system/codex-http.service`):

```ini
[Unit]
Description=Codex HTTP Server
After=network.target

[Service]
Type=simple
User=codex
WorkingDirectory=/opt/codex
ExecStart=/opt/codex/sse-http-server --addr 0.0.0.0:8081
Restart=on-failure
RestartSec=5
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

### Reverse Proxy (nginx)

Example nginx configuration:

```nginx
server {
    listen 80;
    server_name codex.example.com;

    location / {
        proxy_pass http://127.0.0.1:8081;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache off;
        chunked_transfer_encoding off;
    }
}
```

### Docker

Example Dockerfile:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin sse-http-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/sse-http-server /usr/local/bin/
EXPOSE 8081
CMD ["sse-http-server", "--addr", "0.0.0.0:8081"]
```

## Troubleshooting

### Connection Issues

If clients can't connect:

- Verify the server is running: `curl http://localhost:8081/health`
- Check firewall settings
- Verify the bind address allows external connections (use `0.0.0.0` not `127.0.0.1`)

### SSE Not Streaming

For SSE streaming, clients must:

- Use `-N` flag with curl to disable buffering
- Set appropriate HTTP client timeouts
- Handle SSE events properly

### Authentication Errors

The server requires valid Codex authentication:

- Ensure `~/.codex/config.json` is properly configured
- Check authentication credentials are valid
- Review logs with `RUST_LOG=debug`
