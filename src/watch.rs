//! Filtering of filesystem events md-book caused itself.
//!
//! `build.create-missing` writes stub chapters into the source tree. Under
//! `--watch` the watcher sees those writes and would rebuild, so each created
//! path is recorded and the next event naming it is dropped.
//!
//! The record is *consumed* on match: a genuine user edit to the same file
//! immediately afterwards still triggers a rebuild.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Paths md-book wrote during the last build, pending one watcher event each.
#[derive(Debug, Default)]
pub struct SelfWriteFilter {
    pending: HashSet<PathBuf>,
}

impl SelfWriteFilter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record paths written by the build that just finished.
    ///
    /// Paths are canonicalised so they compare equal to the watcher's, which
    /// reports resolved paths. A path that cannot be canonicalised (deleted
    /// again already) is skipped: failing to suppress costs one rebuild,
    /// whereas suppressing the wrong path would drop a real change.
    pub fn record<I, P>(&mut self, paths: I)
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        for path in paths {
            if let Ok(canonical) = path.as_ref().canonicalize() {
                self.pending.insert(canonical);
            }
        }
    }

    /// Whether an event batch was caused solely by md-book's own writes.
    ///
    /// Returns `false` for an empty batch, so an event carrying no paths is
    /// always treated as a real change.
    pub fn should_ignore<I, P>(&mut self, event_paths: I) -> bool
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let paths: Vec<PathBuf> = event_paths
            .into_iter()
            .map(|p| {
                p.as_ref()
                    .canonicalize()
                    .unwrap_or_else(|_| p.as_ref().to_path_buf())
            })
            .collect();

        if paths.is_empty() {
            return false;
        }

        // Only ignore when *every* path in the batch is one of ours; a batch
        // mixing our stub with a user edit must still rebuild.
        if !paths.iter().all(|p| self.pending.contains(p)) {
            return false;
        }

        for path in &paths {
            self.pending.remove(path);
        }
        true
    }

    /// Number of writes still awaiting their event (diagnostics and tests).
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn touch(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, "# stub\n").unwrap();
        path
    }

    #[test]
    fn test_watch_suppresses_created_stub_event() {
        let dir = tempfile::TempDir::new().unwrap();
        let stub = touch(dir.path(), "later.md");

        let mut filter = SelfWriteFilter::new();
        filter.record([&stub]);

        // The event caused by create-missing is dropped...
        assert!(filter.should_ignore([&stub]));
        assert_eq!(filter.pending_count(), 0);

        // ...but only once: the next write to the same file is a user edit.
        assert!(!filter.should_ignore([&stub]));
    }

    #[test]
    fn test_unrelated_edits_are_never_suppressed() {
        let dir = tempfile::TempDir::new().unwrap();
        let stub = touch(dir.path(), "later.md");
        let edited = touch(dir.path(), "intro.md");

        let mut filter = SelfWriteFilter::new();
        filter.record([&stub]);

        assert!(!filter.should_ignore([&edited]));
        // The stub's own event is still pending and still suppressed.
        assert!(filter.should_ignore([&stub]));
    }

    #[test]
    fn test_mixed_batch_still_rebuilds() {
        let dir = tempfile::TempDir::new().unwrap();
        let stub = touch(dir.path(), "later.md");
        let edited = touch(dir.path(), "intro.md");

        let mut filter = SelfWriteFilter::new();
        filter.record([&stub]);

        // A batch containing a real edit must rebuild even though it also
        // carries our stub, or the user's change is silently lost.
        assert!(!filter.should_ignore([&stub, &edited]));
    }

    #[test]
    fn test_empty_batch_is_not_suppressed() {
        let mut filter = SelfWriteFilter::new();
        assert!(!filter.should_ignore(Vec::<PathBuf>::new()));
    }
}
