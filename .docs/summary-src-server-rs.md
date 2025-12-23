# src/server.rs Summary

## Purpose
Development HTTP server with WebSocket-based live reload functionality.

## Key Function
`serve_book(output_dir: String, port: u16, reload_tx: broadcast::Sender<()>)`:
- Serves static files from output directory
- Falls back to index.html for SPA-style routing
- Provides `/live-reload` WebSocket endpoint
- Sends "reload" message to connected clients when rebuild completes

## Live Reload Flow
1. Browser connects to `/live-reload` WebSocket
2. Server subscribes to broadcast channel
3. On file change, rebuild triggers `reload_tx.send(())`
4. Server sends "reload" text message to all WebSocket clients
5. Client-side JS reloads page

## Dependencies
- `warp`: HTTP server and WebSocket support
- `tokio::sync::broadcast`: Multi-subscriber reload channel
- `futures`: Stream handling for WebSocket
