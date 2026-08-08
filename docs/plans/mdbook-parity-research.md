# Research Document: mdBook Feature Parity

**Status**: Approved (2026-08-08)
**Author**: Claude (Opus 5), with Alex Mikhalev
**Date**: 2026-08-08
**Reviewers**: Alex Mikhalev
**Phase**: 1 (disciplined-research)

## Executive Summary

md-book is not a git fork of mdBook -- it is an independent Rust reimplementation that reuses
mdBook's `book.toml` shape and the `MDBOOK_` environment prefix. It currently implements roughly
a third of mdBook's documented surface. The single largest divergence is that **md-book has no
`SUMMARY.md` parser at all**: book structure is inferred from a `walkdir` traversal sorted by
path, with sections derived from parent directory names. Everything mdBook builds on top of the
summary (ordering, nesting, numbering, prefix/suffix chapters, draft chapters, part titles,
file exclusion, correct previous/next) is therefore absent or wrong.

Three other structural gaps compound this: no preprocessor/link-expansion layer
(`{{#include}}`, anchors, `{{#playground}}`), no theme/print/404/redirect renderer features,
and no server-side heading IDs (added client-side only by `doc-toc.js`).

Several md-book decisions are deliberate improvements and must be retained: Pagefind instead
of elasticlunr, Tera instead of Handlebars, Web Components, the `markdown` crate (CommonMark /
GFM / MDX), `twelf` layered configuration, `jiff`, and the WASM build target. Parity should be
read as **behavioural parity of the authored book contract**, not as reimplementing mdBook's
internals.

## Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| Energizing? | Yes | md-book is positioned in `Cargo.toml` and `README.md` as "a modern mdbook replacement"; a replacement that cannot build an existing mdBook book does not meet its own claim. |
| Leverages strengths? | Yes | Rust performance work, Pagefind, Web Components and Terraphim's docs pipeline are existing in-house capability; `test_book_mdbook/` (mdBook's own test book) is already vendored as a conformance corpus. |
| Meets real need? | Yes | `test_book_mdbook/src/SUMMARY.md` uses prefix chapters, draft chapters, part titles and separators -- none of which md-book honours today, so the vendored corpus renders incorrectly. Terraphim docs are the immediate consumer. |

**Proceed**: Yes (3/3).

## Problem Statement

### Description

md-book claims to replace mdBook but cannot faithfully render an mdBook book. Given an existing
`src/` + `SUMMARY.md`, md-book will emit pages in path-sorted order, invent sections from
directory names, ignore the author's declared structure, silently publish files the author
excluded from the summary, and drop every `{{#include}}` directive as literal text.

### Impact

- **Migrating users** (the stated target): output is structurally wrong on the first build; the
  failure is silent, not an error.
- **Terraphim docs**: cannot move off mdBook without hand-restructuring content into
  directory-per-section shape.
- **The vendored conformance corpus** (`test_book_mdbook/`) passes today's tests only because
  those tests assert file existence and substring presence, not structure
  (`tests/integration/mdbook_compatibility.rs:38-47`).

### Success Criteria

1. `md-book -i test_book_mdbook/src -o out` produces a sidebar, ordering, and previous/next
   chain that matches `test_book_mdbook/src/SUMMARY.md` exactly, including prefix chapters,
   part titles, draft chapters and separators.
2. Files present in `src/` but absent from `SUMMARY.md` are not published.
3. A book with `{{#include}}` / anchors renders the included content.
4. Generated pages carry stable server-side heading IDs.
5. Output is deployable under a sub-path and offline (no CDN requirement).
6. No regression in the retained local decisions (Pagefind, Tera, Web Components, WASM).

## Current State Analysis

### Existing Implementation

