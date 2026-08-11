//! Directory-walk book construction (fallback when no SUMMARY.md exists).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;

use super::summary::source_to_output;
use super::{Book, BookItem, Chapter};
use crate::render::html::extract_title;

/// Today's behaviour: walk `src_dir`, sort by path, group by parent directory.
/// Used when no `SUMMARY.md` exists.
pub fn book_from_directory(src_dir: &Path) -> Result<Book> {
    let mut entries: Vec<_> = walkdir::WalkDir::new(src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();

    entries.sort_by_key(|e| e.path().to_path_buf());

    // Group by parent directory for legacy sections, but produce a flat
    // chapter list in path order (matches previous prev/next behaviour).
    let mut root_chapters: Vec<BookItem> = Vec::new();
    let mut by_parent: BTreeMap<String, Vec<BookItem>> = BTreeMap::new();

    for entry in &entries {
        let rel = entry.path().strip_prefix(src_dir)?;
        let parent = rel
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        let content = fs::read_to_string(entry.path())?;
        let name = extract_title(&content).unwrap_or_else(|| {
            entry.path().file_stem().map_or_else(
                || "Untitled".to_string(),
                |s| s.to_string_lossy().into_owned(),
            )
        });

        let chapter = Chapter {
            name,
            source_path: Some(rel.to_path_buf()),
            output_path: Some(source_to_output(rel)),
            number: None,
            sub_items: Vec::new(),
            fragment: None,
            is_external: false,
            external_url: None,
        };

        let item = BookItem::Chapter(chapter);
        if parent.is_empty() {
            root_chapters.push(item);
        } else {
            by_parent.entry(parent).or_default().push(item);
        }
    }

    // Preserve previous ordering: root pages first (as "Guide" section pages),
    // then each parent directory group. Represent as flat top-level chapters so
    // prev/next follows path sort within each group, and overall matches old
    // all_pages order (path-sorted).
    //
    // Old code path-sorted *all* entries globally then built sections. Prev/next
    // used that global order. So items must be path-sorted flat list.
    let mut all_items: Vec<BookItem> = Vec::new();
    for entry in &entries {
        let rel = entry.path().strip_prefix(src_dir)?;
        let content = fs::read_to_string(entry.path())?;
        let name = extract_title(&content).unwrap_or_else(|| {
            entry.path().file_stem().map_or_else(
                || "Untitled".to_string(),
                |s| s.to_string_lossy().into_owned(),
            )
        });
        all_items.push(BookItem::Chapter(Chapter {
            name,
            source_path: Some(rel.to_path_buf()),
            output_path: Some(source_to_output(rel)),
            number: None,
            sub_items: Vec::new(),
            fragment: None,
            is_external: false,
            external_url: None,
        }));
    }

    let _ = (root_chapters, by_parent); // reserved for legacy section titles if needed

    Ok(Book {
        items: all_items,
        from_summary: false,
    })
}
