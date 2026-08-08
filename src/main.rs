use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use md_book::config::BookConfig;
use md_book::core::{build, Args};
use md_book::paths::{self, BookPaths};
use std::path::{Path, PathBuf};

#[cfg(any(feature = "server", feature = "watcher"))]
use futures::future;

#[cfg(feature = "server")]
use tokio::sync::broadcast;

#[cfg(feature = "watcher")]
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(feature = "watcher")]
use tokio::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input directory containing markdown files (legacy / override)
    #[arg(short, long, global = true)]
    input: Option<String>,

    /// Output directory for HTML files (legacy / override)
    #[arg(short = 'o', long, global = true)]
    output: Option<String>,

    /// Destination directory (mdBook-compatible alias for --output)
    #[arg(short = 'd', long = "dest-dir", global = true)]
    dest_dir: Option<String>,

    /// Optional path to config file
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Watch for changes and rebuild
    #[arg(short, long, global = true)]
    #[cfg(feature = "watcher")]
    watch: bool,

    /// Serve the book
    #[arg(short, long, global = true)]
    #[cfg(feature = "server")]
    serve: bool,

    /// Port to serve on (default: 3000)
    #[arg(long, default_value = "3000", global = true)]
    #[cfg(feature = "server")]
    port: u16,

    /// Hostname to bind (default: 127.0.0.1)
    #[arg(short = 'n', long, default_value = "127.0.0.1", global = true)]
    #[cfg(feature = "server")]
    hostname: String,

    /// Open browser after serve (best-effort)
    #[arg(long, global = true)]
    open: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build the book
    Build {
        /// Book directory (contains book.toml)
        dir: Option<PathBuf>,
    },
    /// Build and serve with live reload
    Serve { dir: Option<PathBuf> },
    /// Watch and rebuild on change
    Watch { dir: Option<PathBuf> },
    /// Create a new book scaffold
    Init { dir: Option<PathBuf> },
    /// Remove the build directory
    Clean { dir: Option<PathBuf> },
}

#[cfg(any(
    feature = "server",
    feature = "watcher",
    feature = "search",
    feature = "core"
))]
#[tokio::main]
async fn main() -> Result<()> {
    main_impl().await
}

#[cfg(not(any(
    feature = "server",
    feature = "watcher",
    feature = "search",
    feature = "core"
)))]
fn main() -> Result<()> {
    main_impl_sync()
}

async fn main_impl() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { dir }) => {
            let d = dir.clone().unwrap_or_else(|| PathBuf::from("."));
            return paths::init_book(&d);
        }
        Some(Commands::Clean { dir }) => {
            let d = dir.clone().unwrap_or_else(|| PathBuf::from("."));
            let config = load_config_resolved(Some(d.as_path()), cli.config.as_deref())?;
            let resolved = resolve_paths(&cli, Some(d.as_path()), &config)?;
            if resolved.build.exists() {
                std::fs::remove_dir_all(&resolved.build)
                    .with_context(|| format!("Failed to remove {}", resolved.build.display()))?;
                println!("Removed {}", resolved.build.display());
            }
            return Ok(());
        }
        _ => {}
    }

    let (book_dir, force_watch, force_serve) = match &cli.command {
        Some(Commands::Build { dir }) => (dir.clone(), false, false),
        Some(Commands::Serve { dir }) => (dir.clone(), false, true),
        Some(Commands::Watch { dir }) => (dir.clone(), true, false),
        None => (None, false, false),
        _ => unreachable!(),
    };

    let config = load_config_resolved(book_dir.as_deref(), cli.config.as_deref())?;
    let resolved = resolve_paths(&cli, book_dir.as_deref(), &config)?;
    resolved.validate_for_build(cli.input.is_some())?;

    let args = Args {
        input: resolved.src.to_string_lossy().into_owned(),
        output: resolved.build.to_string_lossy().into_owned(),
        config: cli.config.clone(),
        #[cfg(feature = "watcher")]
        watch: force_watch || cli.watch,
        #[cfg(feature = "server")]
        serve: force_serve || cli.serve,
        #[cfg(feature = "server")]
        port: cli.port,
    };

    #[cfg(any(feature = "watcher", feature = "server"))]
    let watch_enabled = {
        #[cfg(feature = "watcher")]
        {
            args.watch || force_watch
        }
        #[cfg(not(feature = "watcher"))]
        {
            false
        }
    };

    #[cfg(not(any(feature = "watcher", feature = "server")))]
    let watch_enabled = false;

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

            #[cfg(feature = "server")]
            if should_serve {
                let output_dir = args.output.clone();
                let port = args.port;
                let hostname = cli.hostname.clone();
                let reload_tx = reload_tx.clone();
                if cli.open {
                    println!("Open http://{}:{}/ in your browser", hostname, port);
                }

                handles.push(tokio::spawn(async move {
                    if let Err(e) =
                        serve_book_with_host(output_dir, &hostname, port, reload_tx).await
                    {
                        eprintln!("Server error: {}", e);
                    }
                }));
            }

            #[cfg(feature = "watcher")]
            if should_watch {
                let mut watch_paths = vec![args.input.clone()];
                if let Some(templates_dir) = get_templates_dir(&config) {
                    watch_paths.push(templates_dir);
                }
                for extra in &config.build.extra_watch_dirs {
                    let p = book_dir
                        .as_ref()
                        .map(|d| d.join(extra))
                        .unwrap_or_else(|| PathBuf::from(extra));
                    watch_paths.push(p.to_string_lossy().into_owned());
                }

                let args_clone = args.clone();
                let config_clone = config.clone();
                #[cfg(feature = "server")]
                let reload_tx = reload_tx.clone();

                handles.push(tokio::spawn(async move {
                    if let Err(e) = watch_and_rebuild(
                        watch_paths,
                        args_clone,
                        config_clone,
                        #[cfg(feature = "server")]
                        reload_tx,
                    )
                    .await
                    {
                        eprintln!("Watcher error: {}", e);
                    }
                }));
            }

            if !handles.is_empty() {
                let _ = future::join_all(handles).await;
            }
        }
    }

    Ok(())
}

