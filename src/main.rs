use anyhow::Result;
use md_book::config;
use md_book::core::{build, Args};

#[cfg(any(feature = "server", feature = "watcher"))]
use futures::future;

#[cfg(feature = "server")]
use md_book::serve_book;
#[cfg(feature = "server")]
use tokio::sync::broadcast;

#[cfg(feature = "watcher")]
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(feature = "watcher")]
use tokio::time::Duration;

use clap::Parser;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    #[cfg(any(feature = "watcher", feature = "server"))]
    let watch_enabled = {
        #[cfg(feature = "watcher")]
        {
            args.watch
        }
        #[cfg(not(feature = "watcher"))]
        {
            false
        }
    };

    #[cfg(not(any(feature = "watcher", feature = "server")))]
    let watch_enabled = false;

    // Load configuration
    let config = config::load_config(args.config.as_deref())?;

    // Initial build
    #[cfg(any(feature = "server", feature = "watcher"))]
    build(&args, &config, watch_enabled).await?;
    #[cfg(not(any(feature = "server", feature = "watcher")))]
    build(&args, &config, watch_enabled)?;

    #[cfg(any(feature = "watcher", feature = "server"))]
    {
        let should_watch = {
            #[cfg(feature = "watcher")]
            {
                args.watch
            }
            #[cfg(not(feature = "watcher"))]
            {
                false
            }
        };

        let should_serve = {
            #[cfg(feature = "server")]
            {
                args.serve
            }
            #[cfg(not(feature = "server"))]
            {
                false
            }
        };

        if should_watch || should_serve {
            #[cfg(feature = "server")]
            let (reload_tx, _) = broadcast::channel(16);
            #[cfg(not(feature = "server"))]
            let reload_tx = ();

            let mut handles = vec![];

            // Start server if requested
            #[cfg(feature = "server")]
            if should_serve {
                let output_dir = args.output.clone();
                let port = args.port;
                let reload_tx = reload_tx.clone();

                handles.push(tokio::spawn(async move {
                    if let Err(e) = serve_book(output_dir, port, reload_tx).await {
                        eprintln!("Server error: {}", e);
                    }
                }));
            }

            // Start watcher if requested
            #[cfg(feature = "watcher")]
            if should_watch {
                let mut watch_paths = vec![args.input.clone()];
                if let Some(templates_dir) = get_templates_dir(&config) {
                    println!("Adding template dir to watch: {}", templates_dir);
                    watch_paths.push(templates_dir);
                }

                let args = args.clone();
                let config = config.clone();
                let reload_tx = reload_tx.clone();

                handles.push(tokio::spawn(async move {
                    if let Err(e) = watch_files(
                        watch_paths,
                        move || {
                            let args = args.clone();
                            let config = config.clone();
                            async move {
                                #[cfg(feature = "tokio")]
                                {
                                    build(&args, &config, watch_enabled).await
                                }
                                #[cfg(not(feature = "tokio"))]
                                {
                                    build(&args, &config, watch_enabled)
                                }
                            }
                        },
                        reload_tx,
                    )
                    .await
                    {
                        eprintln!("Watch error: {}", e);
                    }
                }));
            }

            // Keep the main task running
            if !handles.is_empty() {
                let _ = future::join_all(handles).await;
            }
        }
    }

    Ok(())
}

fn get_templates_dir(config: &md_book::BookConfig) -> Option<String> {
    let templates_dir = &config.paths.templates;
    if Path::new(templates_dir).exists() {
        Some(templates_dir.clone())
    } else {
        None
    }
}

#[cfg(feature = "watcher")]
async fn watch_files<F, Fut>(paths: Vec<String>, rebuild: F, reload_tx: ReloadSender) -> Result<()>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<()>> + Send,
{
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                println!("Change detected: {:?}", event);
                let _ = tx.blocking_send(());
            }
        },
        notify::Config::default(),
    )?;

    // Watch all paths
    for path in &paths {
        println!("Watching {}", path);
        watcher.watch(std::path::Path::new(path), RecursiveMode::Recursive)?;
    }

    // Debounce timer
    let mut debounce = tokio::time::interval(Duration::from_millis(500));
    let mut pending = false;

    loop {
        tokio::select! {
            Some(_) = rx.recv() => {
                pending = true;
            }
            _ = debounce.tick() => {
                if pending {
                    pending = false;
                    println!("Rebuilding...");
                    if let Err(e) = rebuild().await {
                        eprintln!("Rebuild error: {}", e);
                    } else {
                        #[cfg(feature = "server")]
                        { let _ = reload_tx.send(()); }
                    }
                }
            }
        }
    }
}

#[cfg(all(feature = "watcher", feature = "server"))]
type ReloadSender = broadcast::Sender<()>;

#[cfg(all(feature = "watcher", not(feature = "server")))]
type ReloadSender = ();
