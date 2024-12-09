use anyhow::Result;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use warp::ws::{Message, WebSocket};
use warp::Filter;

pub(crate) async fn serve_book(
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
            ws.on_upgrade(move |socket| {
                println!("WebSocket connection attempt");
                handle_live_reload(socket, reload_tx)
            })
        });

    // Make sure reload route is matched first
    println!("Serving book at http://localhost:{}", port);
    warp::serve(reload.or(static_files))
        .run(([127, 0, 0, 1], port))
        .await;
    Ok(())
}

async fn handle_live_reload(ws: WebSocket, reload_tx: broadcast::Sender<()>) {
    println!("WebSocket connected successfully");
    let mut rx = reload_tx.subscribe();
    let (mut ws_tx, _) = ws.split();

    while rx.recv().await.is_ok() {
        println!("Sending reload message");
        if let Err(e) = ws_tx.send(Message::text("reload")).await {
            eprintln!("WebSocket send error: {}", e);
            break;
        }
    }
}
