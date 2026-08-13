# Implementation Plan: Shoelace and Web Awesome Theme Parity

**Status**: Review
**Canonical Path**: `docs/plans/design-theme-token-parity.md`
**Change Slug**: `theme-token-parity`
**Research**: `docs/plans/research-theme-token-parity.md`
**Author**: Codex
**Date**: 2026-08-13
**Estimated Effort**: 4–6 engineering days, excluding design approval and any upstream Web Awesome asset packaging uncertainty

## Overview

### Summary

Restore a coherent, accessible, offline theme system around Shoelace and add Web Awesome back as a separately selectable template family. Repair Pagefind so full-text results are available on the first successful build and work at a domain root, beneath a deployment prefix, and from nested pages.

### Validated Baseline

Chrome validation against a clean local Pagefind-indexed build showed the fault directly: after selecting `coal`, the body was `rgb(20, 22, 23)` but the header and search modal remained `rgb(255, 255, 255)`. The Pagefind modal was present, but its results interaction did not produce a visible result during this baseline check. These are acceptance failures to close, not speculative changes.

### Approach

1. Make each component family own one semantic colour contract and map its library tokens to it.
2. Ship Shoelace as the default family; restore Web Awesome from `origin/claude/create-webawesome-theme-01U5jiKf7NWSbvTo38XQHqYc` as a distinct `paths.templates` family.
3. Keep all third-party assets local and version-pinned. Never load Shoelace and Web Awesome on the same rendered page.
4. Render the Pagefind UI before the index is created, then index generated HTML; derive all browser module paths from `import.meta.url` so deployed subpaths work.
5. Verify with Rust tests, JavaScript tests, and Chrome browser screenshots/interactions at the root and `/docs/` prefix.

### Scope

**In Scope:**

- Shoelace semantic tokens, dark/light palette consistency, shadow-DOM token propagation, and CSS deduplication.
- An offline-capable Web Awesome template family based on the linked historical branch.
- Pagefind first-build UI, module-relative asset resolution, instant debounced results, accessibility, and theme states.
- Desktop/mobile visual regression and contrast acceptance criteria.

**Out of Scope:**

- New theme names beyond `light`, `rust`, `coal`, `navy`, and `ayu`.
- Replacing Pagefind with elasticlunr or another search backend.
- Redesigning document information architecture, typography, Markdown rendering, or Mermaid diagrams.
- Mixing `sl-*` and `wa-*` components in one template family.

**Avoid At All Cost:**

- A CDN or unpinned beta dependency in the default/offline build path.
- Per-component ad-hoc colours that bypass the semantic palette.
- A first-build search box backed by no index, or successful indexing that leaves the search UI absent.
- CSS cleanup unrelated to an observed duplicate/cascade conflict.

## Architecture

```text
book.toml
  paths.templates ───────> Shoelace default templates OR Web Awesome templates
  output.html.theme ─────> html[data-theme] semantic palette

active template family
  ├─ document stylesheet maps semantic tokens -> library tokens
  ├─ template-specific web components consume the same shared tokens
  ├─ Pagefind modal inherits family semantic tokens
  └─ only that family's local vendor asset closure is linked

render pass (search UI present)
  -> Pagefind CLI indexes HTML into output/pagefind/
  -> module-relative Pagefind wrapper imports output/pagefind/pagefind.js
  -> Pagefind debouncedSearch() -> accessible result modal
```

### Component Boundaries

| Component | Responsibility | Boundary |
|---|---|---|
| Shared semantic palette | Defines surfaces, text, borders, accent, focus, selection, code, and overlay values for the five names. | Library-neutral custom properties only. |
| Shoelace family | Maps semantic palette to Shoelace and styles `sl-*` templates/components. | Default template directory and local Shoelace closure. |
| Web Awesome family | Maps semantic palette to `--wa-*` tokens and styles `wa-*` templates/components. | `src/templates/webawesome/` and local Web Awesome closure. |
| Search core | Loads Pagefind relative to its module, debounces, normalises result records. | No library-specific DOM or colours. |
| Search presentation | Renders input/modal/results with active family components/tokens. | One presentation implementation per family if component APIs differ. |
| Pipeline | Renders UI, runs Pagefind once, reports index availability/failure. | Does not know colour or component tokens. |

### Key Decisions

