//! `SUMMARY.md` parser and book construction.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::{Book, BookItem, Chapter, SectionNumber};

/// Intermediate summary structure before path resolution and numbering.
#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub prefix: Vec<SummaryItem>,
    pub numbered: Vec<SummaryItem>,
    pub suffix: Vec<SummaryItem>,
}

#[derive(Debug, Clone)]
pub enum SummaryItem {
    Link(SummaryLink),
    PartTitle(String),
    Separator,
}

#[derive(Debug, Clone)]
pub struct SummaryLink {
    pub name: String,
    /// Raw target from the markdown link; `None` for drafts (`[Title]()`).
    pub location: Option<String>,
    /// Fragment without `#`, if present.
    pub fragment: Option<String>,
    pub nested: Vec<SummaryItem>,
    pub line: usize,
}

#[derive(Debug, Error)]
pub enum SummaryError {
    #[error("SUMMARY.md line {line}: cannot mix '-' and '*' list delimiters")]
    MixedDelimiters { line: usize },

    #[error(
        "SUMMARY.md line {line}: numbered chapter '{title}' appears after a suffix chapter; \
         suffix chapters must come last"
    )]
    NumberedAfterSuffix { line: usize, title: String },

    #[error("SUMMARY.md line {line}: expected a markdown link, found: {text}")]
    Malformed { line: usize, text: String },

    #[error("SUMMARY.md line {line}: '{path}' is already listed (line {first_line})")]
    DuplicateEntry {
        line: usize,
        path: PathBuf,
        first_line: usize,
    },

    #[error("SUMMARY.md line {line}: chapter target must be a .md file, found: {path}")]
    NonMarkdownTarget { line: usize, path: PathBuf },

    #[error(
        "SUMMARY.md line {line}: '{path}' resolves outside the source directory; refusing to read"
    )]
    EscapesSourceDir { line: usize, path: PathBuf },

    #[error("SUMMARY.md references missing file: {path}")]
    MissingFile { path: PathBuf },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Every problem found in one pass, reported together before aborting.
#[derive(Debug, Error)]
#[error("SUMMARY.md has {} problem(s):\n{}", .0.len(), format_errors(.0))]
pub struct SummaryErrors(pub Vec<SummaryError>);