fn load_config_resolved(book_dir: Option<&Path>, config_path: Option<&str>) -> Result<BookConfig> {
    use twelf::Layer;
    let mut layers = vec![Layer::Env(Some("MDBOOK_".to_string()))];

    if let Some(dir) = book_dir {
        let candidate = dir.join("book.toml");
        if candidate.exists() {
            layers.push(Layer::Toml(candidate));
        }
    } else if Path::new("book.toml").exists() {
        layers.push(Layer::Toml("book.toml".into()));
    }

    if let Some(path) = config_path {
        if Path::new(path).exists() {
            if Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
            {
                layers.push(Layer::Toml(path.into()));
            } else if Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                layers.push(Layer::Json(path.into()));
            } else {
                anyhow::bail!("Unsupported config file type: {}", path);
            }
        }
    }

    BookConfig::with_layers(&layers).context("Failed to load configuration")
}

fn resolve_paths(cli: &Cli, book_dir: Option<&Path>, config: &BookConfig) -> Result<BookPaths> {
    let output = cli.dest_dir.as_deref().or(cli.output.as_deref());
    paths::resolve(book_dir, cli.input.as_deref(), output, config)
}

#[cfg(not(any(
    feature = "server",
    feature = "watcher",
    feature = "search",
    feature = "core"
)))]
fn main_impl_sync() -> Result<()> {
    let cli = Cli::parse();
    if let Some(Commands::Init { dir }) = &cli.command {
        let d = dir.clone().unwrap_or_else(|| PathBuf::from("."));
        return paths::init_book(&d);
    }
    let book_dir = match &cli.command {
        Some(Commands::Build { dir }) => dir.clone(),
        _ => None,
    };
    let config = load_config_resolved(book_dir.as_deref(), cli.config.as_deref())?;
    let resolved = resolve_paths(&cli, book_dir.as_deref(), &config)?;
    let args = Args {
        input: resolved.src.to_string_lossy().into_owned(),
        output: resolved.build.to_string_lossy().into_owned(),
        config: cli.config.clone(),
    };
    build(&args, &config, false)?;
    Ok(())
}

fn get_templates_dir(config: &BookConfig) -> Option<String> {
    let p = &config.paths.templates;
    if Path::new(p).exists() {
        Some(p.clone())
    } else {
        None
    }
}

#[cfg(feature = "server")]
async fn serve_book_with_host(
    output_dir: String,
    hostname: &str,
    port: u16,
    reload_tx: broadcast::Sender<()>,
) -> Result<()> {
    md_book::server::serve_book_on(output_dir, hostname, port, reload_tx).await
}

#[cfg(feature = "watcher")]
async fn watch_and_rebuild(
    watch_paths: Vec<String>,
    args: Args,
    config: BookConfig,
    #[cfg(feature = "server")] reload_tx: broadcast::Sender<()>,
) -> Result<()> {
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    for path in &watch_paths {
        if Path::new(path).exists() {
            watcher.watch(Path::new(path), RecursiveMode::Recursive)?;
            println!("Watching {}", path);
        }
    }

    loop {
        match rx.recv() {
            Ok(_event) => {
                // Debounce briefly
                tokio::time::sleep(Duration::from_millis(200)).await;
                // Drain pending
                while rx.try_recv().is_ok() {}
                println!("Change detected, rebuilding…");
                if let Err(e) = build(&args, &config, true).await {
                    eprintln!("Rebuild failed: {e}");
                } else {
                    #[cfg(feature = "server")]
                    {
                        let _ = reload_tx.send(());
                    }
                }
            }
            Err(e) => {
                eprintln!("Watch error: {e}");
                break;
            }
        }
    }
    Ok(())
}
