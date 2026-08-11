# Implementation Plan: mdBook Feature Parity

**Status**: Approved (2026-08-08)
**Research Doc**: `docs/plans/mdbook-parity-research.md`
**Author**: Claude (Opus 5), with Alex Mikhalev
**Date**: 2026-08-08
**Estimated Effort**: 11-15 days across five increments
**Phase**: 2 (disciplined-design)

## Overview

### Summary

Bring md-book to *contract parity* with mdBook: any valid mdBook book builds correctly, rendered
through md-book's own stack. Five increments, each independently shippable:

| # | Increment | Effort | Ships |
|---|-----------|--------|-------|
| A | Pipeline decomposition | 1-1.5 d | No behaviour change; unblocks everything else |
| B | `SUMMARY.md` book model (P0) | 4-5 d | Correct structure, ordering, navigation |
| C | CLI subcommands + book-directory resolution | 1.5-2 d | `md-book build ./mybook`, `book.src`, `build.build-dir` |
| D | Output-correctness defects (P1) | 2-2.5 d | Relocatable, offline, anchorable output |
| E | Renderer polish (P3) | 2-3 d | Themes, print, redirects, fold, shortcuts, conditional mermaid |
| G | Page metadata and skip link | 0.5-1 d | Description, canonical URL, skip link, search gating (recovers `stash@{0}`) |

The `pulldown-cmark` backend (approved as a feature-flagged second parser) is designed here as
**increment F** and specified at interface level only; it is sequenced after E and gets its own
implementation cycle.

### Approach

Interpretation B from the research doc: parity of the authored book contract, not of mdBook's
internals. Introduce an explicit `Book` model between input and rendering, so that structure
becomes data rather than a side effect of directory traversal.

The current pipeline is one 250-line function (`core.rs:129-386`) that interleaves collection,
rendering and indexing. Increment A splits it into four named stages; every later increment then
targets exactly one stage.

### Scope

**In scope:**

- Decompose `build_sync_impl_sync` into `collect` / `preprocess` / `render` / `index`.
- `SUMMARY.md` parser: prefix/suffix chapters, part titles, nested numbered chapters, draft
  chapters, separators, section numbering, summary-driven file exclusion.
- `README.md` → `index.html` mapping.
- Chapter-tree navigation, correct previous/next, nested sidebar.
- Directory-walk fallback when no `SUMMARY.md` exists (no existing book breaks).
- `md-book build|serve|watch|init|clean [dir]` subcommands, with `-i/-o` retained as overrides.
- `book.src`, `build.build-dir`, `build.extra-watch-dirs`.
- Relative asset paths (`path_to_root`), vendored Shoelace, server-side heading IDs, 404 page,
  copy button wired up.
- Themes (light/dark/ayu/coal/navy toggle), configurable syntect theme, print page,
  `additional-css` / `additional-js`, `[output.html.redirect]`, `[output.html.fold]`, keyboard
  shortcuts, and loading mermaid only on pages that contain a diagram (E7).
- Warnings for parsed-but-unsupported config keys (`config::unsupported_keys_in`), reported per
  file and only for keys the author actually set.
- Per-page `<meta name="description">`, `<link rel="canonical">`, a skip link, and search UI
  gated on a Pagefind index actually existing (increment G).
- Structural conformance fixtures over `test_book_mdbook/`.
- Interface-level design for the `pulldown-cmark` backend (increment F).

**Out of scope (deferred to their own cycles):**

- **P2 preprocessing** -- `{{#include}}` (+ line ranges, anchors), `{{#rustdoc_include}}`,
  `{{#playground}}`, `{{#title}}`, hidden lines, Rust code-block attributes. Increment A leaves
  a designated seam for it. **Consequence to state plainly: after this plan, an mdBook book
  containing `{{#include}}` still renders the directive as literal text.**
- Implementation of the `pulldown-cmark` backend (designed here, built later).
- `mdbook test` (rustdoc doctests).
- Preprocessor / alternative-backend plugin protocol.

**Avoid at all cost** (5/25 -- these threaten the essential):

- Handlebars theme compatibility (`index.hbs`, `head.hbs`, `{{#toc}}`, `{{fa}}`).
- elasticlunr, or emulating its scoring knobs (`boost-*`, `expand`, `use-boolean-and`,
  `teaser-word-count`) on top of Pagefind.
- Rust Playground runtime (run button, live editor).
- `hash-files`, `cname`, `text-direction`, shell `completions`.
- Byte-for-byte HTML matching with mdBook, or porting mdBook's CSS.
- A generic "renderer" trait abstraction with one implementation.
- Making Pagefind a hard build dependency.

## Architecture

### Component diagram

```
                            ┌──────────────────────────────┐
  book.toml ───────────────►│ config::resolve_book_paths   │  (increment C)
  CLI (subcommand + flags)  │  root, src_dir, build_dir    │
                            └──────────────┬───────────────┘
                                           ▼
   src/SUMMARY.md ────►┌───────────────────────────────────┐
                       │ summary::parse  (increment B)     │
                       │   -> Summary { prefix, numbered,  │
                       │                suffix }           │
                       └──────────────┬────────────────────┘
                                      │  absent? -> book::from_directory (fallback, today's walk)
                                      ▼
                       ┌───────────────────────────────────┐
                       │ book::Book  { items: Vec<BookItem> }   ← the model everything reads
                       └──────────────┬────────────────────┘
                                      ▼
                       ┌───────────────────────────────────┐
                       │ preprocess  (increment A: seam)   │  markdown text in -> out
                       │   today: identity                 │  P2 directives land here
                       └──────────────┬────────────────────┘
                                      ▼
                       ┌───────────────────────────────────┐
                       │ render::markdown  (F: 2 backends) │
                       │   mdast splice + syntect          │
                       │   + heading IDs   (increment D)   │
                       └──────────────┬────────────────────┘
                                      ▼
                       ┌───────────────────────────────────┐
                       │ render::html   Tera               │
                       │   nav tree, path_to_root, themes  │  (B, D, E)
                       └──────────────┬────────────────────┘
                                      ▼
                       ┌───────────────────────────────────┐
                       │ index  Pagefind (unchanged)       │
                       └───────────────────────────────────┘
```

### Data flow

```
resolve paths -> load config -> build Book (SUMMARY or directory walk)
  -> for each chapter: read md -> preprocess -> parse+highlight -> heading IDs -> link rewrite
  -> render page (nav tree + path_to_root + prev/next from book order)
  -> render index / print / 404 / redirects -> copy assets -> pagefind index
```

### Key design decisions

