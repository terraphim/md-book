use anyhow::Result;
#[cfg(feature = "server")]
use futures::{SinkExt, StreamExt};
#[cfg(feature = "server")]
use tokio::sync::broadcast;
#[cfg(feature = "server")]
use warp::ws::{Message, WebSocket};
#[cfg(feature = "server")]
use warp::Filter;

#[cfg(feature = "server")]
pub async fn serve_book(
    output_dir: String,
    port: u16,
    reload_tx: broadcast::Sender<()>,
) -> Result<()> {
    let static_files =
        warp::fs::dir(output_dir.clone()).or(warp::fs::file(format!("{}/index.html", output_dir)));

    // Add WebSocket route for live reload
    let reload = warp::path("live-reload")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let reload_tx = reload_tx.clone();
            ws.on_upgrade(move |socket| handle_live_reload(socket, reload_tx))
        });

    println!("Serving book at http://localhost:{}", port);
    warp::serve(static_files.or(reload))
        .run(([127, 0, 0, 1], port))
        .await;
    Ok(())
}

#[cfg(feature = "server")]
async fn handle_live_reload(ws: WebSocket, reload_tx: broadcast::Sender<()>) {
    let mut rx = reload_tx.subscribe();
    let (mut ws_tx, _) = ws.split();

    while rx.recv().await.is_ok() {
        if let Err(e) = ws_tx.send(Message::text("reload")).await {
            eprintln!("WebSocket send error: {}", e);
            break;
        }
    }
}