| Decision | Rationale | Alternatives Rejected |
|---|---|---|
| Two mutually exclusive template families | Preserves Shoelace while restoring the historical Web Awesome product. | Loading both libraries or global migration. |
| Semantic tokens first, library mappings second | Makes palettes coherent and allows each library to consume its documented token surface. | Four bespoke body-only variables or per-selector colours. |
| Locally vendor a minimal, locked Web Awesome closure | Matches the existing offline/sub-path guarantee and Shoelace approach. | Historical `early.webawesome.com` CDN. |
| Render Pagefind UI before indexing | Pagefind itself recommends adding UI before static generation; it exists in the final output after the same build. | Checking an index from a prior build and hiding the UI. |
| Module-relative Pagefind paths | Works at root, nested pages, `file://`, and deployment prefixes. | Root-absolute `/pagefind/` and `/js/` URLs. |
| Use Pagefind’s `debouncedSearch` | It prevents stale searches and preloads internally without a second competing debounce. | Current bespoke timeout plus `preload` calls. |

## Expected Lifecycle Artefacts

| Artefact | Path | Required? |
|---|---|---|
| Specification | `docs/specs/product/theme-token-parity.spec.md` | Yes — user-visible behaviour and compatibility need a precise contract. |
| Decision | `docs/decisions/D-2026-001-webawesome-offline-distribution.md` | Yes — locks Web Awesome asset provenance, version, and update process. |
| ADR | `docs/decisions/adr/ADR-001-template-family-boundaries.md` | Yes — durable two-family architecture. |
| Contract | N/A | No external executable API changes. |
| Verification | `docs/verification/verification-report-theme-token-parity.md` | Yes. |
| Traceability | `docs/verification/traceability-matrix-theme-token-parity.md` | Yes. |
| Validation | `docs/validation/validation-report-theme-token-parity.md` | Yes — visible release change. |

## Token and Interface Design

### Shared Palette Contract

Each theme selector defines these semantic tokens, with no literal colours permitted in family component rules except the palette declarations themselves:

```css
html[data-theme] {
  --docs-surface-canvas: ...;
  --docs-surface-raised: ...;
  --docs-surface-sunken: ...;
  --docs-text-primary: ...;
  --docs-text-secondary: ...;
  --docs-text-link: ...;
  --docs-border: ...;
  --docs-accent: ...;
  --docs-accent-contrast: ...;
  --docs-focus-ring: ...;
  --docs-selection: ...;
  --docs-code-surface: ...;
  --docs-overlay: ...;
}
```

`light` is the default selector. `rust`, `coal`, `navy`, and `ayu` receive complete values, including raised/sunken surfaces and focus contrast. Existing `--bg`, `--fg`, `--sidebar-bg`, and `--accent` remain aliases for one release only when custom-template compatibility analysis identifies use.

### Shoelace Mapping

`src/templates/css/themes.css` maps semantic tokens to the necessary `--sl-color-neutral-*`, `--sl-color-primary-*`, panel/input, and focus tokens. It loads both upstream light/dark Shoelace foundation files only if their required structural tokens differ; otherwise generated semantic mapping is authoritative. The current `styles.css` gray-primary override is removed.

Shadow components load a new module-relative shared token stylesheet and set `:host` variables from the active document attribute via an explicit synchronisation mechanism. The implementation must not rely on document custom-property inheritance across a shadow boundary.

### Web Awesome Mapping

`src/templates/webawesome/css/themes.css` defines the same `--docs-*` interface and maps it to documented `--wa-*` component tokens. Web Awesome components, icons, modal, TOC, sidebar, cards, inputs and buttons are adapted from the historical branch. A dedicated asset inventory records the exact source archive/package version, hashes, component transitive imports, and icon set used.

### Theme State API

`theme-switch.js` becomes a small module with a fixed allow-list:

```js
export const SUPPORTED_THEMES = Object.freeze(['light', 'rust', 'coal', 'navy', 'ayu']);
export function resolveTheme({ stored, defaultTheme, preferredDarkTheme, prefersDark }) { /* supported value or fallback */ }
export function applyTheme(theme) { /* set html[data-theme], colour-scheme, ARIA state */ }
```

The existing `md-book-theme` storage key remains. Invalid persisted values fall back to the configured default or preferred dark value. Explicit selection takes precedence over OS colour preference.

### Pagefind API

`src/templates/js/pagefind-search.js` becomes an ESM module:

```js
export class PagefindSearch {
  constructor({ maxResults = 10, baseUrl = document.baseURI } = {}) { /* ... */ }
  async initialise() { /* import new URL('../pagefind/pagefind.js', import.meta.url) */ }
  async search(query) { /* Pagefind debouncedSearch(query, {}, 150) */ }
  destroy() { /* Pagefind destroy if initialised */ }
}
```

The wrapper calls `pagefind.options()` before initialise with the resolved Pagefind `basePath` and output-relative base URL, using Pagefind’s Search API option names. It returns a typed-normalised `{ query, results, totalResults }` object and ignores `null` stale responses from `debouncedSearch`.

