# Research: Restore Coherent Shoelace Theme Token Parity

**Status**: Review
**Canonical Path**: `docs/plans/research-theme-token-parity.md`
**Change Slug**: `theme-token-parity`
**Author**: Codex
**Date**: 2026-08-13
**Reviewers**: Alex Mikhalev

## Executive Summary

The parity work retained Shoelace as the component library and made it work offline, but it separated the application theme chooser from the Shoelace design tokens that render nearly every surface. Choosing a dark theme changes the document body through four bespoke variables, while panels, navigation, cards, code, search and shadow-DOM components continue to consume the permanently loaded Shoelace *light* token set. The result is the inconsistent, washed-out combination visible in the supplied screenshot.

The smallest credible restoration is a semantic token contract for each component family: Shoelace as the repaired default, and Web Awesome as the expanded alternate theme family requested by the user. Both document CSS and each shadow root must consume their active family contract. The repair must also preserve the project's intended Pagefind full-text search with instant results: its current root-absolute imports break the parity sub-path deployment model, and its first-build index gating can remove the entire search UI. This preserves offline, sub-path, custom-template, theme-picker, Pagefind, and mdBook-parity behaviour without reintroducing an external asset dependency.

## Essential Questions Check

| Question | Answer | Evidence |
|---|---|---|
| Energizing? | Yes | The defect erodes the documentation product's visual identity and readability. |
| Leverages strengths? | Yes | The project owns its renderer, templates, vendored component assets, and visual tests. |
| Meets real need? | Yes | The supplied production screenshot shows a user-visible regression. |

**Proceed**: Yes — 3/3 answers are yes.

## Problem Statement

### Description

Recent mdBook-contract parity changes added the `light`, `rust`, `coal`, `navy`, and `ayu` picker. It applies `data-theme` successfully, but only changes `--bg`, `--fg`, `--sidebar-bg`, and `--accent` in `src/templates/css/themes.css`. The authored UI instead mostly uses `--sl-color-neutral-*`, `--sl-color-primary-*`, `--theme-*`, and Shoelace component tokens. Those resolve to the vendored light palette because every generated document and shadow component imports only `vendor/shoelace/themes/light.css`.

### Impact

- Readers see a dark main canvas alongside light or unrelated gray surfaces, weak link affordances, and mismatched code/search/card backgrounds.
- The component-library based visual system becomes indistinguishable from a set of ad-hoc overrides.
- Any selected theme may remain in `localStorage`, so the defect persists across pages and future book visits.
- Theme behaviour is not tested for token coherence; existing end-to-end evidence only checks that `data-theme=coal` and the page background survive navigation.

### Success Criteria

1. Every supported picker theme has coherent page, panel, sidebar, TOC, card, input, Pagefind search, code, link, and focus colours.
2. No document or shadow root hard-codes the light Shoelace theme when the active theme is dark.
3. Shoelace remains locally vendored and generated books remain offline and sub-path/file URL safe.
4. The rendered theme is accessible: normal text, interactive text, focus rings, and disabled/secondary text meet chosen contrast targets.
5. Pagefind full-text search returns results with low perceived latency as the reader types, at the domain root and from a deployment sub-path.
6. Visual and behavioural tests prove the full token contract for all five advertised themes, not merely the body background.

## Current State Analysis

### Existing Implementation

1. `src/templates/page.html.tera` and `src/templates/index.html.tera` always include `vendor/shoelace/themes/light.css`, followed by `css/styles.css` and `css/themes.css`.
2. `src/templates/js/theme-switch.js` persists `md-book-theme`, then sets `data-theme` to one of five names. It does not swap a stylesheet or validate persisted values.
3. `src/templates/css/themes.css` changes only four bespoke variables. Its final `body` rule wins over the `body` rule in `styles.css`, so the main background changes, but most descendants do not.
4. `src/templates/css/styles.css` maps Shoelace primary tokens to the gray scale at its root and uses Shoelace neutral/primary tokens throughout. It contains overlapping/duplicated historical layout and index/card rule blocks, increasing cascade uncertainty.
5. `src/templates/components/doc-toc.js`, `simple-block.js`, and `doc-sidebar.js` create shadow roots that link to the light Shoelace stylesheet and use Shoelace token names. Shadow trees cannot inherit the page CSS token declarations.
6. `src/templates/css/search.css` is largely Shoelace-token based but switches dark appearance only by OS media preference. That can conflict with an explicit reader choice such as `data-theme="navy"` on a light OS.
7. Pagefind is the project's declared full-text, instant-results search engine (`README.md`). Its browser wrapper, unchanged since `e464bc4`, hard-codes `/pagefind/` and `/` as the bundle/base paths. `search-modal.js` also has a root-absolute dynamic fallback import (`/js/pagefind-search.js`). These URLs work at a domain root but break under the supported `/docs/` sub-path.
8. `src/pipeline/mod.rs` decides whether to emit search UI before it runs Pagefind. On a clean first build no index exists, so it intentionally omits the search UI and only shows it after a second build. That makes a newly deployed build appear to have lost search even when indexing succeeds at the end of the build.