| Component | Location | Purpose |
|-----------|----------|---------|
| CLI args | `src/core.rs:27-56` | Single command: `-i`, `-o`, `-c`, `--watch`, `--serve`, `--port`. No subcommands. |
| Build pipeline | `src/core.rs:129-386` | Template load, asset copy, walkdir collect, per-file render, index render. |
| Navigation model | `src/core.rs:170-225` | `BTreeMap<parent_dir, Vec<PageInfo>>`; root pages forced into a section literally titled `"Guide"`. |
| Previous/next | `src/core.rs:274-284` | Index into path-sorted `all_pages` -- not authored order. |
| Title extraction | `src/core.rs:388-393` | First line starting `"# "`; raw markdown is kept verbatim (`**bold**` leaks into `<title>`). |
| Markdown → HTML | `src/core.rs:714-879` | `markdown` crate; code blocks spliced out by mdast offset and re-highlighted with syntect. |
| Link rewriting | `src/core.rs:640-712` | Manual `href="…md"` → `.html` scan over rendered HTML. |
| Config | `src/config.rs:1-190` | `twelf` layers: `MDBOOK_` env → `book.toml` → custom `.toml`/`.json`. |
| Config load | `src/config.rs:157-190` | `book.toml` only read from the **current working directory**, not the book directory. |
| Search | `src/pagefind_service.rs` | Pagefind CLI subprocess after HTML generation. |
| Server | `src/server.rs`, `src/main.rs:183-240` | warp static + WebSocket reload; `notify` watcher with 500 ms debounce. |
| Templates | `src/templates/*.tera` | `page`, `index`, `sidebar`, `header`, `footer`; overridable via `paths.templates`. |
| Components | `src/templates/components/*.js` | `doc-toc`, `doc-sidebar`, `search-modal`, `simple-block`. |

### Data Flow

`walkdir(input)` → filter `*.md` → sort by path → `PageInfo{title, path}` → group by parent dir
→ per-file: read → mdast → splice code blocks → highlight → HTML → `.md`→`.html` rewrite →
Tera `page` → write → (async) Pagefind index over the output directory.

There is no intermediate book model, no preprocessing stage, and no renderer abstraction.

### Integration Points

`pagefind` CLI (must be on `PATH`); Shoelace 2.12.0 via jsDelivr CDN (`page.html.tera:12-13`);
`mermaid.min.js` (vendored); `syntect` default theme set.

## Parity Matrix

Legend: **Have** = implemented; **Partial** = present but divergent/incomplete; **Missing**;
**N/A (local decision)** = deliberately not matching mdBook.

### A. Book structure and navigation

| mdBook feature | md-book | Evidence / note |
|---|---|---|
| `SUMMARY.md` as source of truth | **Missing** | `src/core.rs:175-225` walks the directory instead. |
| Chapter ordering from summary | **Missing** | Path-sorted (`core.rs:182`). |
| Nested sub-chapters (arbitrary depth) | **Missing** | Flat one-level `section → pages`. |
| Part titles (`# Part Name`) | **Missing** | -- |
| Prefix / suffix chapters | **Missing** | -- |
| Draft chapters (`- [Title]()`) | **Missing** | -- |
| Separators (`---`) | **Missing** | -- |
| Section numbering (`1.1`), `no-section-label` | **Missing** | -- |
| Exclude files not in summary | **Missing** | Every `*.md` under input is published. |
| `README.md` → `index.html` (index preprocessor) | **Missing** | Requires `index.md`; `core.rs:347-371`. |
| Correct previous/next | **Partial** | Exists, but ordered by path (`core.rs:274-284`). |
| `book.src` config key | **Missing** | Source dir comes from `-i` only. |
| `build.build-dir`, `create-missing`, `extra-watch-dirs` | **Missing** | `-o` only; templates dir is watched (`main.rs:150`). |

### B. CLI surface

| mdBook | md-book | Note |
|---|---|---|
| `init` (+`--theme`, `--title`, `--ignore`, `--force`) | **Missing** | No scaffolding path. |
| `build` / `watch` / `serve` / `clean` / `completions` subcommands | **Partial** | Single command with `--watch` / `--serve` flags. |
| `serve -n <hostname>`, `--open`, `-d/--dest-dir`, `--watcher poll\|native` | **Missing** | Only `--port`. |
| `test` (rustdoc doctests) | **Missing** | Requires a Rust toolchain integration. |

### C. Markdown, preprocessing, code

