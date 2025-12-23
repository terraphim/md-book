# src/main.rs Summary

## Purpose
Entry point for the md-book CLI application. Orchestrates the build process and manages optional development features (file watching and live server).

## Key Functionality
- Parses CLI arguments using `clap`
- Loads configuration via `config::load_config()`
- Executes initial build via `build()`
- Conditionally spawns async tasks for:
  - Development server (warp-based, serves output directory)
  - File watcher (debounced rebuild on file changes)
  - Live reload via WebSocket broadcast channel

## Feature Flags
- `server`: Enables HTTP server with live reload
- `watcher`: Enables file system watching for auto-rebuild

## Data Flow
1. Parse CLI args (`Args::parse()`)
2. Load config (`config::load_config()`)
3. Run initial build
4. If watch/serve enabled, spawn async tasks and join

## Dependencies
- `clap::Parser` for CLI
- `notify::RecommendedWatcher` for file watching
- `tokio::sync::broadcast` for reload signaling
