//! Book model: ordered chapter tree from `SUMMARY.md` or a directory walk.

pub mod directory;
pub mod summary;

use std::path::{Path, PathBuf};

use serde::Serialize;

pub use directory::book_from_directory;
pub use summary::{book_from_summary, parse_summary, source_to_output, SummaryErrors};

/// A book: an ordered tree of items, as declared by SUMMARY.md
/// (or inferred from the directory tree when no summary exists).
#[derive(Debug, Clone, Serialize)]
pub struct Book {
    /// Top-level items in authored order.
    pub items: Vec<BookItem>,
    /// True when the structure came from SUMMARY.md rather than a directory walk.
    pub from_summary: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum BookItem {
    Chapter(Chapter),
    /// `# Part Name` — an unclickable heading in the sidebar.
    PartTitle(String),
    /// `---` — a horizontal rule in the sidebar.
    Separator,
}

#[derive(Debug, Clone, Serialize)]
pub struct Chapter {
    /// Link text from SUMMARY.md, or the first H1, or the file stem.
    pub name: String,
    /// Source path relative to the book's `src` dir. `None` for draft chapters.
    pub source_path: Option<PathBuf>,
    /// Output path relative to the build dir, e.g. `individual/heading.html`.
    pub output_path: Option<PathBuf>,
    /// `1.2.3`; `None` for prefix, suffix, draft and external chapters.
    pub number: Option<SectionNumber>,
    /// Nested sub-chapters, in authored order.
    pub sub_items: Vec<BookItem>,
    /// Optional URL fragment from SUMMARY (without `#`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment: Option<String>,
    /// True when this is an external link (no page generated).
    pub is_external: bool,
    /// Absolute external URL when `is_external`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
}

/// Dotted section number, e.g. `SectionNumber(vec![1, 2, 3])` → "1.2.3".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SectionNumber(pub Vec<u32>);

impl SectionNumber {
    pub fn to_label(&self) -> String {
        let body = self
            .0
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(".");
        format!("{body}.")
    }
}

impl std::fmt::Display for SectionNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_label())
    }
}

/// Kind of sidebar entry for Tera.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NavKind {
    Chapter,
    PartTitle,
    Separator,
}

/// One entry of the pre-flattened sidebar list handed to Tera.
#[derive(Debug, Clone, Serialize)]
pub struct NavEntry {
    pub kind: NavKind,
    /// Sidebar label: the SUMMARY link text (markdown left as-is for now; HTML contexts).
    pub title_html: String,
    /// Same label flattened to plain text, for `<title>` and `aria-label`.
    pub title_text: String,
    /// Href relative to the page being rendered, or an absolute external URL.
    pub href: Option<String>,
    pub is_external: bool,
    /// Rendered section number, e.g. "1.2."; empty when suppressed.
    pub number: String,
    pub depth: usize,
    /// `<ul>` elements to open before emitting this entry (0 or 1).
    pub open_lists: usize,
    /// `</ul></li>` pairs to close before emitting this entry.
    pub close_lists: usize,
    pub is_draft: bool,
    pub is_active: bool,
}

impl Book {
    /// Depth-first iteration over chapters that produce pages (have a source path).
    pub fn iter_chapters(&self) -> impl Iterator<Item = &Chapter> {
        BookChapterIter {
            stack: self.items.iter().rev().collect(),
        }
    }

    /// All chapters in authored order including drafts/external (for structure dumps).
    pub fn iter_all_chapters(&self) -> impl Iterator<Item = &Chapter> {
        BookAllChapterIter {
            stack: self.items.iter().rev().collect(),
        }
    }

