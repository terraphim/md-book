# Traceability Report: Theme Token Parity

Requirement IDs below are scoped identifiers for the approved product
specification; they do not replace existing project-wide IDs.

| Req ID | Requirement | Maturity | Plan/spec | Implementation | Tests/evidence | Status |
|---|---|---|---|---|---|---|
| REQ-THEME-01 | Offline Shoelace default and isolated opt-in Web Awesome family | Verified | `docs/plans/design-theme-token-parity.md`; `docs/specs/product/theme-token-parity.spec.md` | `src/render/html.rs`; `src/templates/webawesome/`; `book.webawesome.toml` | `test_webawesome_theme_is_isolated_and_explicitly_online_only`; Chrome family counts | PASS |
| REQ-THEME-02 | Five explicit themes apply coherent colors to every visible surface | Verified | Same | both `css/themes.css` files; shadow components | `test_theme_tokens_reach_shadow_components_and_search_paths_are_relative`; Chrome surface matrix | PASS |
| REQ-LAYOUT-01 | Preserve wide prose multi-columns and remove mobile overflow | Verified | Same | both `css/styles.css` files | Chrome 1600×1000 reports `column-width: 403.125px`; 390×844 reports `scrollWidth == clientWidth` | PASS |
| REQ-SEARCH-01 | Successful first build contains Pagefind index and UI | Verified | Same | `src/core.rs`; `src/pipeline/mod.rs` | `test_search_ui_matches_pagefind_index_after_first_build`; real 30-page builds | PASS |
| REQ-SEARCH-02 | Failed index publishes no search UI and reports failure | Verified | Same | `pipeline::index`; `discard_search_index`; second render | conditional failure branch in first-build test; `test_failed_index_cleanup_discards_stale_pagefind_assets`; explicit stderr strings | PASS |
| REQ-SEARCH-03 | Search works at root, prefixes, and nested pages | Verified | Same | relative module/Pagefind URLs | relative-path integration assertions; Chrome search on `/individual/paragraph.html` | PASS |
| REQ-SEARCH-04 | Instant 150 ms debounce; stale responses cannot win | Verified | Same | `PagefindSearch.searchGeneration`; `pagefind.debouncedSearch` | integration source assertions; Chrome warm update 221 ms total | PASS |
| REQ-A11Y-01 | Keyboard-open, navigate, close, focus and announced state | Verified | Same | both search modal components | Chrome `/`, ArrowDown, Escape; `aria-selected=true`, named dialog/listbox | PASS |
| REQ-SEC-01 | Search metadata cannot inject title/URL HTML | Verified | Same | DOM-created title/URL nodes; only Pagefind encoded excerpt uses HTML | code review; UBS 0 critical; Chrome result rendering | PASS |
| REQ-DOC-01 | Explain family selection, fallback, failure mode, and breaking tokens | Verified | Same | `README.md`; `CHANGELOG.md` | documentation review | PASS |

## Gaps

- No blocking traceability gaps.
- UBS Rust scanning could not run because the installed UBS module manifest and
  downloaded Rust module had different checksums. The integrity check was not
  bypassed; Clippy and the full Rust suite provide the Rust static/test gates.
- Persistent screenshot baselines are not introduced because the repository has
  no existing pixel-diff runner. Chrome screenshots were used as UAT evidence;
  a future CI visual-baseline job is non-blocking.
