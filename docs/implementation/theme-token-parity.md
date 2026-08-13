# Implementation Report: Theme Token Parity

**Change slug:** `theme-token-parity`
**Date:** 2026-08-13
**Approved inputs:** research, design, specification, and user decisions in the
current task

## Outcome

The implementation restores the intended two-family design. Shoelace remains
the offline default; Web Awesome is opt-in, isolated, pinned, and explicitly
online-only while it uses the approved CDN fallback. Both families share five
coherent semantic themes and preserve the wide-screen multi-column reading
layout without mobile overflow.

Pagefind is again a working product feature: a successful first build includes
the index and search controls, while an indexing failure removes stale search
assets and publishes no search UI. Search uses Pagefind's debounced API at 150
ms, ignores stale responses, works from nested pages, and renders authored
metadata safely.

## File-level implementation

- `src/core.rs`, `src/pipeline/mod.rs`: two-pass first build, explicit Pagefind
  success/failure outcome, stale-index cleanup, and search-free failure render.
- `src/render/html.rs`: mutually exclusive Shoelace/Web Awesome asset output.
- `src/templates/css/themes.css`, `src/templates/css/search.css`,
  `src/templates/css/styles.css`: complete semantic palettes, explicit-theme
  authority, component inheritance, wide columns, and mobile containment.
- `src/templates/js/pagefind-search.js`, `src/templates/js/search-init.js`,
  `src/templates/components/search-modal.js`: relative Pagefind loading, native
  debounce, stale-response suppression, shadow-input handling, safe rendering,
  keyboard state, ARIA state, and focus restoration.
- `src/templates/webawesome/`: restored opt-in templates, Web Awesome components,
  family-specific CSS/tokens, and current `start`/`end` slot conventions.
- `book.webawesome.toml`: runnable selection example and online-only disclosure.
- `tests/integration/build_test.rs`: first-build search, failure cleanup, family
  isolation, token inheritance, relative paths, debounce, and accessibility
  regressions.
- `README.md`, `CHANGELOG.md`: selection, operational behavior, and breaking
  semantic-token migration notes.

## Implementation deviations found and corrected

Chrome validation found and drove fixes for:

1. Web Awesome's native shadow input did not propagate typed values to the
   search handler.
2. Mobile grid min-content sizing caused horizontal overflow at 390 px.
3. Web Awesome dark search inputs retained a white internal surface.
4. Historical Web Awesome icon slots used Shoelace names.
5. Keyboard-selected results lacked announced `aria-selected` state.

All five were corrected and revalidated in Chrome.

## Verification entry criteria

- `cargo fmt --check`: pass.
- `cargo clippy --all-targets --all-features -- -D warnings`: pass.
- `cargo test --all-features`: pass (177 passed, 1 pre-existing ignored).
- Changed JavaScript syntax checks: pass.
- Chrome UAT: pass for both families at 1600×1000 and 390×844.