### Historical Findings

- Commit `eb61346` introduced a separate **Web Awesome** documentation theme, using `wa-*` components and a Web Awesome CDN stylesheet.
- Commit `6eeccae` deliberately replaced the earlier external component delivery with a local, trimmed **Shoelace** closure to meet offline/sub-path requirements.
- Commit `965aaeb` introduced the five mdBook-named theme values and the four-variable `themes.css` palette.
- Commit `0374f56` introduced the picker and persistence.
- The linked branch `origin/claude/create-webawesome-theme-01U5jiKf7NWSbvTo38XQHqYc` is available locally and contains `eb61346`. Its commit record confirms **Web Awesome (beta 3.0.0) from Font Awesome** was an intentional alternate theme, using `wa-button`, `wa-icon`, `wa-input`, `wa-card`, `wa-spinner`, a separate template family, and `book.webawesome.toml`.
- The historical Web Awesome implementation used an external early.webawesome.com CDN, whereas current Shoelace is locally vendored. Expanding Web Awesome therefore requires an explicit local, version-pinned asset delivery plan before it can meet current offline parity.

### Code Location Map

| Component | Location | Purpose / finding |
|---|---|---|
| Page template | `src/templates/page.html.tera` | Always loads Shoelace light CSS; owns chapter asset order. |
| Index template | `src/templates/index.html.tera` | Same light-only load; owns home-page asset order. |
| Theme palette | `src/templates/css/themes.css` | Four bespoke tokens per picker value; insufficient contract. |
| Main stylesheet | `src/templates/css/styles.css` | Consumes Shoelace tokens and contains duplicated rules. |
| Search stylesheet | `src/templates/css/search.css` | Uses Shoelace tokens and OS-only dark-mode media query. |
| Pagefind wrapper | `src/templates/js/pagefind-search.js` | Performs debounced client-side full-text search but hard-codes root-relative Pagefind paths. |
| Search modal | `src/templates/components/search-modal.js` | Presents instant results; contains a root-absolute dynamic-import fallback. |
| Search initialiser | `src/templates/js/search-init.js` | Connects header input, modal, URL query, and keyboard shortcuts. |
| Search index pipeline | `src/pipeline/mod.rs`, `src/pagefind_service.rs` | Runs Pagefind after render, yet search UI availability is checked before that step. |
| Historical Web Awesome theme | `origin/claude/create-webawesome-theme-01U5jiKf7NWSbvTo38XQHqYc:src/templates/webawesome/` | Existing alternate `wa-*` template family and Font Awesome-backed Web Awesome icon convention. |
| Theme picker | `src/templates/header.html.tera` | Advertises the five supported names. |
| Theme state | `src/templates/js/theme-switch.js` | Sets/persists `data-theme`. |
| Shadow TOC | `src/templates/components/doc-toc.js` | Imports light CSS in a shadow root. |
| Shadow blocks | `src/templates/components/simple-block.js` | Imports light CSS in a shadow root. |
| Shadow sidebar | `src/templates/components/doc-sidebar.js` | Imports light CSS/tokens in a shadow root. |
| Asset emission | `src/render/html.rs` | Embeds and writes template/vendor trees, then allows custom override files. |
| Current coverage | `tests/e2e.rs`, `tests/integration/build_test.rs` | Verifies picker persistence/offline assets, but not cross-surface palette coherence. |

### Theme and Asset Flow

```text
book.toml default/preferred theme
  -> Rust renderer inserts HTML data attributes
  -> page/index template loads Shoelace light CSS + authored CSS
  -> theme-switch.js stores choice and sets html[data-theme]
  -> themes.css updates body-only bespoke tokens
  -> authored/shadow UI still resolves Shoelace light neutral/primary tokens

HTML generation -> Pagefind index subprocess -> `pagefind/` assets
  -> browser wrapper imports root-relative `/pagefind/pagefind.js`
  -> full-text search works at a domain root but not a sub-path
```

