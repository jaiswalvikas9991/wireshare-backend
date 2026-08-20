# Wireshare Backend

This is the WebSocket signaling backend for Wireshare, a peer-to-peer file sharing app. The backend does not receive, store, or proxy file contents. It only helps browsers discover each other and exchange WebRTC signaling messages so they can create a direct encrypted WebRTC data channel.

Frontend demo: https://wireshare.vercel.app/

> Note: the hosted backend runs on a free server. If it has been idle, the first request or WebSocket connection may be slow or may fail while the server wakes up. Wait a short moment, refresh the frontend, and try again.

Frontend repository: https://github.com/jaiswalvikas9991/wireshare

## What This Service Does

- Accepts WebSocket connections at `/`.
- Assigns each connected client a short random user ID.
- Sends the assigned user ID back to the client.
- Forwards signaling messages from one user ID to another.
- Sends heartbeat messages every 30 seconds.
- Exposes a simple health check at `/health`.

## What This Service Does Not Do

- It does not upload, download, store, inspect, or relay files.
- It does not act as a TURN server.
- It does not persist connected users across restarts.
- It does not provide authentication or account management.

## Tech Stack

- Rust
- Tokio
- Axum
- Axum WebSocket support
- Serde / serde_json
- nanoid
- Docker

## API

### `GET /`

Upgrades the request to a WebSocket connection.

When a client connects, the server generates a 10-character lowercase alphanumeric user ID and sends:

```json
{
  "type": 0,
  "userId": "abc123xyz0"
}
```

The frontend treats `type: 0` as the "connected to server" event.

### WebSocket Message Forwarding

Clients send JSON messages in this shape:

```json
{
  "toUserId": "peer-id",
  "msg": "{\"type\":1,\"fromUserId\":\"sender-id\",\"offer\":\"...\"}"
}
```

The backend looks up `toUserId` in the in-memory connection map and forwards the `msg` string as-is to that peer.

### Heartbeat

Every 30 seconds, the server sends this text message to connected clients:

```text
heartbeat
```

The frontend uses this to keep track of whether the signaling connection is still alive.

### `GET /health`

Returns:

```text
ok
```

## Getting Started

### Prerequisites

- Rust
- Cargo

### Run Locally

```bash
cargo run
```

By default, the server listens on:

```text
0.0.0.0:8080
```

Set `PORT` to run on a different port:

```bash
PORT=8000 cargo run
```

The frontend should point `WS_URL` to the local backend, for example:

```ts
export const WS_URL = 'ws://localhost:8000';
```

## Available Commands

The repository includes a `justfile`:

| Command | Description |
| --- | --- |
| `just build` | Run `cargo build`. |
| `just run_dev` | Run the backend locally on port `8000`. |

You can also use Cargo directly:

```bash
cargo build
cargo run
```

## Docker

Build the image:

```bash
docker build -t wireshare-backend .
```

Run the container:

```bash
docker run --rm -p 8080:8080 wireshare-backend
```

The Dockerfile builds the Rust binary in a builder image and runs it with a distroless Debian runtime image.

## Deployment

The service reads the port from the `PORT` environment variable and falls back to `8080`. This makes it suitable for platforms such as Render, Fly.io, Railway, or other container/Rust hosting providers.

For the hosted Wireshare demo, the frontend currently points to:

```text
wss://wireshare-backend-ucmx.onrender.com
```

Because that backend is hosted on a free tier, cold starts can affect the first connection attempt.

## Project Structure

```text
src/main.rs     Axum server, WebSocket upgrade route, heartbeat task, and message forwarding
Cargo.toml      Rust package metadata and dependencies
Dockerfile      Multi-stage production container build
justfile        Convenience commands for build and local development
```

## Security And Privacy Notes

- File data stays in the browser-to-browser WebRTC connection.
- Signaling messages pass through this service and may include WebRTC offers, answers, and ICE candidates.
- Connected users are stored only in memory.
- Restarting the server disconnects all active signaling sessions.
- A TURN server is not included, so some restrictive networks may fail to establish a peer-to-peer connection.

## License

MIT. See [LICENCE.md](LICENCE.md).
