use warp::Filter;
use std::path::Path;
use std::net::SocketAddr;
use anyhow::Result;

pub async fn serve(dir: &str, port: u16) -> Result<()> {
    let dir = Path::new(dir).to_path_buf();
    
    // Serve static files from the output directory
    let static_files = warp::fs::dir(dir.clone())
        .or(warp::fs::file(dir.join("index.html")));

    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    println!("Serving book at http://localhost:{}", port);
    
    warp::serve(static_files).run(addr).await;
    Ok(())
}