## Constraints

### Technical Constraints

- The Shoelace bundle is deliberately minimal and locally vendored; switching to a CDN or vendoring the full 14 MB distribution would violate the established offline/build-size direction.
- Shadow DOM must receive theme values explicitly or via a loaded stylesheet; document-root custom properties do not cross into it.
- Templates are compiled into the binary and can be overridden per file by a book's `templates` directory. Default changes must preserve that override contract.
- mdBook parity requires configured `default-theme`, `preferred-dark-theme`, persistence, offline output, and sub-path-safe relative URLs.

### Non-Functional Requirements

| Requirement | Target | Current |
|---|---|---|
| Offline | Zero external runtime references | Pass; must remain pass. |
| Sub-path/file URL | Asset URLs resolve from any page depth | Pass; must remain pass. |
| Full-text search | Pagefind index and instant UI on every successful build | Not met consistently: first build hides UI; sub-path imports are root-relative. |
| Theme coherence | One palette drives every surface | Not met. |
| Accessibility | WCAG 2.2 AA contrast for normal interactive text | Unverified; likely inconsistent. |
| Cascade maintainability | One authoritative rule per component concern | Not met; duplicate style regions exist. |

## Vital Few (Essentialism)

| Constraint | Why It's Vital | Evidence |
|---|---|---|
| Semantic token contract per component family | Prevents split palettes while allowing Shoelace and Web Awesome to coexist as separate themes. | Two incompatible token systems currently coexist. |
| Theme propagation into shadow DOM | TOC/sidebar/blocks otherwise remain light. | Each imports `themes/light.css` internally. |
| Preserve local, relocatable assets | This was a key parity/offline deliverable. | `6eeccae` validates all vendor-module URLs under `/docs/`. |
| Separate selectable Shoelace and Web Awesome families | The requester explicitly asks to expand both; the historical branch establishes the boundary. | `eb61346` and `book.webawesome.toml`. |

### Eliminated from Scope

| Eliminated Item | Why Eliminated |
|---|---|
| Redesigning information architecture, grid proportions, or typography | The reported failure is colour-system coherence, not layout or content structure. |
| New colour variants beyond the five advertised mdBook names | Adds palette and test matrix without fixing the regression. |
| Replacing Shoelace globally with Web Awesome | The request is to expand both systems, not delete the current Shoelace offering. |
| Theme customizer UI | Configuration UX is separate from restoring supported defaults. |
| Mermaid diagram colour redesign | Bundled third-party diagram styling is not implicated by the reported document UI mismatch. |

## Dependencies

### Internal Dependencies

| Dependency | Impact | Risk |
|---|---|---|
| `BookConfig` theme options | Defines valid default/dark values supplied to templates. | Medium — token contract must track these values. |
| Pagefind build/index | Supplies static index assets after HTML rendering. | High — search affordance must not depend on a prior build. |
| Pagefind browser wrapper | Loads and queries the local static index. | High — hard-coded root paths violate deployment contract. |
| Tera templates | Control document CSS order and active-theme attributes. | Medium — page/index must remain equivalent. |
| Embedded static asset writer | Publishes changed assets and custom overrides. | Medium — asset paths must retain relocatability. |
| Web components | Need an explicit theme-bridge design. | High — their isolated styles otherwise stay light. |
| Search modal/Pagefind | Needs explicit-choice precedence over OS preference. | Medium. |

### External Dependencies