| Decision | Rationale | Alternatives rejected |
|----------|-----------|----------------------|
| Introduce an explicit `Book`/`BookItem` model | Structure becomes data; navigation, numbering, prev/next and exclusion all read one source | Keep deriving structure inside the render loop (today) -- the cause of every A-row gap |
| `SUMMARY.md` presence *selects* the mode; directory walk is the fallback | No existing book breaks; migration is opt-in by adding a file | Mandatory `SUMMARY.md` (breaks Terraphim docs and `dist/`); config flag (extra knob for a decidable condition) |
| Keep flat `sections` in the Tera context alongside the new `chapters` tree for one release | User-supplied templates (`paths.templates`) keep working | Hard cutover -- silently breaks overridden sidebars |
| Pre-flatten the chapter tree into a depth-tagged list for Tera | Tera has no template recursion; a flat list with `depth` renders nested markup with one loop | Tera macro recursion (fragile, poor errors); render nav in Rust (moves markup out of templates) |
| Subcommands added, `-i/-o` retained as overrides | mdBook users get the familiar entry point; CI, Dockerfile and `scripts/deploy.sh` stay green | Full CLI switch (breaks this repo's workflows in one go); `-i/-o` only (`book.src` unsupportable) |
| `path_to_root` computed per page; all template asset URLs become relative | Fixes sub-path deployment; matches mdBook | Configurable `site-url` prefix only -- still absolute, still breaks local `file://` viewing |
| Heading IDs injected via mdast post-processing, GitHub-compatible slugs | Reachable without a parser swap; fixes cross-page fragments and Pagefind anchors | Client-side only (today) -- invisible to crawlers, static consumers and deep links |
| Second parser as an additive feature (`parser-cmark`), `markdown` stays default | Retains MDX and today's behaviour; group C becomes reachable for those who opt in | Swap wholesale (loses MDX); do nothing (group C unreachable) |
| Unsupported config keys warn once at load | Silent acceptance is the current user-hostile failure shape (`book.toml` carries `# not implemented` comments) | Hard error (breaks existing `book.toml` files); stay silent (status quo) |

### Eliminated options (essentialism)

| Option rejected | Why rejected | Risk of including |
|-----------------|--------------|-------------------|
| Renderer trait with a single HTML impl | Speculative abstraction; no second renderer is planned | Indirection with no payoff; harder to follow |
| Preprocessor plugin protocol (`mdbook-*` over stdio) | Premature before an internal preprocessing pipeline exists | Process spawning, JSON schema and version negotiation for zero current users |
| Porting mdBook's CSS/themes verbatim | Contradicts the Terraphim design decision | Two design systems in one repo |
| Emulating elasticlunr scoring on Pagefind | Pagefind has no equivalent knobs | Config that appears to work and does not |
| Incremental/cached rebuilds | Not a stated problem; builds are fast on the corpus | Cache-invalidation bugs, non-deterministic output |

### Simplicity check

**What if this could be easy?** The whole of increment B is one parser plus one flatten
function. `SUMMARY.md` is a markdown list; parse it into `Vec<BookItem>`; number it by walking
the tree; flatten it once for the sidebar and once for prev/next. Everything else in this plan
is either deleting a hard-coded value (syntect theme, `"Guide"` section title, absolute asset
paths) or wiring an existing file into a template (`code-copy.js`).

Complexity is admitted in exactly two places, both justified:

1. The dual-parser feature flag (increment F) -- a directly approved decision, and additive.
2. Keeping `sections` and `chapters` in the template context simultaneously -- a one-release
   deprecation window, removed in the following release.

**Senior engineer test**: the design adds ~4 focused modules and deletes structure-inference
logic from `core.rs`. Net conceptual weight goes down, not up.

**Nothing speculative checklist**:
- [x] No features the user did not request (P2 explicitly deferred, not smuggled in)
- [x] No abstractions "in case we need them later" (no renderer trait, no plugin protocol)
- [x] No flexibility "just in case" (the preprocess seam is an identity function, not a registry)
- [x] No error handling for impossible scenarios
- [x] No premature optimization (benchmark first, per increment A gate)

## File changes

### New files

| File | Purpose | Increment |
|------|---------|-----------|
| `src/book/mod.rs` | `Book`, `BookItem`, `Chapter`, `SectionNumber`; tree walk, flatten, numbering | B |
| `src/book/summary.rs` | `SUMMARY.md` parser and its errors | B |
| `src/book/directory.rs` | Today's directory-walk fallback, moved out of `core.rs` | B |
| `src/pipeline/mod.rs` | Stage sequencing: `collect` → `preprocess` → `render` → `index` | A |
| `src/pipeline/preprocess.rs` | Identity pass today; the designated seam for P2 | A |
| `src/render/html.rs` | Tera context assembly, `path_to_root`, page/index/print/404/redirect writers | A, D, E |
| `src/render/markdown.rs` | mdast splice + syntect + heading-ID injection; parser-backend switch | A, D, F |
| `src/render/slug.rs` | GitHub-compatible heading slugs with per-page collision counters | D |
| `src/paths.rs` | Book-directory / `src` / `build-dir` resolution and precedence | C |
| `src/templates/css/themes.css` | CSS custom properties per theme | E |
| `src/templates/js/theme-switch.js` | Theme toggle + `localStorage` persistence | E |
| `src/templates/js/keyboard.js` | `←`/`→`/`s`/`/`/`?` shortcuts | E |
| `src/templates/print.html.tera` | Single-document print page | E |
| `src/templates/404.html.tera` | 404 page | D |
| `src/templates/vendor/shoelace/*` | Vendored Shoelace subset: 5 components' import closure, 10 icons, light/dark themes (356KB) | D |
| `src/templates/vendor/shoelace/shoelace-local.js` | Local loader; `setBasePath` from `import.meta.url` | D |
| `src/watch.rs` | `SelfWriteFilter`: drops watcher events caused by md-book's own writes | D |
| `tests/integration/summary_test.rs` | Summary parser unit/integration coverage | B |
| `tests/integration/structure_test.rs` | Structural conformance over `test_book_mdbook/` | B |
| `tests/fixtures/test_book_mdbook.structure.json` | Expected chapter tree, numbers, prev/next chain | B |

### Modified files

| File | Changes | Increment |
|------|---------|-----------|
| `src/core.rs` | Shrinks to `Args`, `build()` orchestration; collection/render/nav logic moves out; `"Guide"` section and path-order prev/next deleted | A, B |
| `src/lib.rs` | `pub mod book; pub mod pipeline; pub mod render; pub mod paths;` + re-exports | A, B, C |
| `src/main.rs` | `clap` subcommand dispatch; watch list from `build.extra-watch-dirs` | C |
| `src/config.rs` | `book.src`, `build.*`, `output.html` additions (`default-theme`, `preferred-dark-theme`, `additional-css/js`, `input-404`, `site-url`, `no-section-label`, `[fold]`, `[print]`, `[redirect]`, `syntax-theme`); `book.toml` resolved from the book root; `warn_unsupported_keys()` | C, D, E |
| `src/templates/page.html.tera` | `path_to_root` on every URL; vendored Shoelace; theme + keyboard + copy scripts; `additional-*` injection | D, E |
| `src/templates/index.html.tera` | Same treatment | D, E |
| `src/templates/sidebar.html.tera` | Renders the depth-tagged chapter list: part titles, separators, section numbers, draft chapters, fold | B, E |
| `src/templates/header.html.tera` | Theme toggle control; print link | E |
| `src/templates/js/code-copy.js` | Referenced from templates (currently dead code) | D |
| `src/server.rs` | `-n/--hostname` bind address | C |
| `tests/integration/mdbook_compatibility.rs` | Substring assertions replaced by structural ones | B |
| `book.toml` | `# not implemented` comments removed as each key becomes real | B-E |
| `README.md`, `CHANGELOG.md`, `.claude/skills/md-book` | Document subcommands, `SUMMARY.md`, themes; state P2 limits | all |

### Deleted files

| File | Reason |
|------|--------|
| none | Logic moves rather than disappears; `core.rs` shrinks in place |

## API design

### Public types (increment B)

```rust
// src/book/mod.rs

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
pub enum BookItem {
    Chapter(Chapter),
    /// `# Part Name` -- an unclickable heading in the sidebar.
    PartTitle(String),
    /// `---` -- a horizontal rule in the sidebar.
    Separator,
}

