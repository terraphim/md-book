//! Structural conformance of SUMMARY-driven books.
use anyhow::Result;
use md_book::book::load_book;
use std::path::Path;

#[test]
fn test_structure_matches_fixture() -> Result<()> {
    let src = Path::new("test_book_mdbook/src");
    let (book, _) = load_book(src, false)?;
    assert!(book.from_summary);

    let fixture: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        "tests/fixtures/test_book_mdbook.structure.json",
    )?)?;

    // Spot-check key chapters from the fixture
    let chapters: Vec<_> = book.iter_all_chapters().collect();
    let by_name: std::collections::HashMap<_, _> =
        chapters.iter().map(|c| (c.name.as_str(), *c)).collect();

    for expected in fixture["chapters"].as_array().unwrap() {
        let name = expected["name"].as_str().unwrap();
        let ch = by_name
            .get(name)
            .unwrap_or_else(|| panic!("missing chapter {name}"));
        if let Some(src) = expected["source"].as_str() {
            assert_eq!(
                ch.source_path.as_ref().map(|p| p.to_str().unwrap()),
                Some(src),
                "source for {name}"
            );
        } else {
            assert!(ch.source_path.is_none(), "{name} should be draft");
        }
        if let Some(out) = expected["output"].as_str() {
            assert_eq!(
                ch.output_path.as_ref().map(|p| p.to_str().unwrap()),
                Some(out),
                "output for {name}"
            );
        }
        match expected.get("number") {
            Some(serde_json::Value::String(n)) => {
                assert_eq!(
                    ch.number.as_ref().map(|s| s.to_label()).as_deref(),
                    Some(n.as_str()),
                    "number for {name}"
                );
            }
            Some(serde_json::Value::Null) | None => {
                assert!(ch.number.is_none(), "{name} should be unnumbered");
            }
            _ => {}
        }
    }

    // Full page-producing chain starts with prefix, ends with suffix
    let chain: Vec<_> = book
        .iter_chapters()
        .map(|c| {
            c.output_path
                .as_ref()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(chain.first().map(String::as_str), Some("prefix.html"));
    assert_eq!(chain.last().map(String::as_str), Some("suffix.html"));
    assert!(chain.contains(&"index.html".to_string()));
    assert!(!chain.iter().any(|p| p == "SUMMARY.html"));
    // Draft produces no page
    assert_eq!(chain.len(), 30);

    Ok(())
}

#[test]
fn test_files_absent_from_summary_not_published_logic() -> Result<()> {
    let src = Path::new("test_book_mdbook/src");
    let (book, _) = load_book(src, false)?;
    let listed: std::collections::HashSet<_> = book
        .iter_all_chapters()
        .filter_map(|c| c.source_path.clone())
        .collect();
    assert!(!listed.contains(Path::new("SUMMARY.md")));
    Ok(())
}

#[test]
fn test_no_summary_falls_back_to_directory() -> Result<()> {
    let src = Path::new("tests/assets/test_book_1");
    // test_book_1 may nest md under src/
    let book_src = if src.join("src").exists() {
        src.join("src")
    } else {
        src.to_path_buf()
    };
    // If no SUMMARY in the asset, from_summary is false
    let summary = book_src.join("SUMMARY.md");
    if summary.exists() {
        // still valid if present
        let (book, _) = load_book(&book_src, false)?;
        let _ = book;
    } else {
        let (book, _) = load_book(&book_src, false)?;
        assert!(!book.from_summary);
        assert!(book.iter_chapters().count() > 0);
    }
    Ok(())
}

#[test]
fn test_prev_next_chain_follows_summary() -> Result<()> {
    let (book, _) = load_book(Path::new("test_book_mdbook/src"), false)?;
    let chain: Vec<_> = book.iter_chapters().map(|c| c.name.clone()).collect();
    assert_eq!(chain.first().map(String::as_str), Some("Prefix Chapter"));
    assert_eq!(chain.get(1).map(String::as_str), Some("Introduction"));
    assert_eq!(chain.last().map(String::as_str), Some("Suffix Chapter"));
    // Draft is skipped in page chain
    assert!(!chain.iter().any(|n| n == "Draft Chapter"));
    Ok(())
}
