use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use anyhow::Result;

pub fn watch<F>(path: &str, callback: F) -> Result<()> 
where
    F: Fn() -> Result<()> + Send + 'static,
{
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
    watcher.watch(Path::new(path), RecursiveMode::Recursive)?;

    println!("Watching for changes in {}...", path);

    loop {
        match rx.recv() {
            Ok(_event) => {
                println!("Change detected, rebuilding...");
                if let Err(e) = callback() {
                    eprintln!("Error rebuilding: {}", e);
                }
            }
            Err(e) => eprintln!("Watch error: {}", e),
        }
    }
}