    /// Flatten to a sidebar list for Tera, marking `active_path` as active.
    ///
    /// Nesting is encoded as deltas:
    /// - `close_lists`: emit `</li></ul>` this many times *before* the entry
    /// - `open_lists`: emit `<ul>` this many times, then `<li>…` for chapters
    /// - siblings at the same depth set `close_lists` to 0 and `open_lists` to 0;
    ///   the template closes the previous `</li>` when `open_lists == 0` and the
    ///   previous entry was a chapter (see sidebar.html.tera).
    pub fn to_nav(&self, active_path: &Path, no_section_label: bool) -> Vec<NavEntry> {
        let mut out = Vec::new();
        let mut list_depth: isize = -1;

        // Links are relative to the page being rendered, so output stays valid
        // under a sub-path and over file://. External links are left untouched.
        let root = crate::render::html::path_to_root(active_path);

        fn chapter_href(ch: &Chapter, root: &str) -> Option<String> {
            if ch.is_external {
                return ch.external_url.clone();
            }
            ch.output_path.as_ref().map(|out_path| {
                let mut h = format!("{root}{}", out_path.display());
                if let Some(ref frag) = ch.fragment {
                    h.push('#');
                    h.push_str(frag);
                }
                h
            })
        }

        fn emit_closes(list_depth: &mut isize, target_depth: isize) -> usize {
            let mut n = 0usize;
            while *list_depth > target_depth {
                n += 1;
                *list_depth -= 1;
            }
            n
        }

        #[allow(clippy::too_many_arguments)]
        fn walk(
            items: &[BookItem],
            depth: usize,
            active_path: &Path,
            no_section_label: bool,
            root: &str,
            list_depth: &mut isize,
            out: &mut Vec<NavEntry>,
        ) {
            for item in items {
                match item {
                    BookItem::Separator => {
                        let close_lists = emit_closes(list_depth, depth as isize - 1);
                        out.push(NavEntry {
                            kind: NavKind::Separator,
                            title_html: String::new(),
                            title_text: String::new(),
                            href: None,
                            is_external: false,
                            number: String::new(),
                            depth,
                            open_lists: 0,
                            close_lists,
                            is_draft: false,
                            is_active: false,
                        });
                    }
                    BookItem::PartTitle(title) => {
                        let close_lists = emit_closes(list_depth, depth as isize - 1);
                        out.push(NavEntry {
                            kind: NavKind::PartTitle,
                            title_html: title.clone(),
                            title_text: flatten_title(title),
                            href: None,
                            is_external: false,
                            number: String::new(),
                            depth,
                            open_lists: 0,
                            close_lists,
                            is_draft: false,
                            is_active: false,
                        });
                    }
                    BookItem::Chapter(ch) => {
                        let close_lists = emit_closes(list_depth, depth as isize);
                        let open_lists = if *list_depth < depth as isize {
                            *list_depth = depth as isize;
                            1
                        } else {
                            0
                        };

                        let is_draft = ch.source_path.is_none() && !ch.is_external;
                        let number = if no_section_label {
                            String::new()
                        } else {
                            ch.number.as_ref().map(|n| n.to_label()).unwrap_or_default()
                        };
                        let is_active = ch
                            .output_path
                            .as_ref()
                            .map(|p| p == active_path)
                            .unwrap_or(false);

                        out.push(NavEntry {
                            kind: NavKind::Chapter,
                            title_html: ch.name.clone(),
                            title_text: flatten_title(&ch.name),
                            href: chapter_href(ch, root),
                            is_external: ch.is_external,
                            number,
                            depth,
                            open_lists,
                            close_lists,
                            is_draft,
                            is_active,
                        });

                        if !ch.sub_items.is_empty() {
                            walk(
                                &ch.sub_items,
                                depth + 1,
                                active_path,
                                no_section_label,
                                root,
                                list_depth,
                                out,
                            );
                        }
                    }
                }
            }
        }

        walk(
            &self.items,
            0,
            active_path,
            no_section_label,
            &root,
            &mut list_depth,
            &mut out,
        );

        // Trailing closes after the last entry (marker separator, not rendered as <hr>)
        if list_depth >= 0 {
            out.push(NavEntry {
                kind: NavKind::Separator,
                title_html: "__trail__".into(),
                title_text: String::new(),
                href: None,
                is_external: false,
                number: String::new(),
                depth: 0,
                open_lists: 0,
                close_lists: (list_depth + 1) as usize,
                is_draft: false,
                is_active: false,
            });
        }

        out
    }

    /// Build the legacy flat `sections` view (one section per top-level chapter).
    pub fn to_legacy_sections(&self) -> Vec<crate::render::html::Section> {
        use crate::core::PageInfo;
        use crate::render::html::Section;

        let mut sections = Vec::new();
        for item in &self.items {
            if let BookItem::Chapter(ch) = item {
                let mut pages = Vec::new();
                if let Some(ref out) = ch.output_path {
                    pages.push(PageInfo {
                        title: flatten_title(&ch.name),
                        path: format!("{}", out.display()),
                    });
                }
                fn collect_desc(items: &[BookItem], pages: &mut Vec<PageInfo>) {
                    for i in items {
                        if let BookItem::Chapter(c) = i {
                            if let Some(ref out) = c.output_path {
                                pages.push(PageInfo {
                                    title: flatten_title(&c.name),
                                    path: format!("{}", out.display()),
                                });
                            }
                            collect_desc(&c.sub_items, pages);
                        }
                    }
                }
                collect_desc(&ch.sub_items, &mut pages);
                if !pages.is_empty() {
                    sections.push(Section {
                        title: flatten_title(&ch.name),
                        pages,
                    });
                }
            }
        }
        sections
    }
}

struct BookChapterIter<'a> {
    stack: Vec<&'a BookItem>,
}

impl<'a> Iterator for BookChapterIter<'a> {
    type Item = &'a Chapter;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.stack.pop() {
            match item {
                BookItem::Chapter(ch) => {
                    for sub in ch.sub_items.iter().rev() {
                        self.stack.push(sub);
                    }
                    // Only page-producing chapters
                    if ch.source_path.is_some() {
                        return Some(ch);
                    }
                }
                BookItem::PartTitle(_) | BookItem::Separator => {}
            }
        }
        None
    }
}

struct BookAllChapterIter<'a> {
    stack: Vec<&'a BookItem>,
}