| Dependency | Version / delivery | Risk | Alternative |
|---|---|---|---|
| Shoelace | Locally vendored subset | Medium: color tokens are designed as global tokens, but shadow boundaries isolate them. | Retain and bridge tokens; do not replace. |
| Web Awesome | Historical beta 3.0.0 CDN delivery | High: external/beta delivery fails the current offline contract. | Vendor a minimal pinned closure, or explicitly make it a non-parity online mode. |
| Browser colour-scheme support | Native CSS/JS | Low | Explicit `data-theme` selectors with an OS-preference fallback. |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| A dark theme fixes body but not components | High | High | Test computed colours in document and every shadow root. |
| Token overrides unintentionally restyle Shoelace controls | Medium | High | Use documented component tokens and visual regression tests for buttons/input/card. |
| OS media query overrides an explicit selection | High | Medium | Make `data-theme` selectors take precedence; retain media query only as initial fallback. |
| CSS cleanup changes layout | Medium | Medium | Separate mechanical duplicate removal from semantic changes; capture layout screenshots at desktop/mobile. |
| Custom template users depend on old variables | Medium | Medium | Keep a documented compatibility alias during the release or explicitly version the contract. |
| Search appears absent on fresh deployment | High | High | Render the search affordance when Pagefind is enabled; make indexing success/failure explicit and test a clean build. |
| Search fails below a deployment prefix | High | High | Derive Pagefind and wrapper URLs from the executing module/document path and test under `/docs/`. |
| Two libraries compete on one page | Medium | High | Make theme-family selection mutually exclusive; do not load `sl-*` and `wa-*` assets/components together by default. |
| Web Awesome restore violates offline parity | High | High | Decide and prove a locally vendored, version-pinned minimal distribution before production use. |

### Open Questions

1. Is the required Web Awesome delivery mode locally vendored/offline (recommended, parity-compatible) or is an external CDN acceptable for that alternate theme? **Owner: requester.**
2. Is the required visual baseline the historical Web Awesome theme, the old Shoelace implementation, or a refreshed branded variant of each? **Owner: requester.**
3. Must existing custom template overrides that reference `--bg`, `--fg`, `--sidebar-bg`, and `--accent` remain compatible? **Owner: maintainers / release policy.**
4. Which named themes are contractual mdBook compatibility values versus choices that can be reinterpreted visually? **Owner: requester.**
5. What is the measurable meaning of “instant” for the shipped corpus (for example, first visible results within 250 ms after the debounce once the index is warm)? **Owner: requester / maintainers.**

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if Wrong | Verified? |
|---|---|---|---|
| The request requires a Shoelace repair plus an expanded Web Awesome alternate theme. | User explicitly named both and linked the historical theme branch. | The plan may underestimate Web Awesome delivery work. | Yes. |
| The five picker names remain supported. | They are rendered in the default header and accepted by configuration. | A desired compatibility reduction could simplify the plan. | Yes for current behaviour; product intent pending. |
| `data-theme` is the canonical explicit preference. | It is the persisted state and existing test assertion. | Another integration may expect class-based theming. | Yes in default templates. |
| A single authored theme token file can be exposed to shadow roots safely. | Components already load local CSS from module-relative paths. | May need a small shared theme asset rather than document-only CSS. | Needs design validation. |
| Pagefind remains the required search backend. | README and the prior parity research call it a retained local decision; requester confirmed it. | None for this scope. | Yes. |
| Search is expected on a clean successful build, not only from the second build. | Request says it used to work; hiding a functional search index is user-visible regression. | May require decoupling UI inclusion from an existing index check. | Needs design validation. |

### Multiple Interpretations Considered

| Interpretation | Implications | Why Chosen / Rejected |
|---|---|---|
| Restore historical Web Awesome unchanged | Reintroduces `wa-*` templates but depends on an external beta CDN. | Rejected unchanged; retain it as the visual/component reference but meet present offline constraints. |
| Offer Shoelace and Web Awesome as discrete template families | Restores the historical alternate while preserving Shoelace and preventing token/component conflicts. | Recommended, subject to asset-delivery decision. |
| Add standalone Font Awesome CSS to Shoelace pages | Changes icons, but does not repair token mismatch and mixes systems unnecessarily. | Rejected. |
| Keep Shoelace and map its tokens per named theme | Repairs current default surfaces while retaining current architecture. | Recommended as one of the two families. |
| Keep current bespoke variables and patch each selector | Leaves two systems and misses future surfaces. | Rejected as brittle. |
| Replace Pagefind with mdBook/elasticlunr search | Violates the explicit retained local decision and the requester clarification. | Rejected. |

## Research Findings

### Key Insights

1. This is principally a split-token and light-only-asset issue. The page-level `body` colour is the only broadly applied part of the picker palette.
2. The Shoelace primary scale is explicitly remapped to gray in `styles.css`, muting the intended accent even in otherwise light UI.
3. Dark support is structurally incomplete: the shipped dark Shoelace stylesheet exists but is never loaded by templates or components.
4. Search adds a third decision mechanism—OS media preference—rather than following the reader's explicit `data-theme` selection.
5. The historical branch confirms Web Awesome is the Font Awesome-backed theme intended by the request. It should be restored as an alternate family, not mixed into Shoelace pages.
6. Pagefind itself has not been replaced: the original full-text wrapper and modal remain. The regression is deployment/lifecycle integration—the wrapper assumes a domain-root URL, and parity's first-build gating can suppress the working UI.