fn format_errors(errors: &[SummaryError]) -> String {
    errors
        .iter()
        .map(|e| format!("  - {e}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse the contents of a `SUMMARY.md`.
///
/// Collects every problem found so one build reports all issues.
pub fn parse_summary(content: &str) -> Result<Summary, SummaryErrors> {
    let mut errors = Vec::new();
    let mut summary = Summary::default();
    let mut list_delimiter: Option<char> = None;

    enum Phase {
        Prefix,
        Numbered,
        Suffix,
    }
    let mut phase = Phase::Prefix;

    struct StackEntry {
        indent: usize,
        link: SummaryLink,
    }
    let mut stack: Vec<StackEntry> = Vec::new();
    let mut numbered_items: Vec<SummaryItem> = Vec::new();

    let flush_stack = |stack: &mut Vec<StackEntry>, out: &mut Vec<SummaryItem>| {
        while let Some(entry) = stack.pop() {
            let item = SummaryItem::Link(entry.link);
            if let Some(parent) = stack.last_mut() {
                parent.link.nested.push(item);
            } else {
                out.push(item);
            }
        }
    };

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end();
        if line.trim().is_empty() {
            continue;
        }

        if line.trim() == "---" {
            match phase {
                Phase::Prefix => summary.prefix.push(SummaryItem::Separator),
                Phase::Numbered => {
                    flush_stack(&mut stack, &mut numbered_items);
                    numbered_items.push(SummaryItem::Separator);
                }
                Phase::Suffix => summary.suffix.push(SummaryItem::Separator),
            }
            continue;
        }

        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            let is_heading = level > 0
                && (trimmed.len() == level || trimmed.as_bytes().get(level) == Some(&b' '));
            if is_heading {
                let title = trimmed[level..].trim().to_string();
                if matches!(phase, Phase::Prefix)
                    && summary.prefix.is_empty()
                    && numbered_items.is_empty()
                    && title.eq_ignore_ascii_case("summary")
                {
                    continue;
                }
                flush_stack(&mut stack, &mut numbered_items);
                if matches!(phase, Phase::Prefix) {
                    phase = Phase::Numbered;
                }
                match phase {
                    Phase::Prefix => summary.prefix.push(SummaryItem::PartTitle(title)),
                    Phase::Numbered => numbered_items.push(SummaryItem::PartTitle(title)),
                    Phase::Suffix => summary.suffix.push(SummaryItem::PartTitle(title)),
                }
                continue;
            }
        }

        let indent = line.len() - line.trim_start().len();
        let after_indent = line.trim_start();

        if after_indent.starts_with("- ") || after_indent.starts_with("* ") {
            let delim = after_indent.chars().next().unwrap();
            if let Some(existing) = list_delimiter {
                if existing != delim {
                    errors.push(SummaryError::MixedDelimiters { line: line_no });
                    continue;
                }
            } else {
                list_delimiter = Some(delim);
            }

            let link_text = after_indent[2..].trim();
            match parse_link(link_text, line_no) {
                Ok(link) => {
                    if matches!(phase, Phase::Prefix) {
                        phase = Phase::Numbered;
                    }
                    if matches!(phase, Phase::Suffix) {
                        // A list item after a suffix chapter cannot be numbered without
                        // reordering the book. Report rather than silently absorbing it
                        // into the suffix, which would drop its section number.
                        errors.push(SummaryError::NumberedAfterSuffix {
                            line: line_no,
                            title: link.name.clone(),
                        });
                        continue;
                    }
                    while stack.last().map(|e| e.indent >= indent).unwrap_or(false) {
                        let finished = stack.pop().unwrap();
                        let item = SummaryItem::Link(finished.link);
                        if let Some(parent) = stack.last_mut() {
                            parent.link.nested.push(item);
                        } else {
                            numbered_items.push(item);
                        }
                    }
                    stack.push(StackEntry { indent, link });
                }
                Err(e) => errors.push(e),
            }
            continue;
        }

        if after_indent.starts_with('[') {
            flush_stack(&mut stack, &mut numbered_items);
            match parse_link(after_indent, line_no) {
                Ok(link) => {
                    let item = SummaryItem::Link(link);
                    match phase {
                        Phase::Prefix => summary.prefix.push(item),
                        Phase::Numbered => {
                            summary.numbered.append(&mut numbered_items);
                            phase = Phase::Suffix;
                            summary.suffix.push(item);
                        }
                        Phase::Suffix => summary.suffix.push(item),
                    }
                }
                Err(e) => errors.push(e),
            }
            continue;
        }

        errors.push(SummaryError::Malformed {
            line: line_no,
            text: line.to_string(),
        });
    }

    flush_stack(&mut stack, &mut numbered_items);
    if !numbered_items.is_empty() {
        summary.numbered.append(&mut numbered_items);
    }

    if errors.is_empty() {
        Ok(summary)
    } else {
        Err(SummaryErrors(errors))
    }
}

fn parse_link(text: &str, line: usize) -> Result<SummaryLink, SummaryError> {
    let text = text.trim();
    if !text.starts_with('[') {
        return Err(SummaryError::Malformed {
            line,
            text: text.to_string(),
        });
    }
    let mut depth = 0usize;
    let mut close_bracket = None;
    for (i, c) in text.char_indices().skip(1) {
        match c {
            '[' => depth += 1,
            ']' => {
                if depth == 0 {
                    close_bracket = Some(i);
                    break;
                }
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    let close = close_bracket.ok_or_else(|| SummaryError::Malformed {
        line,
        text: text.to_string(),
    })?;
    let name = text[1..close].to_string();
    let rest = &text[close + 1..];
    if !rest.starts_with('(') {
        return Err(SummaryError::Malformed {
            line,
            text: text.to_string(),
        });
    }
    let end = rest.rfind(')').ok_or_else(|| SummaryError::Malformed {
        line,
        text: text.to_string(),
    })?;
    let target = &rest[1..end];
    let is_external =
        target.starts_with("http://") || target.starts_with("https://") || target.starts_with("//");
    let (location, fragment) = if target.is_empty() {
        (None, None)
    } else if is_external {
        // Preserve full external URL including any fragment
        (Some(target.to_string()), None)
    } else if let Some((path, frag)) = target.split_once('#') {
        (Some(path.to_string()), Some(frag.to_string()))
    } else {
        (Some(target.to_string()), None)
    };

    Ok(SummaryLink {
        name,
        location,
        fragment,
        nested: Vec::new(),
        line,
    })
}

/// Normalise a relative path for duplicate detection (collapse `.` / `..`).
/// Returns `None` if the path is absolute or would escape via parent components.
fn normalize_rel_path(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(c) => out.push(c),
            std::path::Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => return None,
        }
    }
    Some(out)
}

/// Ensure a not-yet-created path cannot write outside `src_dir` via symlinked ancestors.
fn ensure_missing_target_contained(
    src_dir: &Path,
    rel: &Path,
    line: usize,
) -> Result<(), SummaryError> {
    let canonical_src = src_dir.canonicalize().map_err(SummaryError::Io)?;
    let full = src_dir.join(rel);

    let mut ancestor = full.as_path();
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .ok_or_else(|| SummaryError::EscapesSourceDir {
                line,
                path: rel.to_path_buf(),
            })?;
    }

    let canonical_ancestor = ancestor.canonicalize().map_err(SummaryError::Io)?;
    if !canonical_ancestor.starts_with(&canonical_src) {
        return Err(SummaryError::EscapesSourceDir {
            line,
            path: rel.to_path_buf(),
        });
    }
    Ok(())
}

/// Build a `Book` from a summary, resolving paths against `src_dir` and assigning section numbers.
pub fn book_from_summary(
    summary: &Summary,
    src_dir: &Path,
    create_missing: bool,
) -> Result<(Book, Vec<PathBuf>), SummaryErrors> {
    let mut errors = Vec::new();
    let mut created = Vec::new();
    let mut seen: HashMap<PathBuf, usize> = HashMap::new();
    let mut items = Vec::new();

    for item in &summary.prefix {
        if let Some(i) = convert_unnumbered(
            item,
            src_dir,
            create_missing,
            &mut seen,
            &mut created,
            &mut errors,
        ) {
            items.push(i);
        }
    }

    let mut counters: Vec<u32> = Vec::new();
    for item in &summary.numbered {
        if let Some(i) = convert_numbered(
            item,
            src_dir,
            create_missing,
            0,
            &mut counters,
            &mut seen,
            &mut created,
            &mut errors,
        ) {
            items.push(i);
        }
    }

    for item in &summary.suffix {
        if let Some(i) = convert_unnumbered(
            item,
            src_dir,
            create_missing,
            &mut seen,
            &mut created,
            &mut errors,
        ) {
            items.push(i);
        }
    }

    if !errors.is_empty() {
        return Err(SummaryErrors(errors));
    }

    Ok((
        Book {
            items,
            from_summary: true,
        },
        created,
    ))
}

fn convert_unnumbered(
    item: &SummaryItem,
    src_dir: &Path,
    create_missing: bool,
    seen: &mut HashMap<PathBuf, usize>,
    created: &mut Vec<PathBuf>,
    errors: &mut Vec<SummaryError>,
) -> Option<BookItem> {
    match item {
        SummaryItem::PartTitle(t) => Some(BookItem::PartTitle(t.clone())),
        SummaryItem::Separator => Some(BookItem::Separator),
        SummaryItem::Link(link) => {
            let mut chapter =
                link_to_chapter(link, src_dir, create_missing, None, seen, created, errors)?;
            let mut nested = Vec::new();
            for n in &link.nested {
                if let Some(ni) =
                    convert_unnumbered(n, src_dir, create_missing, seen, created, errors)
                {
                    nested.push(ni);
                }
            }
            chapter.sub_items = nested;
            Some(BookItem::Chapter(chapter))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn convert_numbered(
    item: &SummaryItem,
    src_dir: &Path,
    create_missing: bool,
    depth: usize,
    counters: &mut Vec<u32>,
    seen: &mut HashMap<PathBuf, usize>,
    created: &mut Vec<PathBuf>,
    errors: &mut Vec<SummaryError>,
) -> Option<BookItem> {
    match item {
        SummaryItem::PartTitle(t) => Some(BookItem::PartTitle(t.clone())),
        SummaryItem::Separator => Some(BookItem::Separator),
        SummaryItem::Link(link) => {
            let is_draft = link.location.is_none();
            let is_external = link
                .location
                .as_ref()
                .map(|l| {
                    l.starts_with("http://") || l.starts_with("https://") || l.starts_with("//")
                })
                .unwrap_or(false);

            let number = if is_draft || is_external {
                None
            } else {
                if counters.len() <= depth {
                    counters.resize(depth + 1, 0);
                }
                counters.truncate(depth + 1);
                counters[depth] += 1;
                Some(SectionNumber(counters[..=depth].to_vec()))
            };

            let mut chapter =
                link_to_chapter(link, src_dir, create_missing, number, seen, created, errors)?;

            if !is_draft && !is_external {
                counters.truncate(depth + 1);
            }

            let mut nested = Vec::new();
            for n in &link.nested {
                if let Some(ni) = convert_numbered(
                    n,
                    src_dir,
                    create_missing,
                    depth + 1,
                    counters,
                    seen,
                    created,
                    errors,
                ) {
                    nested.push(ni);
                }
            }
            chapter.sub_items = nested;
            Some(BookItem::Chapter(chapter))
        }
    }
}

fn link_to_chapter(
    link: &SummaryLink,
    src_dir: &Path,
    create_missing: bool,
    number: Option<SectionNumber>,
    seen: &mut HashMap<PathBuf, usize>,
    created: &mut Vec<PathBuf>,
    errors: &mut Vec<SummaryError>,
) -> Option<Chapter> {
    let location = match &link.location {
        None => {
            return Some(Chapter {
                name: link.name.clone(),
                source_path: None,
                output_path: None,
                number: None,
                sub_items: Vec::new(),
                fragment: None,
                is_external: false,
                external_url: None,
            });
        }
        Some(loc) => loc,
    };

    if location.starts_with("http://")
        || location.starts_with("https://")
        || location.starts_with("//")
    {
        return Some(Chapter {
            name: link.name.clone(),
            source_path: None,
            output_path: None,
            number: None,
            sub_items: Vec::new(),
            fragment: None,
            is_external: true,
            external_url: Some(location.clone()),
        });
    }

    let raw_path = PathBuf::from(location);
    if raw_path.extension().and_then(|e| e.to_str()) != Some("md") {
        errors.push(SummaryError::NonMarkdownTarget {
            line: link.line,
            path: raw_path.clone(),
        });
        return None;
    }

    let path = match normalize_rel_path(&raw_path) {
        Some(p) => p,
        None => {
            errors.push(SummaryError::EscapesSourceDir {
                line: link.line,
                path: raw_path.clone(),
            });
            return None;
        }
    };

    if path_escapes_src(src_dir, &path) {
        errors.push(SummaryError::EscapesSourceDir {
            line: link.line,
            path: path.clone(),
        });
        return None;
    }

    if let Some(&first) = seen.get(&path) {
        errors.push(SummaryError::DuplicateEntry {
            line: link.line,
            path: path.clone(),
            first_line: first,
        });
        return None;
    }
    seen.insert(path.clone(), link.line);

    let full = src_dir.join(&path);
    if !full.exists() {
        if create_missing {
            if let Err(e) = ensure_missing_target_contained(src_dir, &path, link.line) {
                errors.push(e);
                return None;
            }
            if let Some(parent) = full.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    errors.push(SummaryError::Io(e));
                    return None;
                }
                // Re-check parent after create_dir_all (symlink races)
                if let Ok(canon_src) = src_dir.canonicalize() {
                    if let Ok(canon_parent) = parent.canonicalize() {
                        if !canon_parent.starts_with(&canon_src) {
                            errors.push(SummaryError::EscapesSourceDir {
                                line: link.line,
                                path: path.clone(),
                            });
                            return None;
                        }
                    }
                }
            }
            let stub = format!("# {}\n", strip_md_for_heading(&link.name));
            if let Err(e) = fs::write(&full, stub) {
                errors.push(SummaryError::Io(e));
                return None;
            }
            created.push(full);
        } else {
            errors.push(SummaryError::MissingFile { path: path.clone() });
            return None;
        }
    }

    Some(Chapter {
        name: link.name.clone(),
        source_path: Some(path.clone()),
        output_path: Some(source_to_output(&path)),
        number,
        sub_items: Vec::new(),
        fragment: link.fragment.clone(),
        is_external: false,
        external_url: None,
    })
}

fn strip_md_for_heading(name: &str) -> String {
    // Flatten simple markdown emphasis for stub H1
    name.replace("**", "").replace(['*', '`'], "")
}

fn path_escapes_src(src_dir: &Path, rel: &Path) -> bool {
    let Ok(canonical_src) = src_dir.canonicalize() else {
        return true;
    };
    let joined = src_dir.join(rel);
    if joined.exists() {
        return joined
            .canonicalize()
            .map(|c| !c.starts_with(&canonical_src))
            .unwrap_or(true);
    }
    // Walk components without leaving src
    let mut cur = canonical_src.clone();
    for comp in rel.components() {
        match comp {
            std::path::Component::ParentDir => {
                if cur == canonical_src {
                    return true;
                }
                match cur.parent() {
                    Some(p) if p.starts_with(&canonical_src) => cur = p.to_path_buf(),
                    _ => return true,
                }
            }
            std::path::Component::Normal(c) => cur.push(c),
            std::path::Component::CurDir => {}
            std::path::Component::RootDir | std::path::Component::Prefix(_) => return true,
        }
    }
    !cur.starts_with(&canonical_src)
}

/// Map `README.md` → `index.html`, other `foo.md` → `foo.html`.
pub fn source_to_output(source: &Path) -> PathBuf {
    let file_name = source
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("index.md");
    let out_name = if file_name.eq_ignore_ascii_case("README.md") {
        "index.html".to_string()
    } else {
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("page");
        format!("{stem}.html")
    };
    match source.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(out_name),
        _ => PathBuf::from(out_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prefix_suffix_chapters() {
        let content = r#"# Summary

[Prefix](prefix.md)

---

- [Intro](intro.md)

---

[Suffix](suffix.md)
"#;
        let s = parse_summary(content).unwrap();
        assert!(matches!(&s.prefix[0], SummaryItem::Link(l) if l.name == "Prefix"));
        assert!(matches!(&s.numbered[0], SummaryItem::Link(l) if l.name == "Intro"));
        assert!(matches!(&s.suffix[0], SummaryItem::Link(l) if l.name == "Suffix"));
    }

    #[test]
    fn test_parse_nested_chapters_three_deep() {
        let content = r#"
- [A](a.md)
  - [B](b.md)
    - [C](c.md)
"#;
        let s = parse_summary(content).unwrap();
        match &s.numbered[0] {
            SummaryItem::Link(a) => match &a.nested[0] {
                SummaryItem::Link(b) => match &b.nested[0] {
                    SummaryItem::Link(c) => assert_eq!(c.name, "C"),
                    _ => panic!("expected C"),
                },
                _ => panic!("expected B"),
            },
            _ => panic!("expected A"),
        }
    }

    #[test]
    fn test_parse_part_titles_and_separators() {
        let content = r#"
# Part One

- [A](a.md)

---

# Part Two

- [B](b.md)
"#;
        let s = parse_summary(content).unwrap();
        assert!(matches!(&s.numbered[0], SummaryItem::PartTitle(t) if t == "Part One"));
        assert!(matches!(&s.numbered[2], SummaryItem::Separator));
        assert!(matches!(&s.numbered[3], SummaryItem::PartTitle(t) if t == "Part Two"));
    }

    #[test]
    fn test_parse_draft_chapter() {
        let content = "- [Draft]()\n";
        let s = parse_summary(content).unwrap();
        assert!(matches!(&s.numbered[0], SummaryItem::Link(l) if l.location.is_none()));
    }

    #[test]
    fn test_parse_rejects_mixed_delimiters() {
        let content = "- [A](a.md)\n* [B](b.md)\n";
        let err = parse_summary(content).unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, SummaryError::MixedDelimiters { .. })));
    }

    #[test]
    fn test_parse_collects_all_errors_in_one_pass() {
        let content = "- [A](a.md)\n* [B](b.md)\nnot a link\n";
        let err = parse_summary(content).unwrap_err();
        assert!(err.0.len() >= 2);
    }

    #[test]
    fn test_parse_rejects_numbered_after_suffix() {
        // A bare link after numbered chapters is a suffix chapter; a list item
        // after that cannot be numbered without reordering the book.
        let content = "- [One](one.md)\n[Suffix](suffix.md)\n- [Two](two.md)\n";
        let err = parse_summary(content).unwrap_err();

        let found = err.0.iter().find_map(|e| match e {
            SummaryError::NumberedAfterSuffix { line, title } => Some((*line, title.clone())),
            _ => None,
        });
        let (line, title) = found.expect("expected NumberedAfterSuffix");
        assert_eq!(line, 3);
        assert_eq!(title, "Two");
    }

    #[test]
    fn test_parse_accepts_suffix_chapters_in_sequence() {
        // Regression guard for the fix above: consecutive bare links after the
        // numbered section are all suffix chapters and must stay valid.
        let content = "- [One](one.md)\n[First](a.md)\n[Second](b.md)\n";
        let summary = parse_summary(content).expect("suffix chapters should parse");
        assert_eq!(summary.suffix.len(), 2);
        assert_eq!(summary.numbered.len(), 1);
    }

    #[test]
    fn test_parse_rejects_duplicate_entry() {
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join("a.md"), "# A\n").unwrap();
        let content = "- [A](a.md)\n- [A again](a.md)\n";
        let s = parse_summary(content).unwrap();
        let err = book_from_summary(&s, dir.path(), false).unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, SummaryError::DuplicateEntry { .. })));
    }

    #[test]
    fn test_parse_rejects_non_markdown_target() {
        let dir = tempfile::TempDir::new().unwrap();
        let content = "- [Sample](sample.rs)\n";
        let s = parse_summary(content).unwrap();
        let err = book_from_summary(&s, dir.path(), false).unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, SummaryError::NonMarkdownTarget { .. })));
    }

    #[test]
    fn test_parse_accepts_anchor_link() {
        let content = "- [API](api.md#errors)\n";
        let s = parse_summary(content).unwrap();
        match &s.numbered[0] {
            SummaryItem::Link(l) => {
                assert_eq!(l.location.as_deref(), Some("api.md"));
                assert_eq!(l.fragment.as_deref(), Some("errors"));
            }
            _ => panic!("expected link"),
        }
    }

    #[test]
    fn test_parse_accepts_external_url() {
        let content = "- [Rust](https://rust-lang.org)\n";
        let s = parse_summary(content).unwrap();
        match &s.numbered[0] {
            SummaryItem::Link(l) => {
                assert_eq!(l.location.as_deref(), Some("https://rust-lang.org"));
            }
            _ => panic!("expected link"),
        }
    }

    #[test]
    fn test_summary_rejects_paths_escaping_src() {
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join("ok.md"), "# Ok\n").unwrap();
        let content = "- [Bad](../../etc/passwd.md)\n- [Ok](ok.md)\n";
        let s = parse_summary(content).unwrap();
        let err = book_from_summary(&s, dir.path(), false).unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, SummaryError::EscapesSourceDir { .. })));
    }

    #[test]
    fn test_create_missing_writes_stub_once() {
        let dir = tempfile::TempDir::new().unwrap();
        let content = "- [New Chapter](new.md)\n";
        let s = parse_summary(content).unwrap();
        let (book, created) = book_from_summary(&s, dir.path(), true).unwrap();
        assert_eq!(created.len(), 1);
        let stub = fs::read_to_string(dir.path().join("new.md")).unwrap();
        assert!(stub.contains("# New Chapter"));
        fs::write(dir.path().join("new.md"), "# Custom\n").unwrap();
        let (_book2, created2) = book_from_summary(&s, dir.path(), true).unwrap();
        assert!(created2.is_empty());
        assert_eq!(
            fs::read_to_string(dir.path().join("new.md")).unwrap(),
            "# Custom\n"
        );
        assert!(book.from_summary);
    }

    #[test]
    fn test_readme_maps_to_index_html() {
        assert_eq!(
            source_to_output(Path::new("README.md")),
            PathBuf::from("index.html")
        );
        assert_eq!(
            source_to_output(Path::new("individual/README.md")),
            PathBuf::from("individual/index.html")
        );
        assert_eq!(
            source_to_output(Path::new("individual/heading.md")),
            PathBuf::from("individual/heading.html")
        );
    }

    #[test]
    fn test_duplicate_detects_normalized_paths() {
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join("a.md"), "# A\n").unwrap();
        let content = "- [A](a.md)\n- [A2](./a.md)\n";
        let s = parse_summary(content).unwrap();
        let err = book_from_summary(&s, dir.path(), false).unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, SummaryError::DuplicateEntry { .. })));
    }

    #[test]
    fn test_external_url_preserves_fragment() {
        let content = "- [Rust](https://doc.rust-lang.org/book/#ownership)\n";
        let s = parse_summary(content).unwrap();
        match &s.numbered[0] {
            SummaryItem::Link(l) => {
                assert_eq!(
                    l.location.as_deref(),
                    Some("https://doc.rust-lang.org/book/#ownership")
                );
            }
            _ => panic!("expected link"),
        }
    }

    #[test]
    fn test_create_missing_rejects_symlink_escape() {
        let dir = tempfile::TempDir::new().unwrap();
        let outside = tempfile::TempDir::new().unwrap();
        let link = dir.path().join("escape");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(outside.path(), &link).unwrap();
            let content = "- [Bad](escape/evil.md)\n";
            let s = parse_summary(content).unwrap();
            let err = book_from_summary(&s, dir.path(), true).unwrap_err();
            assert!(
                err.0
                    .iter()
                    .any(|e| matches!(e, SummaryError::EscapesSourceDir { .. })),
                "expected EscapesSourceDir, got {err:?}"
            );
            assert!(!outside.path().join("evil.md").exists());
        }
    }
}
