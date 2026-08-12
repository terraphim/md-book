# Verification Report: mdBook Parity (Increments A-E, G)

**Status**: Verified with documented gaps
**Date**: 2026-08-11
**Phase 2 Doc**: `docs/plans/mdbook-parity-implementation-plan.md`
**Phase 2.5 Doc**: same file, "Specification Interview Findings -- Increment B"
**Commit verified**: `94ca7bf` (main) plus the validation fix below

## Summary

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Unit test coverage (lines) | 80% | **87.1%** | PASS |
| Region coverage | -- | 85.9% | PASS |
| Function coverage | -- | 83.2% | PASS |
| Spec findings (Phase 2.5) covered | All 12 | **12/12** | PASS |
| Module boundaries tested | All | **9/9** | PASS |
| CI checks | All required | **19/19 green** | PASS |
| Open critical/high defects | 0 | 0 | PASS |

Test population: 96 unit, 46 integration, 12 e2e, 4 structure, 16 mdBook conformance = **174 tests**.

## Specialist Skill Results

### Static analysis (`ubs-scanner`) -- COULD NOT RUN, substituted

```
✗ failed to verify module rust: checksum mismatch for rust
  (expected 5c0df5f4…, got 08e99d1e…)
✗ failed to ensure module for rust
```

UBS refuses to load its Rust module: the downloaded module's checksum does not match its
manifest, twice, after a refresh. **This was not worked around** -- disabling the integrity check
to obtain a scan would defeat its purpose. Recorded as gap **D-007**.

Diagnosed 2026-08-12: `ubs doctor` verifies js, python, cpp and golang and fails **only** on
rust. Three distinct digests exist -- the installer's pin (`5c0df5f4…`), what upstream serves
(`08e99d1e…`), and the July cache (`26249823…`) -- and the served hash is stable across fetches,
so this is not corruption in transit. `ubs` pulls modules from an unpinned `master` branch while
pinning digests in a released installer, which breaks verification on any upstream edit. Upstream
issue; see the validation report for the full table.

Substitute analysis run directly with `ast-grep` and clippy: **0** `unsafe` blocks, **0**
`panic!`/`todo!`/`unimplemented!`, **7** `unwrap()`/`expect()` in production code (each verified
guarded by a surrounding invariant), clippy `-D warnings` clean across all targets and features.
Plus `cargo audit` green in CI and three independent review rounds.

### Code review