| mdBook | md-book | Note |
|---|---|---|
| `{{#include file}}` + `:line` ranges | **Missing** | Emitted literally. |
| `ANCHOR:` / `ANCHOR_END:` anchors | **Missing** | -- |
| `{{#rustdoc_include}}`, `{{#playground}}` | **Missing** | -- |
| `{{#title …}}` | **Missing** | -- |
| Hidden lines (`# ` prefix, `hidelines=`) | **Missing** | Rust `#` lines render verbatim. |
| Rust attributes: `ignore`, `no_run`, `should_panic`, `compile_fail`, `editable`, `noplayground`, `editionYYYY` | **Missing** | `core.rs:560-596` only branches on `rust` / `mermaid` / other; an unknown token such as `rust,ignore` fails syntax lookup and falls back to plain text. |
| MathJax `\\(…\\)` / `\\[…\\]` | **Missing** | `mathjax-support` parses but is unused; test ignored at `core.rs:1109`. |
| Smart punctuation | **Missing** | -- |
| Footnotes | **Missing** | Not in the `markdown` crate's GFM set. |
| Definition lists, admonitions (`> [!NOTE]`) | **Missing** | -- |
| Heading attributes `{ #id .class }` | **Missing** | -- |
| Heading anchor IDs in HTML | **Missing** | No `id` on any `<h*>` in generated output (verified against `dist_test_2/*.html`); `doc-toc.js:28-39` injects them client-side only, so cross-page `#fragment` links and Pagefind anchors fail. |
| Strikethrough / tables / task lists | **Have** | Via GFM mode. |
| `.md` → `.html` link conversion | **Have** | `core.rs:640-712`. |
| Preprocessor protocol (`mdbook-*`, JSON over stdio) | **Missing** | No plugin surface. |
| Alternative backends (`[output.*]`), `markdown` renderer | **Missing** | HTML only. |
| Mermaid diagrams | **Have (superset)** | mdBook needs a third-party preprocessor. |
| MDX input | **Have (superset)** | -- |

### D. HTML renderer

| mdBook | md-book | Note |
|---|---|---|
| Themes: Light/Rust/Coal/Navy/Ayu + toggle, `default-theme`, `preferred-dark-theme` | **Missing** | Single stylesheet. |
| Print page (`print.html`, `print.css`, `page-break`) | **Missing** | -- |
| 404 page (`input-404`, `site-url`) | **Missing** | -- |
| `[output.html.redirect]` map | **Missing** | -- |
| `additional-css` / `additional-js` | **Missing** | -- |
| Sidebar fold (`[output.html.fold]`) | **Missing** | -- |
| Keyboard shortcuts (←/→, `s`, `/`, `?`) | **Missing** | -- |
| Copy-to-clipboard button | **Partial** | `js/code-copy.js` exists but is referenced by no template -- dead code. |
| Configurable syntax theme | **Missing** | `"Solarized (light)"` hard-coded, `core.rs:242` (marked TODO). |
| `git-repository-url` / `-icon` | **Partial (renamed)** | Local `book.github_url`. |
| `edit-url-template` with `{path}` | **Partial (renamed)** | Local `book.github_edit_url_base` + suffix. |
| `cname`, `hash-files`, `text-direction` | **Missing** | -- |
| Relative asset paths (`path_to_root`) | **Missing (defect)** | Templates hard-code absolute `/css/...`, `/js/...`, and `PageInfo.path` is `/`-prefixed (`core.rs:196`), so output only works at a domain root. |
| Self-contained offline output | **Missing (defect)** | Shoelace loaded from jsDelivr (`page.html.tera:12-13`); mdBook output is fully offline. |
| Search backend | **N/A (local decision)** | Pagefind replaces elasticlunr. Consequence: `use-boolean-and`, `boost-*`, `expand`, `teaser-word-count`, `copy-js` have no Pagefind equivalent; `limit-results` and `heading-split-level` do. These keys parse today and are silently ignored. |
| Templating engine | **N/A (local decision)** | Tera, not Handlebars: `index.hbs`/`head.hbs`/`{{#toc}}`/`{{fa}}` will not be supported. |
| SEO: canonical URL, meta description, skip link | **Have (superset)** | Added by the browser-validation work. |

## Constraints

### Technical Constraints

- **`markdown` crate (1.0.0-alpha.21)** does not emit heading IDs, footnotes, definition lists,
  admonitions, smart punctuation, or heading attributes. Several C-row gaps therefore require
  either post-processing the HTML/mdast or changing parser. mdBook itself uses `pulldown-cmark`.
- **Code-block splicing by byte offset** (`core.rs:747-813`) means any preprocessing that
  rewrites markdown must run *before* this stage, on the markdown text, not on the HTML.
- **WASM target** must keep building: any new dependency has to be optional or wasm-safe
  (`features = ["wasm", "wasm-core"]`).
- **Pagefind CLI** is an external process; search remains a post-build step.
- Global project rules: no mocks in tests, `jiff` not `chrono`, British English, no emoji.

### Business Constraints

- md-book is not a Q3 priority in the current North Star; this work must be decomposable into
  small, independently shippable increments rather than one long migration.
