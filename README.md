# MD book is a mdbook replacement with some extra features
## Run
Checkout the source code and run:

```rust
cargo run -- -i ../mdBook/test_book  -o ./test_mdbook
```

-i is the input directory and -o is the output directory.
input directory is the directory with md files.

input directory with markdown files and the tool will generate the output directory with the html files ready to be deployed on any static site. 

Adjust the styling in the src/templates/css/styles.css file.

Or anything you want to change in the src/templates folder, it's a standard tera template so you can add your own custom stuff there.

## Styling

* Nicer default styling for content - multiple columns for horizontal layout,
* Right hand TOC to navigate around the page.
* Create index.md to create a content for home page alternatively it will create a list of cards with all the pages as index.


- Code blocks with syntax highlighting
- Copy to clipboard button (doesn't work)
- Better default styling

# Screenshots

![screen_resize](gif/screen_resize.gif)
![screen](gif/screen.gif)
