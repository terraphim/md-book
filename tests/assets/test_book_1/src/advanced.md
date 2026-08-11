# Advanced Features

This chapter tests advanced markdown features and edge cases.

## Tables

| Name | Age | City | Occupation |
|------|-----|------|------------|
| Alice | 30 | New York | Engineer |
| Bob | 25 | San Francisco | Designer |
| Charlie | 35 | Los Angeles | Manager |
| Diana | 28 | Chicago | Developer |

### Table with Alignment

| Left Aligned | Center Aligned | Right Aligned |
|:-------------|:--------------:|--------------:|
| Left | Center | Right |
| Text | Text | Text |
| A | B | C |

## Task Lists (GFM)

- [x] Completed task
- [x] Another completed task
- [ ] Incomplete task
- [ ] Another incomplete task
  - [x] Nested completed task
  - [ ] Nested incomplete task

## Strikethrough (GFM)

~~This text should be crossed out.~~

Regular text ~~with strikethrough in the middle~~ and more regular text.

## Definition Lists

Term 1
: Definition for term 1

Term 2
: First definition for term 2
: Second definition for term 2

Longer Term
: This is a longer definition that spans multiple lines
  and provides more detailed information about the term.

## Footnotes

This text has a footnote[^1] and another footnote[^note].

Here's a reference to the first footnote again[^1].

[^1]: This is the first footnote.
[^note]: This is a named footnote with more content.
    
    It can even contain multiple paragraphs and code:
    
    ```
    Code in footnote
    ```

## Math (LaTeX)

Inline math: $E = mc^2$

Block math:

$$
\sum_{i=1}^{n} x_i = x_1 + x_2 + \ldots + x_n
$$

More complex formula:

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

## Emoji

:smile: :heart: :rocket: :book: :computer:

## Abbreviations

The HTML specification defines many elements.

*[HTML]: HyperText Markup Language

## Superscript and Subscript

Water molecule: H~2~O

Einstein's equation: E = mc^2^

## Admonitions/Callouts

> **Note**
> This is an important note that readers should pay attention to.

> **Warning**
> This is a warning about something potentially dangerous.

> **Tip**
> This is a helpful tip for users.

## Complex Nested Structures

1. First item
   
   This is a paragraph within a list item.
   
   ```rust
   fn example() {
       println!("Code within list");
   }
   ```
   
   - Nested unordered list
   - Another nested item
   
   More paragraph content.

2. Second item with blockquote
   
   > This is a blockquote within a list item.
   > It can span multiple lines.
   
   And more content.

## HTML Entities and Special Characters

&copy; &trade; &reg; &hellip; &mdash; &ndash;

Quotes: &ldquo;smart quotes&rdquo; and &lsquo;single quotes&rsquo;

Math: &alpha; + &beta; = &gamma;

## URLs and Email Addresses

Automatic links:
- https://www.example.com
- http://test.org
- mailto:test@example.com

## Line Blocks

| This is a line block.
| Each line starts with a vertical bar.
| Preserves line breaks.
| Useful for poetry or addresses.

## Escape Characters

\*Not italic\* \[Not a link\] \`Not code\`

Special markdown characters: \\ \* \_ \{ \} \[ \] \( \) \# \+ \- \. \!

## Mixed Content Example

Here's a complex example mixing various elements:

### Recipe: Markdown Soup

**Ingredients:**
- 2 cups of *headings*
- 1 tablespoon of **bold text**
- A pinch of ~~strikethrough~~ (optional)
- `code snippets` to taste

**Instructions:**

1. Start with a base of [good content](README.md)
2. Add headings gradually:
   
   ```markdown
   # Like this
   ## And this
   ### And this too
   ```

3. Season with formatting:
   - **Bold** for emphasis
   - *Italic* for style
   - `Code` for technical bits

4. Serve with a side of tables:

   | Difficulty | Time | Serves |
   |:-----------|:----:|-------:|
   | Easy | 30 min | 4 people |

> **Tip:** Best enjoyed with syntax highlighting enabled!

**Nutritional Information:**[^nutrition]

[^nutrition]: Contains 100% daily value of readability.

Previous: [Code Examples](code.md) | Next: [Nested Content](nested/README.md)