- Existing published output (`dist/`, deploy workflows, Cloudflare Pages) must keep building.

### Non-Functional Requirements

| Requirement | Target | Current |
|-------------|--------|---------|
| Build determinism | Identical output for identical input | Met (path sort), but authored order not honoured |
| Sub-path deployment | Works under `/docs/` | Fails (absolute asset paths) |
| Offline output | No network at view time | Fails (Shoelace CDN) |
| Build speed | No worse than today on the vendored corpus | Baseline: `benches/pagefind_bench.rs` |

## Vital Few (Essentialism)

### Essential Constraints (max 3)

| Constraint | Why it's vital | Evidence |
|------------|----------------|----------|
| `SUMMARY.md` must become the book model | Every ordering, navigation, numbering and exclusion behaviour derives from it; without it no other parity item is observable to a migrating user. | `core.rs:170-225` vs `test_book_mdbook/src/SUMMARY.md` |
| Preprocessing must run on markdown before code-block splicing | `{{#include}}` and hidden-line handling are textual; retro-fitting them after HTML generation would break syntax highlighting. | `core.rs:747-813` |
| Local decisions are non-negotiable | Pagefind, Tera, Web Components, `markdown` crate, WASM. Parity means the *authored book contract*, not mdBook internals. | User instruction; `Cargo.toml` features |

### Eliminated from Scope (5/25 rule)

| Eliminated item | Why eliminated |
|-----------------|----------------|
| Handlebars theme compatibility (`index.hbs`, `{{#toc}}`, `{{fa}}`) | Contradicts the Tera decision; would fork the template layer. |
| elasticlunr search + its config semantics (`boost-*`, `expand`, `use-boolean-and`, `teaser-word-count`) | Contradicts the Pagefind decision. Resolve by documenting these keys as unsupported rather than accepting them silently. |
| `mdbook test` (rustdoc doctests) | Requires toolchain orchestration; large, isolated, and unused by Terraphim docs. |
| Rust Playground integration (run button, editable editor, `[output.html.playground]` runtime) | Rust-book-specific; only the *rendering* attributes matter for correctness. |
| Preprocessor / alternative-backend plugin protocol (`mdbook-*` over stdio) | Only worth building once an internal preprocessing pipeline exists; premature now. |
| `hash-files`, `cname`, `text-direction`, `completions` | Low value relative to the structural gaps. |

## Dependencies

### Internal

| Dependency | Impact | Risk |
|------------|--------|------|
| `src/core.rs` `build_sync_impl_sync` | Single 250-line function owning collection, rendering and indexing; every structural change lands here. | High -- needs decomposition first |
| `src/templates/sidebar.html.tera` | Assumes flat `sections[].pages[]`; nesting changes the contract. | Medium -- breaks user-overridden templates |
| `tests/integration/mdbook_compatibility.rs` | Asserts substrings, not structure; gives false confidence. | Medium |
| `src/config.rs` | `book.toml` resolved from CWD, not book dir; blocks `mdbook build <dir>` semantics. | Medium |

### External

| Dependency | Version | Risk | Alternative |
|------------|---------|------|-------------|
| `markdown` | 1.0.0-alpha.21 | Alpha; lacks heading IDs/footnotes/admonitions; pinning risk | `pulldown-cmark` (what mdBook uses) -- a large, separate decision |
| `pagefind` | 1.3.0 | External CLI must be installed | Bundled crate API |
| `syntect` | 5.0 | Theme set fixed at build | -- |
| Shoelace | 2.12.0 CDN | Network dependency in output | Vendor locally |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Introducing `SUMMARY.md` breaks existing directory-driven books (including Terraphim's own docs and `dist/` deploys) | High | High | Fall back to today's directory walk when no `SUMMARY.md` is present; make summary mode opt-in-by-presence, never mandatory. |
| Nested navigation breaks user-supplied Tera templates | Medium | Medium | Keep the flat `sections` variable populated alongside a new `chapters` tree for one release; document the deprecation. |
| `markdown` crate cannot supply heading IDs / footnotes / admonitions | High | Medium | Post-process mdast (IDs, achievable) and defer footnotes/admonitions/definition lists to a separate parser decision. |
| Parser swap to `pulldown-cmark` becomes the real blocker for group C | Medium | High | Treat it as its own research/design cycle; do not bundle it into this plan. |
| Scope creep into full mdBook internals | High | Medium | The Eliminated table above is binding. |
| Rebuild performance degrades with a preprocessing pass | Low | Low | Benchmark against `benches/pagefind_bench.rs` before/after. |

### Open Questions -- all resolved 2026-08-08

1. **CLI shape** -- *Resolved*: add `md-book build|serve|watch|init|clean [dir]` honouring
   `book.src` / `build.build-dir`, and keep `-i/-o` as overrides so CI, `Dockerfile` and
   `scripts/deploy.sh` stay green.
2. **Parser stance** -- *Resolved*: `pulldown-cmark` becomes an additive, feature-flagged second
   backend (`parser-cmark`); the `markdown` crate stays the default and MDX is retained. Group C
   markdown features become reachable under the opt-in backend rather than being deferred
   outright. Implementation is its own cycle (increment F).
3. **Unsupported config keys** -- *Resolved*: warn once on stderr at config load. Silent
   acceptance is the current failure shape; a hard error would break every existing `book.toml`,
   including this repo's (three such keys today).
4. **Sub-path deployment** -- *Resolved*: not required by this repo's own consumers (Cloudflare
   Pages and Netlify both publish `dist` at domain root; the one gh-pages sub-path step
   publishes rustdoc from `target/doc`, not md-book output). It **is** required by md-book's
   users on GitHub Pages project sites, so `path_to_root` stays in increment D alongside the
   other relocatability/offline defects, which touch the same template lines.