#[derive(Debug, Clone, Serialize)]
pub struct Chapter {
    /// Link text from SUMMARY.md, or the first H1, or the file stem.
    pub name: String,
    /// Source path relative to the book's `src` dir. `None` for draft chapters.
    pub source_path: Option<PathBuf>,
    /// Output path relative to the build dir, e.g. `individual/heading.html`.
    /// `None` for draft chapters (rendered as a disabled sidebar entry).
    pub output_path: Option<PathBuf>,
    /// `1.2.3`; `None` for prefix, suffix and draft chapters.
    pub number: Option<SectionNumber>,
    /// Nested sub-chapters, in authored order.
    pub sub_items: Vec<BookItem>,
}

/// Dotted section number, e.g. `SectionNumber(vec![1, 2, 3])` -> "1.2.3".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SectionNumber(pub Vec<u32>);

/// One entry of the pre-flattened sidebar list handed to Tera.
///
/// Tera macros cannot recurse, so the tree is flattened and nesting is
/// reconstructed by the template from the `open_lists` / `close_lists` deltas.
/// This yields genuinely nested `<ul>` markup (required by the accessibility
/// decision in the interview findings) from a single template loop.
#[derive(Debug, Clone, Serialize)]
pub struct NavEntry {
    pub kind: NavKind,          // Chapter | PartTitle | Separator
    /// Sidebar label: the SUMMARY link text, inline markdown rendered.
    pub title_html: String,
    /// Same label flattened to plain text, for `<title>` and `aria-label`.
    pub title_text: String,
    /// Href relative to the current page; `None` for drafts, part titles and
    /// separators. External SUMMARY links carry their absolute URL here.
    pub href: Option<String>,
    /// True when `href` points off-site (rendered with `rel="external"`).
    pub is_external: bool,
    /// Rendered section number, e.g. "1.2."; empty when `no-section-label` is set.
    pub number: String,
    pub depth: usize,
    /// `<ul>` elements to open before emitting this entry (0 or 1).
    pub open_lists: usize,
    /// `</ul></li>` pairs to close before emitting this entry.
    pub close_lists: usize,
    pub is_draft: bool,
    pub is_active: bool,
}
```

### Public functions

```rust
// src/book/summary.rs

/// Parse the contents of a `SUMMARY.md`.
///
/// Parsing does not stop at the first problem: the whole file is scanned and
/// every error collected, so one build round-trip reports everything wrong.
///
/// # Errors
/// Returns every problem found, each carrying a line number and the offending
/// text: `MixedDelimiters`, `PrefixAfterNumbered`, `Malformed`,
/// `DuplicateEntry`, `NonMarkdownTarget`.
pub fn parse_summary(content: &str) -> Result<Summary, SummaryErrors>;

/// Build a `Book` from a summary, resolving paths against `src_dir` and
/// assigning section numbers.
///
/// Path resolution is containment-checked: every chapter path is canonicalised
/// (resolving symlinks) and must remain under `src_dir`.
///
/// When `create_missing` is true (the default), a referenced file that does not
/// exist is created as a stub containing a single H1 taken from the SUMMARY link
/// text; an existing file is never overwritten. Created paths are returned so the
/// watcher can suppress the resulting filesystem events.
///
/// # Errors
/// `EscapesSourceDir` for any path resolving outside `src_dir`; `MissingFile`
/// when the file is absent and `create_missing` is false.
pub fn book_from_summary(
    summary: &Summary,
    src_dir: &Path,
    create_missing: bool,
) -> Result<(Book, Vec<PathBuf>), SummaryErrors>;

// src/book/directory.rs

/// Today's behaviour: walk `src_dir`, sort by path, group by parent directory.
/// Used when no `SUMMARY.md` exists.
pub fn book_from_directory(src_dir: &Path) -> Result<Book>;

// src/book/mod.rs

impl Book {
    /// Depth-first iteration over chapters in authored order -- the basis of
    /// previous/next and of the Pagefind ordering.
    pub fn iter_chapters(&self) -> impl Iterator<Item = &Chapter>;

    /// Flatten to a sidebar list for Tera, marking `active_path` as active.
    pub fn to_nav(&self, active_path: &Path, no_section_label: bool) -> Vec<NavEntry>;
}

// src/paths.rs (increment C)

/// Resolved book locations. Precedence: CLI flag > book.toml > default.
#[derive(Debug, Clone)]
pub struct BookPaths { pub root: PathBuf, pub src: PathBuf, pub build: PathBuf }

pub fn resolve(
    book_dir: Option<&Path>,
    input_override: Option<&str>,
    output_override: Option<&str>,
    config: &BookConfig,
) -> Result<BookPaths>;

// src/render/slug.rs (increment D)