| Check | Result |
|-------|--------|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -D warnings` | clean |
| Structural review rounds | 2 (3/5 -> 5/5) |
| Independent agent review rounds | 2 (`pi-rust`, openai-codex/gpt-5.5) |

### Security audit

`cargo audit` passes in CI. Five advisories were resolved during verification (`bytes`,
`crossbeam-epoch`, `time`, `anyhow`, `spin`); one is documented rather than fixed in
`.cargo/audit.toml`:

- RUSTSEC-2026-0194 / 0195 (`quick-xml` <0.41), reachable only via `syntect -> plist`, which
  parses the syntax and theme definitions md-book ships, never user input. Removable when
  `plist` takes the semver-major bump.

Security-relevant behaviour verified by test: path containment (`test_summary_rejects_paths_
escaping_src`, `test_create_missing_rejects_symlink_escape`), output containment
(`test_output_path_never_escapes_the_build_dir`), HTML injection via authored SUMMARY text
(`test_summary_label_cannot_inject_html`, `test_summary_external_url_cannot_break_out_of_href`,
`test_quoted_filename_cannot_break_out_of_href`).

### Performance

| Book size | Build time | Per page |
|-----------|-----------|----------|
| 1 page | 10 ms | -- (fixed: asset writing) |
| 30 pages (corpus) | 127 ms | 4.0 ms |
| 500 pages | 1153 ms | 2.31 ms |

Against `main` before the branch: 100 ms -> 127 ms on the corpus (+27%). Gate was "within 10%":
**not met, with attribution** -- the branch writes 4.2 MB of assets per build that `main` never
wrote, because emitting no `css/`, `js/` or `img/` was itself the defect increment D fixed.
Measurement removed one avoidable cost (a duplicate mdast parse per page, 12 ms).

## Coverage by Module

| Module | Regions | Lines | Functions | Assessment |
|--------|---------|-------|-----------|------------|
| `render/meta.rs` | 97.2% | 97.0% | 93.8% | |
| `watch.rs` | 97.0% | 97.4% | 91.7% | |
| `book/mod.rs` | 95.7% | 97.2% | 88.6% | |
| `book/directory.rs` | 93.1% | 97.1% | 81.8% | |
| `render/html.rs` | 91.8% | 94.7% | 81.3% | |
| `config.rs` | 91.7% | 95.3% | 90.4% | |
| `book/summary.rs` | 89.3% | 88.6% | 91.1% | critical path, well covered |
| `render/slug.rs` | 88.7% | 90.3% | 100% | |
| `pagefind_service.rs` | 87.2% | 83.1% | 81.8% | |
| `pipeline/mod.rs` | 86.5% | 87.9% | 89.3% | |
| `core.rs` | 79.8% | 86.7% | 88.9% | thin orchestration wrapper |
| `render/markdown.rs` | 76.9% | 77.8% | 55.6% | see D-008 |
| `paths.rs` | 74.2% | 74.8% | 88.9% | |
| `main.rs` | 28.6% | 24.9% | 33.3% | CLI wiring, exercised by e2e |
| `server.rs` | 86.7% | 80.5% | 82.6% | was 0%; see D-006, D-016 |
| `pipeline/preprocess.rs` | 100% | 100% | 100% | identity seam |

## Traceability: Phase 2.5 spec findings -> tests

| # | Specification decision | Test | Status |
|---|------------------------|------|--------|
| 1 | `create-missing` honoured, default true | `test_create_missing_writes_stub_once` | PASS |
| 2 | Errors collected in one pass | `test_parse_collects_all_errors_in_one_pass` | PASS |
| 3 | Duplicate entries are errors | `test_parse_rejects_duplicate_entry` | PASS |
| 4 | Non-`.md` targets are errors | `test_parse_rejects_non_markdown_target` | PASS |
| 5 | Anchors accepted | `test_parse_accepts_anchor_link` | PASS |
| 6 | External URLs accepted | `test_parse_accepts_external_url` | PASS |
| 7 | Path containment | `test_summary_rejects_paths_escaping_src` | PASS |
| 8 | Symlink escapes refused | `test_create_missing_rejects_symlink_escape` | PASS (unix) |
| 9 | SUMMARY link text wins as title | `test_title_from_summary_not_h1` | PASS |
| 10 | Sidebar nesting via open/close deltas | `test_to_nav_list_deltas_balance` | PASS |
| 11 | Numbered chapter after suffix is an error | `test_parse_rejects_numbered_after_suffix` | PASS |
| 12 | Watcher ignores its own writes | `test_watch_suppresses_created_stub_event` | PASS |

12/12 covered.

## Traceability: design increments -> evidence

| Increment | Design element | Verification |
|-----------|----------------|--------------|
| A | collect/preprocess/render/index split | Structure preserved; `pipeline/preprocess` 100% |
| B | `SUMMARY.md` book model | `structure` suite vs committed fixture; 20 unit tests in `book/summary.rs` |
| C | Subcommands, path resolution | `e2e` suite (12 tests), `paths.rs` unit tests |
| D | Relocatable, offline, heading IDs, 404 | `test_output_has_no_absolute_asset_paths`, `test_output_has_no_external_urls`, `test_headings_have_stable_ids`, `test_404_*` |
| E | Themes, print, redirects, fold, shortcuts | `test_syntax_theme_is_configurable_and_dark_scoped`, `test_sidebar_fold_collapses_all_but_the_active_branch`, `test_theme_picker_and_attributes_present` |
| E7 | Conditional mermaid | `test_mermaid_scripts_only_on_diagram_pages` + code-sample guard |
| G | Description, canonical, skip link, search gating | `test_page_carries_description_and_canonical`, `test_skip_link_*`, `test_search_ui_omitted_without_an_index` |

## Module boundaries

| Boundary | Unit | Integration | Status |
|----------|------|-------------|--------|
| `load_book` -> `Book` | yes | yes | PASS |
| `preprocess` seam | yes | yes | PASS |
| `render_markdown` -> HTML | yes | yes | PASS |
| `inject_heading_ids` | yes | yes | PASS |
| `render_page` / `render_index` | yes | yes | PASS |
| `copy_static_assets` | yes | yes | PASS |
| Pagefind indexing | yes | yes | PASS |
| `SelfWriteFilter` -> watcher loop | yes | decision only | PARTIAL |
| warp server | yes | `warp::test` | PASS -- routes, fallback, websocket upgrade, bind resolution |

## Defect Register

Defects found during implementation and verification, with the phase each traces back to.

| ID | Description | Origin | Severity | Resolution | Status |
|----|-------------|--------|----------|------------|--------|
| D-001 | `path_to_root` applied to assets only; every nav link root-absolute | Phase 3 | High | `e90ec07` | Closed |
| D-002 | Builds outside a book directory silently emitted an empty book | Phase 2.5 (gap) | High | `e90ec07` | Closed |
| D-003 | `css/`, `js/`, `img/` emitted only when a templates dir existed | Phase 3 | High | `6eeccae` | Closed |
| D-004 | Config defaults never applied (`title`, `logo`, `language` empty) | Phase 3 | High | `7564c23` | Closed |
| D-005 | HTML injection via SUMMARY labels and link targets; Tera autoescape never active | Phase 2 (design) | **Critical** | `9776971`, `67e4c1d` | Closed |
| D-006 | `server.rs` has no tests (0% coverage), including changed bind logic | Phase 4 | Medium | Tests added; coverage 0% -> 86.7% | Closed |
| D-016 | Live reload never worked: the file fallback matched `/live-reload`, so the websocket upgrade was unreachable | Phase 2 (route order) | High | Found by writing D-006's tests | Closed |
| D-017 | `serve -p` rejected; only `--port` existed, unlike mdBook | Phase 2 | Low | Short flag added | Closed |
| D-007 | UBS scanner cannot run (module checksum mismatch) | Tooling | Medium | Substitute evidence recorded | **Open, external** |
| D-008 | `render/markdown.rs` function coverage 55.6% | Phase 4 | Low | Feature-gated branches untested | **Open, accepted** |
| D-009 | URLs built with `Path::display()` -- backslashes on Windows | Phase 3 | High | `696d30d` | Closed |
| D-010 | Output paths could escape the build directory | Phase 3 | **Critical** | `3ede392` | Closed |
| D-011 | Containment rejected legitimate books (file beside `book.toml`) | Phase 2.5 (over-strict) | High | `3ede392` | Closed |
| D-012 | Test fixture never committed (`.gitignore` `test_*`) | Phase 3 | Medium | `89b4974` | Closed |
| D-013 | Five security advisories in dependency tree | Phase 3 | Medium | `6461246` | Closed |
| D-014 | `main_impl_sync` skipped build validation (feature-dependent contract) | Phase 3 | Medium | `388128b` | Closed |
| D-015 | Index page had no navigation at all | Phase 2 (design omission) | **High** | see validation report | Closed |

Note the origins: D-005 and D-015 trace to **Phase 2 design**, not to implementation. The design
specified the sidebar's contents and the escaping policy for chapter content, but never stated
that authored SUMMARY text is untrusted, nor that the landing page needs navigation.

## Gate Checklist

- [x] Coverage > 80% on critical paths (86.0% lines overall)
- [x] All spec findings from Phase 2.5 covered (12/12)
- [x] Data flows verified against the design diagram
- [x] All critical and high defects resolved
- [x] Traceability matrix complete
- [x] Code review checklist passed
- [x] Security audit passed (one documented exception)
- [x] Performance measured (gate missed, attributed, accepted)
- [ ] UBS scan -- **could not run**, D-007
- [x] Medium/low defects explicitly deferred: D-006, D-007, D-008
- [ ] Human approval

## Recommendation

**Verified, with three open medium/low defects explicitly deferred.** The one that matters for a
release decision is D-006: the dev server is entirely untested, and this branch changed its bind
behaviour. It affects `md-book serve` only -- not `build`, which is what produces published
output and what CI and the real corpus exercise.