Additionally verified during Phase 2, closing two assumptions from the table above:

5. **Directory-derived section titles** -- nothing depends on them. The only artefact showing
   them is `dist/`, a demo build of `test_book_mdbook` whose sidebar reads
   `Guide / headings / individual / languages / rust` -- i.e. the defective output this work
   replaces.
6. **User-supplied templates** -- only this repo's `paths.templates = "src/templates"` was found;
   the one-release `sections` deprecation window is sufficient.

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if wrong | Verified? |
|------------|-------|---------------|-----------|
| "Fork" means "independent reimplementation", not a git fork | No mdBook history in `git log`; no `pulldown-cmark`/`handlebars` in `Cargo.toml` | Wording only | Yes |
| Parity target is mdBook's *authored-book contract*, not its internals | User instruction to retain local decisions | Would mean reverting Pagefind/Tera | Stated, needs sign-off |
| `test_book_mdbook/` is the intended conformance corpus | Vendored, plus three dedicated test targets in `Cargo.toml` | Would need a new corpus | Yes |
| No consumer currently depends on directory-derived section titles (e.g. the literal `"Guide"` section) | `core.rs:216-219`; Terraphim docs use directories | Sidebar labels change for existing books | No -- check `dist/` |
| Existing user templates are rare (only `paths.templates` in this repo's `book.toml`) | Repo inspection | Breaking template change affects users | No |

### Multiple Interpretations Considered

| Interpretation | Implications | Chosen / rejected |
|----------------|--------------|-------------------|
| **A. Byte-level parity** -- match mdBook's HTML, themes and Handlebars | Reverts Pagefind, Tera, Web Components | **Rejected** -- contradicts "retain local decisions" |
| **B. Contract parity** -- any valid mdBook book builds correctly, with md-book's own presentation | Requires `SUMMARY.md`, preprocessing, renderer features; keeps local stack | **Chosen** |
| **C. Config parity only** -- accept every `book.toml` key, implement what is cheap | Cheap, but silently wrong output; today's failure mode | **Rejected** |

## Research Findings

### Key Insights

1. **The parity gap is structural, not cosmetic.** One missing artefact (`SUMMARY.md`) accounts
   for eleven of the thirteen A-row gaps and makes previous/next incorrect.
2. **Feature gaps are already encoded as configuration.** `book.toml` in this repo carries
   `# not implemented` comments on `mathjax-support`, `playground` and `search` -- config
   acceptance has outrun behaviour, which is the most user-hostile failure shape.
3. **Two defects are worse than missing features**: absolute asset paths (breaks sub-path
   deployment) and CDN-loaded Shoelace (breaks offline/air-gapped viewing). mdBook output is
   relocatable and self-contained; md-book's is neither.
4. **Group C is parser-bound.** Heading IDs are reachable via mdast post-processing, but
   footnotes, definition lists, admonitions and smart punctuation are properties of the parser.
   Recognising this prevents a doomed incremental effort.
5. **`{{#include}}` must be a pre-parse text pass**, sequenced ahead of the existing offset-based
   code-block splice.
6. **Existing "compatibility" tests do not test compatibility.** They assert files exist and
   contain substrings; a structural fixture comparison is needed for any of this to be provable.
7. **`build_sync_impl_sync` must be decomposed first.** Collection, preprocessing, rendering and
   indexing are one function; every parity item otherwise piles into it.

### Relevant Prior Art

- **mdBook** (`rust-lang/mdBook`) -- `SummaryParser`, `links` and `index` preprocessors, the
  `Book`/`BookItem` model: the reference for the book model shape.
- **mdBook's own test book** (`test_book_mdbook/`) -- already vendored; exercises prefix, draft,
  part titles, separators and per-tag markdown cases.
- **`docs/plans/browser-validation-research.md`** -- the in-repo precedent for this document
  shape; its explicit "Out of scope: a full `SUMMARY.md` parser" is the deferral this document
  now takes up.

### Technical Spikes Needed

| Spike | Purpose | Effort |
|-------|---------|--------|
| mdast heading-ID injection | Confirm stable GitHub-compatible slugs can be produced from `markdown` crate mdast without a parser swap | 2-4 h |
| Nested sidebar in Tera | Confirm recursive rendering of a chapter tree in Tera (no native recursion; needs a macro or pre-flattened list) | 2 h |
| `pulldown-cmark` evaluation | Cost of migration; whether MDX can be dropped | 1 day (separate cycle) |

## Recommendations

### Proceed / No-Proceed

**Proceed**, with interpretation B (contract parity), scoped to four sequenced increments and
explicitly excluding the Eliminated table. The prerequisite for all of them is decomposing
`build_sync_impl_sync` into collect → preprocess → render → index stages.

### Scope Recommendations

Priority order, by user-visible correctness per unit of effort:

1. **P0 -- Book model**: `SUMMARY.md` parser, chapter tree, correct ordering/previous/next,
   `README.md`→`index.html`, summary-driven exclusion, part titles, prefix/suffix, draft
   chapters, separators, section numbers. Directory-walk fallback preserved.
2. **P1 -- Output correctness defects**: relative asset paths (`path_to_root`), vendored
   Shoelace, server-side heading IDs, 404 page, wire up the dead copy button.
3. **P2 -- Preprocessing**: `{{#include}}` with line ranges and anchors, `{{#title}}`, hidden
   lines, Rust code-block attribute parsing (`rust,ignore` etc. must not degrade highlighting).
4. **P3 -- Renderer polish**: theme switching (light/dark), configurable syntax theme, print
   page, `additional-css`/`additional-js`, redirects, sidebar fold, keyboard shortcuts.

Cross-cutting: replace substring assertions with structural fixtures over `test_book_mdbook/`,
and make unsupported config keys warn rather than parse silently.

### Risk Mitigation Recommendations

- Gate the summary parser on the presence of `src/SUMMARY.md`; never break directory-mode books.
- Ship the chapter tree *alongside* the existing flat `sections` template variable for one
  release, with a documented deprecation.
- Benchmark before and after the preprocessing pass.
- Record the parser question (`pulldown-cmark`) as its own research cycle rather than absorbing
  it here.

## Next Steps

If approved:

1. Answer open questions 1-4 (CLI shape, parser stance, warning policy, sub-path requirement).
2. Proceed to Phase 2 (`disciplined-design`) for the P0 increment:
   `docs/plans/mdbook-parity-implementation-plan.md`.
3. Build the structural conformance fixture over `test_book_mdbook/` before implementation.

## Appendix

### Reference Materials

- mdBook documentation: <https://rust-lang.github.io/mdBook/>
- In-repo corpus: `test_book_mdbook/src/SUMMARY.md`
- Precedent: `docs/plans/browser-validation-research.md`

### Code Snippets

Current navigation derivation -- the crux of the A-row gaps (`src/core.rs:201-225`):

```rust
if parent_dir.is_empty() {
    root_pages.push(page_info);
} else {
    section_map.entry(parent_dir.to_string()).or_default().push(page_info);
}
// ...
if !root_pages.is_empty() {
    sections.push(Section { title: "Guide".to_string(), pages: root_pages });
}
for (title, pages) in section_map {
    sections.push(Section { title, pages });   // section title == directory name
}
```

Previous/next by path order rather than authored order (`src/core.rs:274-284`):

```rust
let previous = if current_page > 0 { Some(all_pages[current_page - 1].clone()) } else { None };
let next = if current_page + 1 < total_pages { Some(all_pages[current_page + 1].clone()) } else { None };
```