impl<'a> Iterator for BookAllChapterIter<'a> {
    type Item = &'a Chapter;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.stack.pop() {
            match item {
                BookItem::Chapter(ch) => {
                    for sub in ch.sub_items.iter().rev() {
                        self.stack.push(sub);
                    }
                    return Some(ch);
                }
                BookItem::PartTitle(_) | BookItem::Separator => {}
            }
        }
        None
    }
}

/// Flatten inline markdown in titles for plain-text contexts.
pub fn flatten_title(title: &str) -> String {
    title
        .replace("**", "")
        .replace("__", "")
        .replace(['*', '_', '`', '[', ']'], "")
}

/// Load a book: SUMMARY.md if present, otherwise directory walk.
pub fn load_book(src_dir: &Path, create_missing: bool) -> anyhow::Result<(Book, Vec<PathBuf>)> {
    let summary_path = src_dir.join("SUMMARY.md");
    if summary_path.exists() {
        let content = std::fs::read_to_string(&summary_path)?;
        let summary = parse_summary(&content).map_err(|e| anyhow::anyhow!("{e}"))?;
        let (book, created) = book_from_summary(&summary, src_dir, create_missing)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok((book, created))
    } else {
        let book = book_from_directory(src_dir)?;
        Ok((book, Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summary::{book_from_summary, parse_summary};

    #[test]
    fn test_section_numbering_matches_nesting() {
        let dir = tempfile::TempDir::new().unwrap();
        for f in ["a.md", "b.md", "c.md", "d.md"] {
            std::fs::write(dir.path().join(f), format!("# {f}\n")).unwrap();
        }
        let content = r#"
- [A](a.md)
  - [B](b.md)
    - [C](c.md)
- [D](d.md)
"#;
        let s = parse_summary(content).unwrap();
        let (book, _) = book_from_summary(&s, dir.path(), false).unwrap();
        let nums: Vec<String> = book
            .iter_chapters()
            .filter_map(|c| c.number.as_ref().map(|n| n.to_label()))
            .collect();
        assert_eq!(nums, vec!["1.", "1.1.", "1.1.1.", "2."]);
    }

    #[test]
    fn test_prefix_suffix_draft_are_unnumbered() {
        let dir = tempfile::TempDir::new().unwrap();
        for f in ["prefix.md", "a.md", "suffix.md"] {
            std::fs::write(dir.path().join(f), "# x\n").unwrap();
        }
        let content = r#"
[Prefix](prefix.md)
- [A](a.md)
- [Draft]()
[Suffix](suffix.md)
"#;
        let s = parse_summary(content).unwrap();
        let (book, _) = book_from_summary(&s, dir.path(), false).unwrap();
        for ch in book.iter_all_chapters() {
            if ch.name == "A" {
                assert!(ch.number.is_some());
            } else {
                assert!(ch.number.is_none(), "{} should be unnumbered", ch.name);
            }
        }
    }

    #[test]
    fn test_iter_chapters_authored_order() {
        let dir = tempfile::TempDir::new().unwrap();
        for f in ["z.md", "a.md"] {
            std::fs::write(dir.path().join(f), "# x\n").unwrap();
        }
        let content = "- [Z](z.md)\n- [A](a.md)\n";
        let s = parse_summary(content).unwrap();
        let (book, _) = book_from_summary(&s, dir.path(), false).unwrap();
        let names: Vec<_> = book.iter_chapters().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["Z", "A"]);
    }

    #[test]
    fn test_to_nav_list_deltas_balance() {
        let dir = tempfile::TempDir::new().unwrap();
        for f in ["a.md", "b.md", "c.md"] {
            std::fs::write(dir.path().join(f), "# x\n").unwrap();
        }
        let content = r#"
- [A](a.md)
  - [B](b.md)
- [C](c.md)
"#;
        let s = parse_summary(content).unwrap();
        let (book, _) = book_from_summary(&s, dir.path(), false).unwrap();
        let nav = book.to_nav(Path::new("a.html"), false);
        let opens: usize = nav.iter().map(|e| e.open_lists).sum();
        let closes: usize = nav.iter().map(|e| e.close_lists).sum();
        assert_eq!(opens, closes, "open_lists must balance close_lists");
    }

    #[test]
    fn test_to_nav_depth_and_active() {
        let dir = tempfile::TempDir::new().unwrap();
        for f in ["a.md", "b.md"] {
            std::fs::write(dir.path().join(f), "# x\n").unwrap();
        }
        let content = "- [A](a.md)\n  - [B](b.md)\n";
        let s = parse_summary(content).unwrap();
        let (book, _) = book_from_summary(&s, dir.path(), false).unwrap();
        let nav = book.to_nav(Path::new("b.html"), false);
        let active: Vec<_> = nav.iter().filter(|e| e.is_active).collect();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title_text, "B");
        assert!(nav.iter().any(|e| e.depth == 1));
    }

    #[test]
    fn test_title_from_summary_not_h1() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "# Actual H1\n").unwrap();
        let content = "- [Summary Title](a.md)\n";
        let s = parse_summary(content).unwrap();
        let (book, _) = book_from_summary(&s, dir.path(), false).unwrap();
        let ch = book.iter_chapters().next().unwrap();
        assert_eq!(ch.name, "Summary Title");
    }
}
