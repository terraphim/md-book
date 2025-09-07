use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use anyhow::Result;
use md_book::{Args, BookConfig, build};

pub struct TestBook {
    pub temp_dir: TempDir,
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
    
    pub fn create_book_toml(&self, content: &str) -> Result<()> {
        let config_path = self.temp_dir.path().join("book.toml");
        fs::write(config_path, content)?;
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
        build(&args, &self.config, false).await
    }
    
    #[cfg(not(feature = "tokio"))]
    pub fn build(&self) -> Result<()> {
        let args = self.args();
        build(&args, &self.config, false)
    }
    
    pub fn output_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.output_dir.join(path).exists()
    }
    
    pub fn read_output<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let content = fs::read_to_string(self.output_dir.join(path))?;
        Ok(content)
    }
    
    pub fn input_path(&self) -> &Path {
        &self.input_dir
    }
    
    pub fn output_path(&self) -> &Path {
        &self.output_dir
    }
}

pub fn create_simple_book() -> Result<TestBook> {
    let book = TestBook::new()?;
    
    book.create_file("README.md", "# Test Book\n\nThis is a test book.")?;
    book.create_file("chapter1.md", "# Chapter 1\n\n## Section 1.1\n\nContent for section 1.1")?;
    book.create_file("chapter2.md", "# Chapter 2\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```")?;
    
    Ok(book)
}

pub fn create_complex_book() -> Result<TestBook> {
    let book = TestBook::new()?;
    
    // Create a more complex book structure
    book.create_file("README.md", 
        "# Complex Test Book\n\nThis book tests various markdown features.\n\n[Next Chapter](chapter1.md)")?;
    
    book.create_file("chapter1/README.md", 
        "# Chapter 1: Basics\n\n- Item 1\n- Item 2\n- Item 3\n\n[Section 1.1](section1.md)")?;
    
    book.create_file("chapter1/section1.md", 
        "## Section 1.1\n\n> This is a blockquote\n\n**Bold text** and *italic text*")?;
    
    book.create_file("chapter2.md", 
        "# Chapter 2: Code\n\n```rust\n// Rust code example\nfn fibonacci(n: u32) -> u32 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fibonacci(n - 1) + fibonacci(n - 2),\n    }\n}\n```\n\n```javascript\n// JavaScript example\nconst add = (a, b) => a + b;\n```")?;
    
    book.create_file("chapter3.md", 
        "# Chapter 3: Tables and Images\n\n| Name | Age | City |\n|------|-----|------|\n| Alice | 30 | NYC |\n| Bob | 25 | LA |\n\n![Test Image](https://via.placeholder.com/150)")?;
    
    Ok(book)
}

#[macro_export]
macro_rules! assert_contains {
    ($text:expr, $pattern:expr) => {
        assert!($text.contains($pattern), 
                "Expected text to contain '{}', but it didn't.\nFull text:\n{}", 
                $pattern, $text);
    };
}

#[macro_export]
macro_rules! assert_not_contains {
    ($text:expr, $pattern:expr) => {
        assert!(!$text.contains($pattern), 
                "Expected text to not contain '{}', but it did.\nFull text:\n{}", 
                $pattern, $text);
    };
}