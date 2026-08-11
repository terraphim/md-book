# Nested Chapter 2

This is the second chapter in the nested directory.

## Advanced Nested Content

This chapter contains more complex content to test edge cases.

### Tables in Nested Directories

| Feature | Status | Notes |
|---------|--------|-------|
| Nested tables | ✅ | Working |
| Path resolution | ✅ | Working |
| Cross-linking | ✅ | Working |

### Code Blocks

```python
# Python code in nested chapter 2
class NestedExample:
    def __init__(self, name):
        self.name = name
    
    def greet(self):
        return f"Hello from {self.name}!"

example = NestedExample("Nested Chapter 2")
print(example.greet())
```

### Blockquotes

> This is a blockquote in a nested chapter.
> It should render properly regardless of directory depth.

### Complex Links

Links to various locations:

- [Root README](../README.md) - Goes up one level
- [Previous nested chapter](chapter1.md) - Same level
- [Back to nested index](README.md) - Same directory
- [Advanced features](../advanced.md) - Up one level, different file

### Math (if supported)

Complex formula in nested content:

$$
\frac{d}{dx}\left( \int_{a}^{x} f(t) \, dt\right) = f(x)
$$

## End of Nested Content

This concludes the nested directory testing structure.

Previous: [Nested Chapter 1](chapter1.md) | Up: [Nested Index](README.md)