### Relevant Prior Art

- `eb61346` (`feat: Add Web Awesome theme for documentation sites`): demonstrates the original component-library-led presentation before the parity conversion.
- `6eeccae` (`feat: vendor Shoelace locally and embed default assets`): establishes the local/offline/sub-path asset constraints that the repair must retain.
- `965aaeb` (`feat: CLI subcommands, relocatable output, themes and print`): introduces the current named-theme picker palette and is the origin of the split.

### Technical Spikes Needed

| Spike | Purpose | Estimated Effort |
|---|---|---|
| Palette contrast audit | Select semantic values and measure AA contrast across all five themes in both component families. | 3–4 hours |
| Shadow-token bridge prototype | Verify a shared token stylesheet or host-level token injection styles `doc-toc`, `doc-sidebar`, `simple-block`, and search modal consistently. | 2–3 hours |
| Screenshot baseline capture | Capture each named theme plus idle, loading, result, no-result and selected Pagefind states at desktop/mobile before cleanup. | 2 hours |
| Clean/sub-path Pagefind spike | Build a minimal book once, serve it at `/docs/`, and prove first-build UI and results work without root-absolute requests. | 2–3 hours |
| Web Awesome offline closure spike | Determine the minimal version-pinned Web Awesome/Font Awesome asset closure needed by the historical template and prove all component/icon assets resolve offline. | 3–5 hours |

## Recommendations

### Proceed / No-Proceed

**Proceed.** The defect is reproduced by architecture inspection and explains the screenshot. It is bounded, repairable without changing the rendering model, and has a clear compatibility-preserving path.

### Scope Recommendations

1. Rebuild the Shoelace family around `data-theme`-scoped semantic tokens, then map required component tokens to them; remove the gray-only primary override.
2. Make document, shadow component, search, syntax, and interactive states consume the same semantic tokens.
3. Restore and expand Web Awesome from the linked branch as a separately selectable template family, using a version-pinned offline-compatible asset strategy.
4. Refactor duplicated authored CSS only where needed to make each family’s token cascade deterministic; do not use the task as a visual redesign.
5. Restore Pagefind as an always-available client-side full-text search on successful builds: derive asset paths relatively, preserve debounced instant results, and style its states in both families.
6. Add computed-style, asset-resolution, contrast, first-build, sub-path-search, and screenshot coverage for both families.

### Risk-Mitigation Recommendations

- Treat the current user screenshot and an agreed historical baseline as visual acceptance fixtures.
- Gate CSS deletion with focused desktop/mobile screenshots and existing offline/sub-path suite.
- Validate localStorage inputs against the supported theme allow-list; unknown persisted values must fall back to the configured default.
- Preserve legacy bespoke variables temporarily as aliases only if custom-template compatibility is required.
- Treat Pagefind index failure as a visible build/reporting condition; do not silently ship a search-looking control that cannot return results.
- Keep Shoelace and Web Awesome templates mutually exclusive; test both against the same Pagefind and deployment matrix.

## Next Steps

If approved, Phase 2 will create `docs/plans/design-theme-token-parity.md` with the Shoelace and Web Awesome family boundaries, token schemas, offline asset strategy, selectors, shadow-DOM propagation, Pagefind repair, file-by-file edits, visual/behavioural test matrix, rollback plan, and lifecycle artefacts. No implementation will begin without subsequent approval.

## Appendix

### Evidence Examined

- User-supplied screenshot: `/Users/alex/Documents/Screenshot 2026-08-13 at 16.11.52.png`
- Current template and CSS source under `src/templates/`
- Current renderer asset-copy implementation: `src/render/html.rs`
- Theme-related commits: `eb61346`, `6eeccae`, `965aaeb`, `0374f56`, `aca695d`, `16b5d4f`
- Existing parity validation: `docs/plans/mdbook-parity-validation-report.md`

### Research Gate Checklist

- [x] Research document completed
- [x] Existing system and code locations mapped
- [x] Risks and unknowns identified
- [x] Essential questions and vital few completed
- [x] No implementation code changed
- [ ] Human approval received
- [ ] Terminology/baseline questions resolved or explicitly deferred