/// GitHub-compatible slug: lowercase, non-alphanumerics to `-`, collisions
/// suffixed `-1`, `-2`, … via the caller-held counter.
pub fn slugify(text: &str, seen: &mut HashMap<String, usize>) -> String;

// src/render/html.rs (increment D)

/// `../`-style prefix from a page to the build-dir root; `""` at the root.
pub fn path_to_root(page: &Path) -> String;
```

### Error types

```rust
// src/book/summary.rs
#[derive(Debug, thiserror::Error)]
pub enum SummaryError {
    #[error("SUMMARY.md line {line}: cannot mix '-' and '*' list delimiters")]
    MixedDelimiters { line: usize },

    #[error("SUMMARY.md line {line}: prefix chapter '{title}' appears after numbered chapters")]
    PrefixAfterNumbered { line: usize, title: String },

    #[error("SUMMARY.md line {line}: expected a markdown link, found: {text}")]
    Malformed { line: usize, text: String },

    #[error("SUMMARY.md line {line}: '{path}' is already listed (line {first_line})")]
    DuplicateEntry { line: usize, path: PathBuf, first_line: usize },

    #[error("SUMMARY.md line {line}: chapter target must be a .md file, found: {path}")]
    NonMarkdownTarget { line: usize, path: PathBuf },

    #[error("SUMMARY.md line {line}: '{path}' resolves outside the source directory; \
             refusing to read")]
    EscapesSourceDir { line: usize, path: PathBuf },

