# Specification: Shoelace and Web Awesome Theme Parity

**Status**: Review
**Canonical Path**: `docs/specs/product/theme-token-parity.spec.md`
**Change Slug**: `theme-token-parity`
**Research**: `docs/plans/research-theme-token-parity.md`
**Design**: `docs/plans/design-theme-token-parity.md`
**Interview Date**: 2026-08-13
**Dimensions Covered**: Failure recovery, integration, migration/compatibility, user experience, performance, accessibility, operational readiness
**Convergence Status**: Complete — two interview rounds resolved all high-impact ambiguities.

## Product Behaviour

1. md-book ships two mutually exclusive documentation theme families:
   - **Shoelace** remains the default family.
   - **Web Awesome** is restored as an opt-in template family, based on the historical branch `claude/create-webawesome-theme-01U5jiKf7NWSbvTo38XQHqYc`.
2. Each family supports `light`, `rust`, `coal`, `navy`, and `ayu` through one complete semantic token palette per theme.
3. A selected theme applies coherently to all visible surfaces, including document canvas, header, sidebar, TOC, cards, controls, code, Pagefind modal/results, focus indicators, and library components inside shadow DOM.
4. The document must never show the current mismatch where `coal` darkens the body but leaves the header or Pagefind modal white.
5. The existing wide-screen multi-column reading layout remains: only long prose paragraphs and lists flow across columns. Tables, code blocks, cards, diagrams, controls, and navigation remain single-column/full-width as they do today.

## Theme Delivery and Family Selection

### Shoelace

- Shoelace stays locally vendored and must remain fully usable offline, below a deployment prefix, and via local file URLs where existing parity supports them.
- Its library tokens must map from the family’s semantic palette; ad-hoc grey primary overrides are removed.

### Web Awesome

- Local, pinned, offline assets are preferred and are the release-quality delivery mode.
- If a minimal Web Awesome closure cannot be packaged cleanly, the Web Awesome family may fall back to its CDN delivery.
- CDN fallback must be explicit in the generated book/documentation as **online-only**, must not silently affect Shoelace output, and must retain a pinned version and documented provenance.
- A Web Awesome CDN fallback does not relax Shoelace’s offline guarantee.

### Compatibility

- Token changes may be breaking. One-release compatibility aliases for `--bg`, `--fg`, `--sidebar-bg`, and `--accent` are optional, not required.
- The release notes must name the break and show the semantic-token migration path for custom templates.

## Pagefind Full-Text Search

1. Pagefind remains the sole search backend and provides full-text results as the reader types.
2. On a successful build, the Pagefind trigger and modal are present in the final output from the first build; they must not require a second build.
3. The browser wrapper, Pagefind bundle, and search component imports resolve relative to their modules/output location. They must work at the domain root, under `/docs/` (or equivalent prefix), and from nested generated pages.
4. A warm search uses a 150 ms debounce and ignores stale responses; first visible results target 250 ms after the debounce on the reference corpus.
5. Keyboard behaviour:
   - `/` and Cmd/Ctrl+K open search when focus is not inside editable content.
   - Typed characters reach the modal input exactly once.
   - Arrow keys move the active result, Enter opens it, Escape closes the modal and restores focus to its trigger.
6. Query, title, and result presentation are safe from HTML injection. Only Pagefind’s documented encoded excerpts may be rendered as HTML highlights.

### Pagefind Failure Behaviour

- If Pagefind indexing fails, md-book still publishes the generated documentation.
- The final output contains **no search UI**—not a disabled trigger or a control that cannot return results.
- Build output clearly identifies indexing failure and the reason. Release/CI verification treats this as a failed search acceptance criterion.

## Accessibility and Visual Acceptance

- Normal, secondary, link, interactive, and focus text meets WCAG 2.2 AA contrast for its context.
- Theme selection, search trigger/modal, result selection, and close behaviour are keyboard accessible and announced with appropriate names/states.
- Theme state remains persisted using `md-book-theme`; unknown stored values fall back deterministically to configured defaults.
- Explicit user choice takes precedence over OS dark-mode preference.
- Chrome validation captures desktop and mobile screenshots for `light`, `coal`, and `navy`, including search idle/loading/results/no-results/selected states, for both families.

## Operational Acceptance

| Situation | Required behaviour |
|---|---|
| Clean successful build | Search UI and Pagefind index exist in final output. |
| Pagefind CLI/index failure | Book publishes, UI is omitted, build log explains failure. |
| Shoelace output | No external asset URLs. |
| Local Web Awesome output | No external asset URLs. |
| Web Awesome CDN fallback | Online-only status is visible/documented and CDN source/version is pinned. |
| `/docs/` deployment | Search returns results without root-absolute asset failures. |
| Wide desktop | Prose/list multi-column layout remains; code/tables/cards/diagrams do not enter columns. |

## Deferred Items

- The exact branded colour values are implementation/design-system work, constrained by contrast and screenshot approval.
- Web Awesome local asset closure size and packaging mechanism require the planned provenance spike.
- A new visual customiser or additional named themes is not part of this change.

## Interview Summary

The implementation must restore rather than replace the original two-family design direction: Shoelace is the reliable offline default and Web Awesome is a separately selected theme family. Web Awesome should be locally packaged when possible, while a clearly labelled, pinned CDN fallback is permitted if its minimal local closure cannot be sustained.

Pagefind is a first-class part of the rendered product. Successful first builds must contain working search, and failed indexing must never leave a non-functional search affordance in a published book. The user accepts breaking custom-token changes but requires preservation of the broad-screen prose-only multi-column reading experience.
