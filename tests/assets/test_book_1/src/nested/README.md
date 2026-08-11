# Nested Content

This chapter tests nested directory structures and relative linking.

## Directory Structure

This file is located in a subdirectory to test how md-book handles:
- Nested navigation
- Relative path resolution  
- Asset linking from subdirectories

## Links Back to Parent

- [Back to Home](../README.md)
- [Basic Features](../basic.md)
- [Code Examples](../code.md)
- [Advanced Features](../advanced.md)

## Links to Siblings

- [Nested Chapter 1](chapter1.md)
- [Nested Chapter 2](chapter2.md)

## Local Content

This is content specific to the nested directory structure.

### Testing Path Resolution

When building this content, md-book should correctly:

1. Resolve relative paths to parent directories
2. Maintain proper navigation structure
3. Handle nested HTML output correctly
4. Preserve relative links in the final HTML

### Images and Assets

If there were images in this directory, they should be handled properly:

![Placeholder](https://via.placeholder.com/200x100/green/white?text=Nested+Image)

## Code Example in Nested Directory

```rust
// This code is in a nested directory
mod nested {
    pub fn hello() {
        println!("Hello from nested module!");
    }
}

fn main() {
    nested::hello();
}
```

## Navigation

Previous: [Advanced Features](../advanced.md) | Next: [Nested Chapter 1](chapter1.md)