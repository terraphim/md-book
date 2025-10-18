use anyhow::Result;
use md_book::{Args, BookConfig};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestBook {
    #[allow(dead_code)]
    temp_dir: TempDir,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub config: BookConfig,
}

impl TestBook {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("book");

        fs::create_dir_all(&input_dir)?;
        fs::create_dir_all(&output_dir)?;

        let config = BookConfig::default();

        Ok(TestBook {
            temp_dir,
            input_dir,
            output_dir,
            config,
        })
    }

    pub fn with_config(mut self, config: BookConfig) -> Self {
        self.config = config;
        self
    }

    pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        let file_path = self.input_dir.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Ok(())
    }

    pub fn args(&self) -> Args {
        Args {
            input: self.input_dir.to_string_lossy().to_string(),
            output: self.output_dir.to_string_lossy().to_string(),
            config: None,
            #[cfg(feature = "watcher")]
            watch: false,
            #[cfg(feature = "server")]
            serve: false,
            #[cfg(feature = "server")]
            port: 3000,
        }
    }

    #[cfg(feature = "tokio")]
    pub async fn build(&self) -> Result<()> {
        let args = self.args();
        md_book::build(&args, &self.config, false).await
    }

    #[cfg(not(feature = "tokio"))]
    pub fn build(&self) -> Result<()> {
        let args = self.args();
        md_book::build(&args, &self.config, false)
    }

    pub fn output_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.output_dir.join(path).exists()
    }

    pub fn read_output<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let content = fs::read_to_string(self.output_dir.join(path))?;
        Ok(content)
    }

    #[allow(dead_code)]
    pub fn output_path(&self) -> &Path {
        &self.output_dir
    }
}

#[macro_export]
macro_rules! assert_contains {
    ($text:expr, $pattern:expr) => {
        assert!(
            $text.contains($pattern),
            "Expected text to contain '{}', but it didn't.\nFull text:\n{}",
            $pattern,
            $text
        );
    };
}

#[macro_export]
macro_rules! assert_not_contains {
    ($text:expr, $pattern:expr) => {
        assert!(
            !$text.contains($pattern),
            "Expected text to not contain '{}', but it did.\nFull text:\n{}",
            $pattern,
            $text
        );
    };
}
