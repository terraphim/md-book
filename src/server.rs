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
    serve_book_on(output_dir, "127.0.0.1", port, reload_tx).await
}

/// The routes a served book exposes: its files, and the live-reload socket.
///
/// Separated from [`serve_book_on`] so the behaviour can be tested with
/// `warp::test` rather than by binding a port.
#[cfg(feature = "server")]
pub fn book_routes(
    output_dir: String,
    reload_tx: broadcast::Sender<()>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let reload = warp::path("live-reload")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let reload_tx = reload_tx.clone();
            ws.on_upgrade(move |socket| handle_live_reload(socket, reload_tx))
        });

    let static_files =
        warp::fs::dir(output_dir.clone()).or(warp::fs::file(format!("{}/index.html", output_dir)));

    // The live-reload route must come first: the file fallback answers *any*
    // path, so behind it the websocket upgrade was never reached and the
    // browser silently got index.html instead of a socket.
    reload.or(static_files)
}

/// Resolve a hostname to an address to bind.
///
/// IP literals are used directly; names are resolved. A name that does not
/// resolve is an error rather than a silent fall back to loopback, which would
/// leave the server listening somewhere the operator was not told about.
///
/// # Errors
///
/// Returns an error when `hostname` is neither an IP literal nor resolvable.
#[cfg(feature = "server")]
pub fn resolve_bind_addr(hostname: &str, port: u16) -> Result<std::net::IpAddr> {
    if let Ok(ip) = hostname.parse() {
        return Ok(ip);
    }

    use std::net::ToSocketAddrs;
    (hostname, port)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.next())
        .map(|resolved| resolved.ip())
        .ok_or_else(|| anyhow::anyhow!("cannot resolve hostname '{hostname}'; pass an IP address"))
}

/// Serve on a specific hostname (`127.0.0.1`, `0.0.0.0`, or an IP literal).
#[cfg(feature = "server")]
pub async fn serve_book_on(
    output_dir: String,
    hostname: &str,
    port: u16,
    reload_tx: broadcast::Sender<()>,
) -> Result<()> {
    let addr = resolve_bind_addr(hostname, port)?;
    let routes = book_routes(output_dir, reload_tx);

    println!(
        "Serving book at http://{}:{} (bound to {})",
        hostname, port, addr
    );
    warp::serve(routes).run((addr, port)).await;
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

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn book_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("index.html"), "<h1>Home</h1>").unwrap();
        fs::create_dir_all(dir.path().join("guide")).unwrap();
        fs::write(dir.path().join("guide/intro.html"), "<h1>Intro</h1>").unwrap();
        dir
    }

    #[tokio::test]
    async fn test_serves_a_built_page() {
        let dir = book_dir();
        let (tx, _rx) = broadcast::channel(4);
        let routes = book_routes(dir.path().to_string_lossy().into_owned(), tx);

        let res = warp::test::request()
            .path("/guide/intro.html")
            .reply(&routes)
            .await;

        assert_eq!(res.status(), 200);
        assert!(String::from_utf8_lossy(res.body()).contains("Intro"));
    }

    #[tokio::test]
    async fn test_unknown_path_falls_back_to_index() {
        // A single-page-app style fallback: unknown paths serve index.html
        // rather than 404, which is what the dev server has always done.
        let dir = book_dir();
        let (tx, _rx) = broadcast::channel(4);
        let routes = book_routes(dir.path().to_string_lossy().into_owned(), tx);

        let res = warp::test::request()
            .path("/does/not/exist.html")
            .reply(&routes)
            .await;

        assert_eq!(res.status(), 200);
        assert!(String::from_utf8_lossy(res.body()).contains("Home"));
    }

    #[tokio::test]
    async fn test_live_reload_socket_upgrades_and_pushes_on_rebuild() {
        let dir = book_dir();
        let (tx, _rx) = broadcast::channel(4);
        let routes = book_routes(dir.path().to_string_lossy().into_owned(), tx.clone());

        let mut client = warp::test::ws()
            .path("/live-reload")
            .handshake(routes)
            .await
            .expect("live-reload should accept a websocket");

        // A rebuild broadcasts; the browser must be told to reload.
        tx.send(()).unwrap();
        let msg = client.recv().await.expect("expected a reload message");
        assert_eq!(msg.to_str().unwrap(), "reload");
    }

    #[test]
    fn test_resolve_bind_addr_accepts_ip_literals() {
        assert_eq!(
            resolve_bind_addr("127.0.0.1", 3000).unwrap().to_string(),
            "127.0.0.1"
        );
        assert_eq!(
            resolve_bind_addr("0.0.0.0", 3000).unwrap().to_string(),
            "0.0.0.0"
        );
    }

    #[test]
    fn test_resolve_bind_addr_resolves_localhost() {
        let addr = resolve_bind_addr("localhost", 3000).unwrap();
        assert!(addr.is_loopback(), "localhost should resolve to loopback");
    }

    #[test]
    fn test_resolve_bind_addr_rejects_unresolvable_names() {
        // Silently binding loopback while reporting the requested name left the
        // server listening somewhere the operator was never told about.
        let err = resolve_bind_addr("nope.invalid", 3000).unwrap_err();
        assert!(err.to_string().contains("cannot resolve"), "{err}");
    }
}
