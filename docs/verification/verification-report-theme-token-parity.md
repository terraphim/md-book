# Verification Report: Theme Token Parity

**Date:** 2026-08-13
**Environment:** macOS, local debug profile, Pagefind CLI installed, Chrome
extension against localhost builds

## Automated gates

| Gate | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --check` | PASS |
| Rust lint | `cargo clippy --all-targets --all-features -- -D warnings` | PASS, zero warnings |
| Full suite | `cargo test --all-features` | PASS: 177 passed, 0 failed, 1 pre-existing ignored |
| Focused regressions | `cargo test --test integration --all-features` | PASS: 49/49 |
| JavaScript syntax | `node --check` on changed search/theme/modal modules | PASS |
| Patch hygiene | `git diff --check` | PASS |
| UBS JavaScript | reduced scan of 11 changed modules | PASS with review: 0 critical; generic template/DOM warnings reviewed |
| UBS Rust | signed module verification | BLOCKED BY TOOL: checksum mismatch; not bypassed |

The first sandboxed full-suite run failed only because the live-reload test was
not permitted to bind `127.0.0.1:0`. The same command passed completely when
rerun with localhost binding authorized.

## Code-review findings

- Critical: 0.
- Important: 0 remaining.
- Corrected during review: shadow-input event delivery, mobile grid overflow,
  Web Awesome dark input part styling, Web Awesome slot names, and ARIA selected
  state.
- Security: titles and URLs use `textContent`; Pagefind's documented encoded
  excerpt is the only result field inserted as HTML so its `<mark>` highlights
  survive. No `eval` or dynamic code construction is used.
- Dependency advisory tools (`cargo audit`, `cargo deny`) are not installed;
  no dependency was added by this change.

## Performance evidence

- Real Pagefind indexing of the 30-page reference corpus: 69–113 ms across
  repeated local debug builds.
- Chrome warm result update: 221 ms from query entry through the configured
  150 ms debounce, within the 250 ms target.
- No new Rust hot loop, dependency, or unbounded allocation was introduced.

## Browser matrix

| Family | Viewport | Themes/states | Result |
|---|---:|---|---|
| Shoelace | 1600×1000 | light/rust/coal/navy/ayu; page/header/sidebar/TOC/modal; columns | PASS |
| Web Awesome | 1600×1000 | light/rust/coal/navy/ayu; page/header/sidebar/TOC/modal; columns | PASS |
| Shoelace | 390×844 | coal; modal/results; no horizontal overflow | PASS |
| Web Awesome | 390×844 | navy; modal/results; dark input; no horizontal overflow | PASS |
| Both | nested page | Pagefind query `markdown`, five visible results | PASS |
| Shoelace | keyboard | `/`, ArrowDown, Escape, named dialog/listbox, `aria-selected` | PASS |

## Decision

Verification status: **PASS**. The UBS Rust tool-integrity failure is documented
but non-blocking because the compiler, Clippy, full suite, and focused
regressions all pass and the change adds no unsafe code.
