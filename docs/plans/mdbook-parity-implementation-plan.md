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
| E | Renderer polish (P3) | 2-3 d | Themes, print, redirects, fold, shortcuts |

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
  shortcuts.
- Warnings for parsed-but-unsupported config keys.
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
| `src/templates/vendor/shoelace/*` | Vendored Shoelace 2.12.0 (CSS + autoloader) | D |
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

/// One entry of the pre-flattened sidebar list handed to Tera
/// (Tera has no template recursion, so nesting is expressed as `depth`).
#[derive(Debug, Clone, Serialize)]
pub struct NavEntry {
    pub kind: NavKind,          // Chapter | PartTitle | Separator
    pub title: String,
    /// Root-relative href; `None` for drafts, part titles and separators.
    pub href: Option<String>,
    /// Rendered section number, e.g. "1.2."; empty when `no-section-label` is set.
    pub number: String,
    pub depth: usize,
    pub is_draft: bool,
    pub is_active: bool,
}
```

### Public functions

```rust
// src/book/summary.rs

/// Parse the contents of a `SUMMARY.md`.
///
/// # Errors
/// - `SummaryError::MixedDelimiters` -- `-` and `*` both used for list items.
/// - `SummaryError::PrefixAfterNumbered` -- a prefix chapter follows numbered chapters.
/// - `SummaryError::Malformed` -- a list item that is not a link.
pub fn parse_summary(content: &str) -> Result<Summary, SummaryError>;

/// Build a `Book` from a summary, resolving paths against `src_dir` and
/// assigning section numbers.
///
/// # Errors
/// Returns `SummaryError::MissingFile` when a referenced file is absent and
/// `build.create-missing` is false.
pub fn book_from_summary(
    summary: &Summary,
    src_dir: &Path,
    create_missing: bool,
) -> Result<Book, SummaryError>;

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

    #[error("SUMMARY.md references missing file: {path}")]
    MissingFile { path: PathBuf },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
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
| `test_files_absent_from_summary_not_published` | same | Summary-driven exclusion |
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
**D2. Vendor Shoelace** — copy 2.12.0 CSS + autoloader into `src/templates/vendor/`; drop the
jsDelivr tags; assert no external URLs in output. *Est:* 3 h.
**D3. Server-side heading IDs** — `render/slug.rs` + mdast injection; `doc-toc.js` reuses
existing IDs instead of minting them. *Est:* 5 h.
**D4. 404 page** — `input-404`, `site-url`; render `404.html`. *Est:* 2 h.
**D5. Copy button** — reference `js/code-copy.js` from templates (dead since it was added).
*Est:* 1 h.

**Gate:** built output loads from `file://` and from a sub-path with no network.

### Increment E -- renderer polish (2-3 d) [P3]

**E1. Themes** — `themes.css` custom properties for light/rust/coal/navy/ayu; toggle with
`localStorage`; `default-theme` / `preferred-dark-theme`. *Est:* 8 h.
**E2. Configurable syntax theme** — `output.html.syntax-theme`, replacing the hard-coded
`"Solarized (light)"` (`core.rs:242`); a light and a dark stylesheet emitted. *Deps:* E1.
*Est:* 3 h.
**E3. Print page** — `print.html.tera`, `print.css`, `[output.html.print]` enable/page-break.
*Est:* 4 h.
**E4. `additional-css` / `additional-js`** — copy and inject. *Est:* 2 h.
**E5. Redirects + fold** — `[output.html.redirect]` stub pages; `[output.html.fold]` collapse
depth. *Est:* 4 h.
**E6. Keyboard shortcuts** — `←`/`→`/`s`/`/`/`?`. *Est:* 2 h.

**Gate:** theme choice persists across pages; `print.html` contains every chapter in book order.

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
| `pulldown-cmark` | 0.13.4 | Optional second parser backend; behind `parser-cmark` | F (not now) |

No new dependency is required for A-E. Summary parsing uses the existing `markdown` crate's
mdast (a `SUMMARY.md` is a markdown document); slugging and `path_to_root` are `std`. Shoelace is
vendored as static assets, not as a dependency. WASM builds keep compiling because everything
added is `std`-only or behind existing feature gates.

## Performance considerations

| Metric | Target | Measurement |
|--------|--------|-------------|
| Full build, `test_book_mdbook` | Within 10% of the A3 baseline | `benches/pagefind_bench.rs` |
| Summary parse | < 5 ms for 500 entries | New criterion bench |
| Watch rebuild | No regression vs today | Manual, `--watch` |

Benchmarks to add:

```rust
// benches/summary_bench.rs
fn bench_parse_summary_500_entries(c: &mut Criterion) { /* generated fixture */ }
fn bench_book_to_nav_500_chapters(c: &mut Criterion) { /* to_nav on every page */ }
```

`to_nav` runs once per page, so it is O(pages × chapters). At the corpus size (30 pages) this is
irrelevant; if a book exceeds ~1,000 chapters, hoist the flatten out of the page loop and mutate
only `is_active`. **Do not do this pre-emptively** -- measure first.

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

**Next phase:** `disciplined-specification` (Phase 2.5) on increment B before any code is
written -- the summary parser has the most edge cases (mixed delimiters, missing files,
`create-missing`, non-UTF-8 paths, duplicate entries, links with anchors or query strings).
Findings are appended to this document.