    #[error("SUMMARY.md references missing file: {path}")]
    MissingFile { path: PathBuf },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Every problem found in one pass, reported together before aborting.
#[derive(Debug, thiserror::Error)]
#[error("SUMMARY.md has {} problem(s)", .0.len())]
pub struct SummaryErrors(pub Vec<SummaryError>);
```

## Test strategy

No mocks (project rule). Every test builds real books from real fixtures.

### Unit tests

| Test | Location | Purpose |
|------|----------|---------|
| `test_parse_prefix_suffix_chapters` | `book/summary.rs` | Prefix before, suffix after numbered |
| `test_parse_nested_chapters_three_deep` | `book/summary.rs` | Arbitrary nesting |
| `test_parse_part_titles_and_separators` | `book/summary.rs` | `# Part` and `---` become items |
| `test_parse_draft_chapter` | `book/summary.rs` | `- [Title]()` → `source_path: None` |
| `test_parse_rejects_mixed_delimiters` | `book/summary.rs` | `MixedDelimiters` |
| `test_parse_rejects_prefix_after_numbered` | `book/summary.rs` | `PrefixAfterNumbered` |
| `test_parse_collects_all_errors_in_one_pass` | `book/summary.rs` | Three seeded problems → three errors, in line order |
| `test_parse_rejects_duplicate_entry` | `book/summary.rs` | `DuplicateEntry` names both lines |
| `test_parse_rejects_non_markdown_target` | `book/summary.rs` | `NonMarkdownTarget` |
| `test_parse_accepts_anchor_link` | `book/summary.rs` | Fragment stripped for resolution, kept on href |
| `test_parse_accepts_external_url` | `book/summary.rs` | `is_external`, no output path, no chain slot |
| `test_summary_rejects_paths_escaping_src` | `book/summary.rs` | `../`, absolute, and a symlink out of `src/` |
| `test_create_missing_writes_stub_once` | `book/summary.rs` | Stub H1 from link text; existing file never overwritten; created paths returned |
| `test_to_nav_list_deltas_balance` | `book/mod.rs` | Σ`open_lists` == Σ`close_lists` for any tree |
| `test_title_from_summary_not_h1` | `book/mod.rs` | Link text wins; markdown rendered for HTML, flattened for `<title>` |
| `test_section_numbering_matches_nesting` | `book/mod.rs` | `1`, `1.1`, `1.1.1`, `2` |
| `test_prefix_suffix_draft_are_unnumbered` | `book/mod.rs` | `number: None` |
| `test_readme_maps_to_index_html` | `book/mod.rs` | `README.md` → `index.html` |
| `test_iter_chapters_authored_order` | `book/mod.rs` | Order ≠ path sort |
| `test_to_nav_depth_and_active` | `book/mod.rs` | Depth tags, single active entry |
| `test_path_to_root_depths` | `render/html.rs` | `""`, `"../"`, `"../../"` |
| `test_slugify_matches_github` | `render/slug.rs` | Punctuation, case, unicode |
| `test_slugify_collisions_suffixed` | `render/slug.rs` | `-1`, `-2` |
| `test_resolve_precedence_cli_over_toml` | `paths.rs` | `-i` beats `book.src` |
| `test_warn_unsupported_keys` | `config.rs` | Every rejected key warns once |

### Integration tests

| Test | Location | Purpose |
|------|----------|---------|
| `test_structure_matches_fixture` | `tests/integration/structure_test.rs` | Built tree vs `test_book_mdbook.structure.json` |
| `test_files_absent_from_summary_not_published` | same | Summary-driven exclusion; orphan `.md` warns |
| `test_non_markdown_assets_copied_through` | same | `src/img/*.png` reaches the build dir at the same relative path |
| `test_watch_suppresses_created_stub_event` | same | `create-missing` under `--watch` does not trigger a second rebuild |
| `test_sidebar_nesting_and_aria` | same | Nested `<ul>`, `aria-current="page"`, `aria-disabled` drafts, part-title `<h2>` |
| `test_prev_next_chain_follows_summary` | same | Walk the whole chain |
| `test_no_summary_falls_back_to_directory` | same | `tests/assets/test_book_1` unchanged |
| `test_existing_templates_still_render` | same | Flat `sections` still populated |
| `test_output_has_no_absolute_asset_paths` | `tests/integration/build_test.rs` | No `href="/…"`/`src="/…"` |
| `test_output_has_no_external_urls` | same | Offline output (Shoelace vendored) |
| `test_headings_have_stable_ids` | same | Server-side `id` on every `<h1-6>` |
| `test_404_and_redirects_written` | same | `404.html`, redirect stubs |
| `test_print_page_contains_all_chapters` | same | `print.html` completeness |
| `test_theme_toggle_present_and_persists` | `tests/frontend.test.js` | Browser-level theme check |
| `test_build_subcommand_uses_book_toml_paths` | `tests/e2e/cli_test.rs` | `md-book build ./dir` |
| `test_legacy_io_flags_still_work` | same | `-i/-o` regression |

### Fixture strategy

`tests/fixtures/test_book_mdbook.structure.json` is generated once from
`test_book_mdbook/src/SUMMARY.md`, reviewed by hand against mdBook's own rendering, then
committed. Structural assertions compare parsed trees, never HTML substrings -- replacing the
false confidence in `tests/integration/mdbook_compatibility.rs:38-47`.

## Implementation steps

### Increment A -- pipeline decomposition (1-1.5 d)

**A1. Extract stages** — `src/pipeline/mod.rs`, `src/render/html.rs`, `src/render/markdown.rs`,
`src/core.rs`. Move code with zero behaviour change; `core::build` becomes orchestration.
*Tests:* the existing suite must pass untouched. *Est:* 6 h.

**A2. Add the preprocess seam** — `src/pipeline/preprocess.rs`: `fn preprocess(md: &str, ctx:
&PreprocessCtx) -> Result<String>`, identity today, called before mdast parsing.
*Tests:* `test_preprocess_identity_preserves_input`. *Est:* 1 h.

**A3. Baseline benchmark** — record `benches/pagefind_bench.rs` numbers on `test_book_mdbook`
before any later increment. *Est:* 1 h.

**Gate:** `make ci-local` green, output byte-identical to pre-A build.

### Increment B -- `SUMMARY.md` book model (4-5 d) [P0]

**B1. Structure fixture first** — build `tests/fixtures/test_book_mdbook.structure.json` and the
failing `structure_test.rs`. *Est:* 3 h.

**B2. Summary parser** — `src/book/summary.rs` with the full unit table above. *Deps:* B1.
*Est:* 8 h.

**B3. Book model + numbering + flatten** — `src/book/mod.rs`: `Book`, `BookItem`, `Chapter`,
`SectionNumber`, `iter_chapters`, `to_nav`; `README.md` → `index.html`. *Deps:* B2. *Est:* 8 h.

**B4. Directory fallback** — move today's walk to `src/book/directory.rs`; select on
`SUMMARY.md` presence; delete the `"Guide"` literal and the path-order prev/next
(`core.rs:216-219`, `core.rs:274-284`). *Deps:* B3. *Est:* 4 h.

**B5. Sidebar template** — nested rendering from `NavEntry`: part titles, separators, section
numbers, disabled draft entries. Keep `sections` populated for one release. *Deps:* B4.
*Est:* 5 h.

**B6. Conformance** — rewrite `mdbook_compatibility.rs` onto structural assertions; make B1 pass.
*Deps:* B5. *Est:* 4 h.

**Gate:** `test_book_mdbook` structure matches the fixture exactly; `tests/assets/test_book_1`
(no summary) renders as before.

### Increment C -- CLI and path resolution (1.5-2 d)

**C1. `src/paths.rs`** — precedence CLI > `book.toml` > default. *Est:* 3 h.
**C2. Subcommands** — `build|serve|watch|init|clean [dir]`, `-d/--dest-dir`, `-n/--hostname`,
`--open`; no-subcommand invocation keeps today's `-i/-o` behaviour. *Deps:* C1. *Est:* 6 h.
**C3. Config from book root** — fix `config.rs:157-190` (currently CWD-only); add `book.src`,
`build.*`; `extra-watch-dirs` into the watch list. *Deps:* C1. *Est:* 3 h.
**C4. `init` scaffolding** — `book.toml`, `src/SUMMARY.md`, `src/chapter_1.md`, `.gitignore`.
*Deps:* C2. *Est:* 2 h.

**Gate:** `md-book build ./test_book_mdbook` works with no flags; every existing `-i/-o`
invocation in CI, `Dockerfile` and `scripts/deploy.sh` still passes.

### Increment D -- output-correctness defects (2-2.5 d) [P1]

**D1. `path_to_root`** — compute per page; convert every template URL and `NavEntry.href`.
*Est:* 5 h.
**D2. Vendor Shoelace** — *shipped, revised in flight.* Copying the full distribution was
rejected once measured: 14MB over 2,920 files, and `copy_static_assets` writes templates into
every built book. Vendored instead: the transitive import closure of the five components used
(button, icon, input, spinner, card), the ten referenced icons, and light/dark themes — 43 JS
files, 356KB. `shoelace-local.js` replaces the CDN autoloader and derives `setBasePath` from
`import.meta.url`. Shadow-DOM components resolve their own stylesheets the same way, since they
cannot see Tera variables. *Actual:* 4 h.
**D3. Server-side heading IDs** — `render/slug.rs` + mdast injection; `doc-toc.js` reuses
existing IDs instead of minting them. *Est:* 5 h.
**D4. 404 page** — `input-404`, `site-url`; render `404.html`. *Est:* 2 h.
**D5. Copy button** — reference `js/code-copy.js` from templates (dead since it was added).
*Est:* 1 h.

**Gate:** built output loads from `file://` and from a sub-path with no network.

### Increment E -- renderer polish (2-3 d) [P3]

**E1. Themes** — *shipped.* `themes.css` custom properties for light/rust/coal/navy/ayu, a
`<details>`-based picker in the header (keyboard accessible without script; `theme-switch.js`
only applies the choice and marks it with `aria-current`), `localStorage` persistence, and
`default-theme` / `preferred-dark-theme` surfaced as `data-*` attributes on the root element.
Browser-verified: selecting Coal sets `data-theme`, computed `background` becomes `#141617`,
the choice survives navigation, and the picker closes on select. *Est:* 8 h.
**E2. Configurable syntax theme** — *shipped.* `output.html.syntax-theme` plus a new
`syntax-theme-dark`, replacing the hard-coded `"Solarized (light)"`. One stylesheet carries both:
light rules unscoped, dark rules prefixed with `[data-theme="coal"|"navy"|"ayu"]` by `scope_css`,
so the picker switches code colours too. An unknown theme name warns and lists the available
ones rather than failing the build. *Actual:* 3 h.
**E3. Print page** — `print.html.tera`, `print.css`, `[output.html.print]` enable/page-break.
*Est:* 4 h.
**E4. `additional-css` / `additional-js`** — *shipped.* Copied into `additional/` and injected
last, so author styles override the defaults. A listed file that does not exist warns rather than
being skipped in silence. *Actual:* 2 h.
**E5. Redirects + fold** — *shipped.* Redirect stubs were already in place; folding now works:
`to_nav` marks each chapter with `has_children` and whether its sub-list `starts_folded`, honouring
`level` and always leaving the branch containing the current page open. The list is hidden by class,
so `fold.js` can toggle it without a rebuild. *Actual:* 4 h.
**E6. Keyboard shortcuts** — `←`/`→`/`s`/`/`/`?`. *Est:* 2 h.

**E7. Load mermaid only on pages that contain a diagram** — *shipped.* Two extras found while
building: `index.html.tera` and `print.html.tera` never loaded mermaid at all, so a diagram in
`index.md` or in the print view silently failed to render. Both now load it, gated. — `page.html.tera:14-15` loads
`js/mermaid.min.js` (2.9MB) and `js/mermaid-init.js` on *every* page, whether or not it has a
diagram. On the 30-page corpus that is 2.9MB of the 4.2MB output, and a parse-and-execute cost
on every page load for the majority of books that contain no diagrams at all.

Design:

1. `render::markdown` already special-cases mermaid fences (`src/render/markdown.rs:156-161`),
   emitting `<code class="language-mermaid">`. Have that path report back rather than only
   emit: return `RenderedMarkdown { html: String, has_mermaid: bool }` from the markdown stage,
   set when at least one mermaid fence was rendered. Detecting at fence level is exact --
   substring-searching the finished HTML for `language-mermaid` would also match a page that
   merely *documents* the class in a code sample.
2. Thread `has_mermaid` into the page context alongside `path_to_root`, and gate both script
   tags on it:
   ```jinja
   {% if has_mermaid %}
   <script src="{{ path_to_root }}js/mermaid.min.js" type="module"></script>
   <script src="{{ path_to_root }}js/mermaid-init.js" type="module"></script>
   {% endif %}
   ```
3. `print.html` aggregates every chapter, so it sets `has_mermaid` if *any* included chapter
   does. The index page sets it from `index.md` only.
4. Keep emitting `js/mermaid.min.js` unconditionally in `copy_static_assets`. It is one file in
   the asset tree; making its presence conditional would mean a book whose only diagram is added
   later silently fails until a full rebuild. Revisit only if output size, rather than per-page
   cost, becomes the constraint.

Tests:

| Test | Purpose |
|------|---------|
| `test_mermaid_scripts_only_on_diagram_pages` | A book with one mermaid page and one plain page: the script tags appear in the first and not the second |
| `test_mermaid_class_in_code_sample_does_not_trigger_load` | A page that shows ```` ```language-mermaid ```` inside a fenced sample must not load mermaid -- guards the fence-level detection against a substring shortcut |
| `test_print_page_loads_mermaid_when_any_chapter_has_one` | Aggregate page behaviour |

*Est:* 3 h. Measured on the corpus: pages loading the bundle went from all 38 to zero (it has no
mermaid fences), so a reader pulls ~440KB instead of ~3.3MB per page. Five tests, including the
code-sample guard and both print-page directions.

**Gate:** theme choice persists across pages; `print.html` contains every chapter in book order;
a page with no diagram references no mermaid script.

### Increment G -- page metadata and skip link (0.5-1 d) -- SHIPPED

Recovers the work sitting in `stash@{0}` and finishes it. The stash was written against the
pre-increment-A monolith, so it **cannot be popped**: `src/core.rs` no longer contains the
functions its hunks patch, and all three templates it touches were rewritten by C-E. It is a
source to salvage from, not a change to replay. Two of its items have already landed
independently (the `lang` attribute and the header `aria-label`s), so only four remain.

**What is missing today** (verified, not assumed):

| Item | State on the branch |
|------|--------------------|
| `<meta name="description">` | absent; nothing derives a per-page description |
| `<link rel="canonical">` | absent; `book.base_url` and `output.html.site-url` both parse and are unused |
| Skip link + `id` on the article | absent; keyboard users cannot bypass the sidebar |
| Search gating | absent; the search modal loads even when no Pagefind index exists |

**G1. Per-page description** -- `src/render/meta.rs`. Salvage `extract_description` and
`first_plaintext_paragraph` from the stash, but **not** their implementation: the stashed version
does `.map(|c| if c.is_ascii() { c } else { ' ' })`, which turns "café" into "caf ". Derive the
text from the parsed mdast instead -- take the first `Paragraph` node and concatenate its `Text`
descendants -- so emphasis, links and inline code flatten correctly and Unicode survives.
`book::flatten_title` already does the equivalent for titles and is the model to follow.
Precedence: first paragraph, then `book.description`, then the book title. *Est:* 3 h.

**G2. Canonical URLs, and reconcile the two config keys** -- md-book has a local `book.base_url`
and mdBook's `output.html.site-url`; both parse, neither is used. Prefer `site-url` (the
mdBook-compatible spelling), accept `base_url` as a deprecated alias, and warn when both are set
and disagree. `canonical_url(base, page_path)` joins them with exactly one separator. Absent
config means no `<link rel="canonical">` at all -- never emit a relative canonical, which is
worse than none. *Est:* 2 h.

**G3. Skip link** -- `<a class="skip-link" href="#main-content">` as the first focusable element,
`id="main-content"` on `<article>`, and the `.skip-link` rule (visually hidden until focused)
salvaged from the stash's `styles.css` hunk. Applies to page, index and 404; the print page has
no sidebar to skip. *Est:* 1 h.

**G4. Search gating** -- restore `search_index_available(output_dir)` (checks for
`pagefind/pagefind.js`) and pass `search_enabled` into the templates, so the modal and its
scripts are omitted when no index was produced. This matters more since increment D: Pagefind is
optional and its absence currently ships a search box that silently does nothing. *Est:* 2 h.

Tests:

| Test | Purpose |
|------|---------|
| `test_description_from_first_paragraph` | Precedence, and that markup flattens |
| `test_description_preserves_non_ascii` | Guards the defect in the stashed implementation |
| `test_description_falls_back_to_book_then_title` | Both fallbacks |
| `test_canonical_absent_without_site_url` | No config, no canonical tag |
| `test_canonical_joins_site_url_once` | Trailing/leading slash handling |
| `test_base_url_alias_warns_when_it_disagrees` | Deprecated alias behaviour |
| `test_skip_link_is_first_focusable_and_targets_article` | Structural, on built output |
| `test_search_modal_omitted_without_index` | Gating both ways |

**Gate: met.** A built page carries a description and, with `site-url` set, a canonical URL; the
skip link is the first focusable element and `#main-content` exists; with no Pagefind index no
search UI is emitted, and it reappears once an index is present. `stash@{0}` is now fully superseded: its
`src/core.rs` helpers, the four template items and the skip-link style were checked one by one
against what shipped. It is left in place for the owner to drop (`git stash drop stash@{0}`);
nothing in it is still needed. The skip-link style differs deliberately -- it uses the theme
variables rather than the stash's `--sl-color-primary-600`, so it adapts to all five themes.

Deviation worth noting: `render_page` had grown to 16 positional arguments, so this increment
collapsed them into a `PageRender` struct rather than adding two more.

### Increment F -- `pulldown-cmark` backend (designed only; separate cycle)

Interface fixed now so increment A's `render/markdown.rs` split is right:

```rust
// src/render/markdown.rs
pub trait MarkdownBackend {
    /// Render markdown to an HTML fragment, highlighting code blocks.
    fn render(&self, content: &str, config: &BookConfig) -> Result<String>;
}

#[cfg(feature = "parser-markdown")] pub struct MarkdownCrateBackend;  // default; MDX
#[cfg(feature = "parser-cmark")]    pub struct CmarkBackend;          // footnotes, deflists,
                                                                       // admonitions, smart punct
```

```toml
# Cargo.toml
[features]
default = ["server", "watcher", "search", "syntax-highlighting", "parser-markdown"]
parser-markdown = []                          # markdown 1.0.0-alpha.21, MDX
parser-cmark    = ["pulldown-cmark"]          # 0.13.4, mdBook-compatible extensions
```

Constraint: the two features are additive but the *active* backend is one, selected by
`markdown.parser` in `book.toml` and defaulting to whichever is compiled in. Both backends must
satisfy the same conformance fixtures for content the `markdown` crate can already express;
group C features are asserted only under `parser-cmark`.

## Rollback plan

Each increment is a separate PR against `main` and is revertable on its own.

1. **B** is the only behaviour-breaking increment. If summary mode misbehaves in the field,
   revert the mode selector in `src/core.rs` (one call site) to force
   `book_from_directory` -- the parser stays in the tree, dormant.
2. **D1** (`path_to_root`) is the riskiest cosmetic change: revert the template commit; the Rust
   side is additive (the variable becomes unused).
3. **C** is additive; reverting restores flag-only invocation.
4. No data migration, no persistent state, no feature-flag runtime needed beyond the above.

## Dependencies

### New dependencies

| Crate | Version | Justification | Increment |
|-------|---------|---------------|-----------|
| `include_dir` | 0.7.4 | Embeds the default css/js/img/components trees and the vendored Shoelace subset, so an installed binary emits a complete, offline book with no templates directory on disk | D (shipped) |
| `pulldown-cmark` | 0.13.4 | Optional second parser backend; behind `parser-cmark` | F (not now) |

`include_dir` was not anticipated by this plan. Summary parsing uses the existing `markdown` crate's
mdast (a `SUMMARY.md` is a markdown document); slugging and `path_to_root` are `std`. Shoelace is
vendored as static assets, not as a dependency. WASM builds keep compiling because everything
added is `std`-only or behind existing feature gates.

## Performance considerations

| Metric | Target | Measurement |
|--------|--------|-------------|
| Full build, `test_book_mdbook` | Within 10% of the A3 baseline | **Measured 2026-08-08: 127 ms vs 100 ms on `main` (+27%).** Gate not met on the raw number; see attribution below. |
| Summary parse | < 5 ms for 500 entries | New criterion bench |
| Watch rebuild | No regression vs today | Manual, `--watch` |

Benchmarks to add:

```rust
// benches/summary_bench.rs
fn bench_parse_summary_500_entries(c: &mut Criterion) { /* generated fixture */ }
fn bench_book_to_nav_500_chapters(c: &mut Criterion) { /* to_nav on every page */ }
```

`to_nav` runs once per page, so it is O(pages × chapters).

**Measured (release build, 2026-08-08):**

| Book size | Build time | Per page |
|-----------|-----------|----------|
| 1 page | 10 ms | -- (fixed cost: assets) |
| 30 pages (corpus) | 127 ms | 4.0 ms |
| 50 pages | 57 ms | 1.15 ms |
| 200 pages | 258 ms | 1.29 ms |
| 500 pages | 1153 ms | 2.31 ms |

Per-page cost doubles between 50 and 500 pages, so the O(pages × chapters) term is real and bites
well before the 1,000-chapter threshold this plan guessed at. It is, however, **inherent to the
output format**: every page embeds the full sidebar, so the total sidebar markup is
pages × chapters whatever the implementation does. mdBook has the same property. Hoisting the
flatten would shave construction but not serialisation or rendering, so it is not obviously worth
the complexity; 500 pages in 1.15 s is acceptable.

Against `main`, the 30-page corpus went 100 ms → 127 ms (+27%), missing the plan's 10% gate. The
gate is recorded as **not met, with attribution**: the branch writes 4.2 MB of assets per build
that `main` never wrote (`main` emitted no `css/`, `js/` or `img/` at all -- the defect increment
D fixed), and adds correctness work per page (heading-ID injection, nav tree, theme context). One
avoidable cost was found and removed by this measurement: `has_mermaid` originally re-parsed each
page's mdast, which the highlighting path already walks -- worth 12 ms of the 30-page build.

## Open items

| Item | Status | Owner |
|------|--------|-------|
| Sign-off on the research doc's Interpretation B (contract parity) | **Resolved** -- approved 2026-08-08 | Alex |
| Directory-derived section titles (the literal `"Guide"`) | **Resolved** -- nothing depends on them; `dist/` is a demo build of the defective output | Claude |
| Sub-path deployment priority | **Resolved** -- not needed by this repo (root-published `dist`), needed by md-book's GitHub Pages users; `path_to_root` stays in D | Alex |
| Warn vs. silent for unsupported config keys | **Resolved** -- warn once on stderr at config load | Alex |
| Gitea issues per increment, with `Refs #IDX` on every commit | Created -- `terraphim/md-book` #1-#5, dependency-ordered | Claude |
| P2 preprocessing research cycle (`{{#include}}`, anchors, hidden lines, Rust attributes) | Deferred | -- |
| Increment F implementation cycle | Deferred | -- |

## Approval

- [x] Technical review complete
- [x] Test strategy approved
- [x] Performance targets agreed
- [x] Increment sequencing agreed (A → B → C → D → E, F later)
- [x] Human approval received -- Alex Mikhalev, 2026-08-08

**Next phase:** `disciplined-specification` (Phase 2.5) on increment B -- **complete**, findings
appended below. Implementation may proceed with increment A.

---

## Specification Interview Findings -- Increment B

**Interview date**: 2026-08-08
**Dimensions covered**: Failure modes; Edge cases & boundaries; Concurrency; Integration
effects; Migration & compatibility; Operational concerns; Security; Accessibility; User mental
models (9 of 10 -- scale & performance was already settled in the plan's benchmark section).
**Convergence status**: Complete. Round 3 produced one design consequence (nested markup vs the
flat `NavEntry` list, folded into the API above) and otherwise confirmed the recommended
positions.

### Key decisions from interview

#### Failure modes

- **`create-missing` is honoured, defaulting to true** -- matching mdBook. A `SUMMARY.md` entry
  whose file is absent causes md-book to write a stub into `src/` and build it. Accepted
  consequence: the build mutates the user's source tree by default. The stub is a single H1
  derived from the SUMMARY link text; nothing else is written, and an existing file is never
  overwritten.
- **Summary syntax errors are collected, not fail-fast.** Parse the whole file, accumulate every
  problem with line number and offending text, print them together, exit non-zero. Rationale:
  migrating a large book should take one build round-trip, not one per typo. This changes
  `parse_summary` from `Result<Summary, SummaryError>` to returning
  `Result<Summary, Vec<SummaryError>>` (or a `SummaryErrors` collection implementing
  `Display` over the set) -- **update the API signature accordingly during B2**.

#### Edge cases and boundaries

Accepted SUMMARY link forms:

| Form | Behaviour |
|------|-----------|
| `[API](api.md#errors)` | Fragment stripped for file resolution, preserved on the sidebar href. Depends on increment D's server-side heading IDs to actually land. |
| `[Rust](https://rust-lang.org)` | Sidebar entry linking off-site; no page generated, no slot in the prev/next chain, `rel="external"`. |

Rejected forms -- both join the collected error set and abort the build:

| Form | Rationale |
|------|-----------|
| The same file listed twice | Almost always a copy-paste error; the error names both line numbers. |
| Non-`.md` target, e.g. `[Sample](sample.rs)` | The summary describes a tree of markdown chapters; assets are reached by in-page links, not sidebar entries. |

- **Unlisted files**: non-markdown files under `src/` are copied through to the build directory
  preserving relative paths (this is how in-book images work). Orphan `.md` files are not
  published, and each is named in a warning so accidental omissions stay visible.

#### Concurrency

- **`create-missing` under `--watch`/`--serve`**: md-book records the paths it creates and
  suppresses the next watcher event for each, so a created stub cannot trigger a rebuild. The
  system converges after a single build because the second parse finds the file present.
  Implementation note: the suppression set is consumed (not merely checked), so a genuine user
  edit to that same file immediately afterwards still triggers a rebuild.

#### Security

- **Path containment is enforced.** Every resolved chapter path is canonicalised and must remain
  under the source directory; `../../private/notes.md` and absolute paths are refused as
  collected errors. Rationale: a documentation build that reads outside its tree and republishes
  the contents is an exfiltration path, and `SUMMARY.md` may arrive from an untrusted pull
  request in CI. Canonicalisation happens after symlink resolution, so a symlink inside `src/`
  pointing outside it is also refused.
- New test: `test_summary_rejects_paths_escaping_src`, including the symlink case.

#### User mental models

- **SUMMARY link text wins as the chapter title**, for the sidebar, `<title>`, prev/next labels
  and numbering. Inline markdown in the link text is rendered in HTML contexts and flattened for
  `<title>`. The page's own H1 is left untouched. This also fixes today's defect where raw
  `**bold**` from a heading leaks into the browser tab (`core.rs:388-393`).
- **Section numbers appear in the sidebar only**, suppressible with `no-section-label`. Page
  headings, `<title>` and therefore Pagefind's indexed titles stay as the author wrote them --
  numbering is a navigation aid, not a rewrite of the author's content.

#### Integration effects

- **Previous/next chain** covers every reachable page in authored order -- prefix, then numbered,
  then suffix. Draft chapters, part titles, separators and external links are skipped rather
  than becoming dead ends, since none of them has a page.

#### Accessibility

- The sidebar emits genuinely nested `<ul>` markup mirroring the chapter tree; part titles are
  real `<h2>` elements between lists; separators are `<hr aria-hidden="true">`; the active page
  carries `aria-current="page"`; draft chapters render as `<span aria-disabled="true">` rather
  than links, so keyboard users never tab into a dead end. Section numbers sit inside the link
  text. The whole nav is wrapped in `<nav aria-label="Book navigation">`.
- **Design consequence**: Tera macros cannot recurse, so nested markup cannot come from a plain
  depth-tagged list. `NavEntry` gains `open_lists` / `close_lists` deltas (see the API section
  above) allowing one template loop to emit correct nesting. New test:
  `test_to_nav_list_deltas_balance` -- the sum of opens equals the sum of closes for any tree.

#### Migration and compatibility

- The deprecated flat `sections` variable is populated as **one section per top-level chapter**,
  titled after that chapter and containing its descendants flattened. An unmigrated template
  renders a sensible, if flatter, sidebar. A deprecation warning fires when a custom template
  directory is in use. Removal target: **0.3.0**.

### Deferred items

| Item | Deferred because |
|------|------------------|
| Scale beyond ~1,000 chapters (hoisting `to_nav` out of the page loop) | Measure first; the corpus is 30 pages. Bench added in increment A. |
| Non-UTF-8 paths in `SUMMARY.md` | Existing code already surfaces these as errors (`core.rs:485-489`); no new behaviour needed. |
| Per-chapter search enable (`[output.html.search.chapter]`) | Pagefind-side capability question, not a book-model question. |

### Interview summary

Three rounds, four questions each. The interview changed the plan in five substantive ways.
Two decisions went against the drafted recommendation: `create-missing` is honoured with mdBook's
default of true (the plan had assumed a non-destructive warn-and-draft), and summary errors are
collected rather than fail-fast (the plan's error enum implied one error per build). Both are
recorded in the API section above and must be reflected in B2's signatures before coding starts.

The security dimension surfaced a requirement absent from the plan entirely: path containment.
`SUMMARY.md` is author-controlled input that names files to read and republish, which in CI can
mean input from an untrusted pull request. Rejecting anything that canonicalises outside `src/`
-- symlinks included -- closes that path, and is cheap to enforce at resolution time.

The accessibility decision had the largest design ripple. Committing to genuinely nested `<ul>`
markup with `aria-current` and disabled draft entries is incompatible with the flat depth-tagged
`NavEntry` list, because Tera macros cannot recurse. Adding `open_lists`/`close_lists` deltas
keeps the single-loop template while producing correct hierarchy for screen readers. The
remaining decisions -- SUMMARY link text as the title source, sidebar-only numbering, the
prev/next chain skipping page-less items, and asset pass-through with orphan warnings -- all
confirmed the drafted positions and needed no structural change.