`search-modal.js` statically imports the wrapper using `new URL('../js/pagefind-search.js', import.meta.url)` or an equivalent standard ESM relative import. It removes the root-absolute dynamic fallback. It must use text-safe DOM construction for raw title/content fields; Pagefind-provided encoded excerpts may be rendered using the documented safe excerpt path.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/templates/css/theme-tokens.css` | Shoelace family semantic palettes and compatibility aliases. |
| `src/templates/css/theme-bridge.js` | Synchronises active semantic tokens/attribute into shadow components. |
| `src/templates/webawesome/**` | Restored alternate templates, family CSS, JS, components, image assets. |
| `src/templates/vendor/webawesome/**` | Pinned minimal offline Web Awesome and Font Awesome asset closure plus manifest. |
| `docs/specs/product/theme-token-parity.spec.md` | Behavioural theme/search specification. |
| `docs/decisions/D-2026-001-webawesome-offline-distribution.md` | Third-party provenance/upgrade decision. |
| `docs/decisions/adr/ADR-001-template-family-boundaries.md` | Two-family architecture decision. |
| `tests/assets/theme_book/**` | Small reproducible book for family, palette, Pagefind, and prefix tests. |

### Modified Files

| File | Changes |
|---|---|
| `src/templates/css/themes.css` | Replace four-variable body override with Shoelace token mappings; import/shared token contract. |
| `src/templates/css/styles.css` | Remove gray-primary remap and duplicate blocks; consume only semantic/library tokens. |
| `src/templates/css/search.css` | Replace OS-only dark rules with explicit theme selectors and semantic tokens. |
| `src/templates/page.html.tera` | Link only active family assets and shared theme machinery; retain relative asset URLs. |
| `src/templates/index.html.tera` | Same family/token/search contract as chapter pages. |
| `src/templates/header.html.tera` | Keep accessible picker; expose selected name and family-consistent icons. |
| `src/templates/components/doc-toc.js` | Receive active token bridge; remove hard-coded light stylesheet. |
| `src/templates/components/doc-sidebar.js` | Same shadow-token bridge. |
| `src/templates/components/simple-block.js` | Same shadow-token bridge. |
| `src/templates/components/search-modal.js` | ESM Pagefind import, accessible states, stale-search handling, safe rendering. |
| `src/templates/js/theme-switch.js` | Allow-list, initial choice resolution, `color-scheme`, event synchronisation. |
| `src/templates/js/pagefind-search.js` | ESM export, module-relative Pagefind location, Pagefind debounced API. |
| `src/templates/js/search-init.js` | Remove duplicate shortcut ownership and pass full query to modal without losing characters. |
| `src/render/html.rs` | Emit Web Awesome vendor tree when its template family requests it; retain default/custom override order. |
| `src/pipeline/mod.rs` | Render search UI as part of first build; surface Pagefind outcome without prior-index gating. |
| `src/pagefind_service.rs` | Return a structured indexing outcome to pipeline callers rather than swallowing failures. |
| `tests/integration/build_test.rs` | Cover family asset closure, first-build search markup, and no root-absolute search URLs. |
| `tests/e2e.rs` / JS frontend tests | Cover switching, shadow components, Pagefind debounce/results, keyboard navigation and safe rendering. |
| `README.md`, `book.webawesome.toml` | Document both families, offline requirement, and Pagefind behaviour. |

### Deleted Files / Rules

| Target | Reason |
|---|---|
| Duplicated regions in `styles.css` | They cause cascade ambiguity; retain one verified canonical rule per concern. |
| Root-absolute `/pagefind/` and `/js/pagefind-search.js` imports | They break sub-path deployment. |
| OS-only search dark mode block | Explicit theme selection must control search presentation. |

## Implementation Steps

### Step 1: Lock baseline, provenance, and specification

**Files:** lifecycle artefacts; `tests/assets/theme_book/**`

- Capture Chrome screenshots and computed styles for `light`, `coal`, `navy`, and one mobile viewport.
- Record Web Awesome branch commit, upstream version/source, license, hashes, components and icons needed.
- Write behavioural specification: visible theme state, contrast target, Pagefind first-build/root/subpath contract, and mutually exclusive family selection.

**Tests:** Baseline artefacts include the observed `coal` dark-canvas/light-header defect and Pagefind modal behaviour.

### Step 2: Repair Shoelace token contract

**Files:** `theme-tokens.css`, `themes.css`, `styles.css`, `search.css`, Shoelace templates/components, `theme-switch.js`

- Define all five palettes and aliases.
- Map semantic values to Shoelace tokens; remove gray-primary mapping.
- Replace literals and OS-only theme branching with semantic selectors.
- Remove duplicates only after every retained rule is assigned to one canonical section.
- Bridge the selected token contract into each existing Shoelace shadow component.

**Tests:** CSS/token assertion tests, invalid-localStorage fallback test, computed-style checks for document, card, header, sidebar, TOC, code, modal and focus ring.

### Step 3: Restore Web Awesome as an offline template family

**Files:** `src/templates/webawesome/**`, `src/templates/vendor/webawesome/**`, renderer asset emission, `book.webawesome.toml`

- Bring the historical family forward without importing Shoelace templates/components.
- Vendor only the dependency closure used by the restored template, with a checked-in manifest and license notices.
- Re-map `--docs-*` to `--wa-*`; make Pagefind presentation and theme picker conform to the same behavioural specification.
- Add `book.webawesome.toml` as a supported example, not a forked build path.

**Tests:** Build both family fixtures with zero external URLs, all referenced assets present, and no `sl-*` assets in Web Awesome output or `wa-*` assets in default output.

### Step 4: Repair Pagefind lifecycle and path resolution

**Files:** `pagefind_service.rs`, `pipeline/mod.rs`, `pagefind-search.js`, `search-modal.js`, `search-init.js`, templates

- Make initial rendering include Pagefind controls when the search feature/configuration is enabled.
- Run Pagefind after that render as required by Pagefind’s static-generator model; report indexing failure clearly in build output and tests.
- Convert Pagefind wrapper and modal imports to ESM module-relative URLs; configure `basePath`/base URL before initialise.
- Replace independent timeout logic with `pagefind.debouncedSearch(query, {}, 150)` and cancel/ignore stale responses.
- Ensure all keyboard shortcuts have a single owner, one typed character reaches the modal input once, modal close restores focus, and results use semantic colours.

**Tests:** First clean build has controls plus index; result query returns title/excerpt/link; root and `/docs/` prefix contain no root-absolute search module requests; stale searches never overwrite newer results.

### Step 5: Verification and release readiness

**Files:** verification/traceability/validation documents; README

- Run Rust fmt, clippy with warnings denied, unit/integration tests, and JavaScript test suite.
- Use Chrome extension at desktop and mobile breakpoints to test each family/theme state, Pagefind interaction, and browser console/network errors.
- Re-run existing offline and sub-path parity suite; compare resulting screenshots to approved baselines.
- Publish verification, traceability, and UAT report with all acceptance criteria mapped.

## Test Strategy

| Layer | Test | Acceptance |
|---|---|---|
| Rust unit | Pagefind index outcome | Index success/failure is observable; no silent success claim. |
| Rust integration | Fresh build | Search controls and `pagefind/` assets exist after one successful build. |
| Rust integration | Family assets | Each family emits exactly its vendor closure with no external URLs. |
| JS unit | Theme resolver | Stored invalid/unsupported themes fall back deterministically. |
| JS unit | Search wrapper | Resolves Pagefind relative to module; stale debounced response is ignored. |
| JS integration | Modal | Typing returns Pagefind result, arrow/Enter/Escape work, focus returns to trigger. |
| Browser Chrome | Shoelace `coal` | Canvas, header, modal, cards, TOC and code are dark/coherent; no light white panel remains. |
| Browser Chrome | Web Awesome | `wa-*` controls render, colours match selected palette, no Shoelace module is requested. |
| Browser Chrome | Pagefind root and `/docs/` | Query `configuration` produces results; no 404/root-absolute resource error. |
| Visual | Desktop 1440px and mobile 390px | Approved screenshots for light/coal/navy and search states. |
| Accessibility | axe + contrast | Zero new violations; WCAG 2.2 AA for normal/interactable text and visible focus. |

### Performance Targets

| Metric | Target | Measurement |
|---|---|---|
| Search warm-result response | First visible result within 250 ms after a 150 ms debounce on reference corpus | Chrome performance timing. |
| Search cold readiness | Modal shows a non-blocking loading state; no hung state | Chrome interaction test. |
| Extra default output size | No material increase beyond token CSS/targeted tests | Compare default Shoelace output before/after. |
| Web Awesome output | Minimal closure documented and justified; no full distribution by default | Asset manifest/size check. |

## Rollback Plan

1. Keep the Shoelace family as the default throughout; Web Awesome is opt-in via `paths.templates`.
2. If a palette fails accessibility or visual validation, revert only its semantic values and retain the common token architecture.
3. If Web Awesome closure proof fails, do not enable/document that family as production-ready; retain the existing default and the provenance decision draft.
4. If Pagefind indexing fails, fail the verified-release gate and report the error; do not claim search availability.

## Approval

- [x] Approved research referenced
- [x] Five-or-fewer major implementation concerns: token contract, Shoelace, Web Awesome, Pagefind, verification
- [x] File changes and test strategy specified
- [x] Chrome validation plan and baseline evidence included
- [ ] Web Awesome local-vs-CDN choice confirmed (plan assumes local/offline)
- [ ] Technical review complete
- [ ] Human approval received
