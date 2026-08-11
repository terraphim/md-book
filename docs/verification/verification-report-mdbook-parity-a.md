# Verification Report: mdBook parity Increment A

**Date**: 2026-08-08  
**UTC**: 09:57:29 UTC  
**Change slug**: mdbook-parity-a  
**Issue**: [terraphim/md-book#1](https://git.terraphim.cloud/terraphim/md-book/issues/1)  
**Branch**: `task/1-pipeline-decomposition`  
**Plan**: `docs/plans/mdbook-parity-implementation-plan.md` (Increment A)

## Scope verified

| Step | Description | Status |
|------|-------------|--------|
| A1 | Extract `collect` / `render` / `index` into `pipeline/` + `render/` | Pass |
| A2 | Identity `preprocess` seam with unit tests | Pass |
| A3 | Baseline build timing + byte-identical gate | Pass |

## Checks run

```text
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib --bins
cargo test --test integration --features "tokio,search,syntax-highlighting"
cargo test --test mdbook_compatibility
cargo test --test mdbook_test_book
make qa
```

### Results

| Suite | Result |
|-------|--------|
| Unit (`--lib --bins`) | 33 passed, 1 ignored |
| Integration `build_test` | 12 passed |
| `mdbook_compatibility` | 5 passed |
| `mdbook_test_book` | 8 passed |
| Clippy `-D warnings` | Clean |
| `make qa` | Clean |

## Byte-identical gate

Pre-A and post-A builds of:

- `tests/assets/test_book_1`
- `test_book_mdbook/src`

compared via SHA-256 over all HTML/CSS/JS assets (excluding `pagefind/` and the hash manifest itself).

```text
match=72 diff=0 missing=0
```

Gate satisfied: output is byte-identical to the pre-A build.

## A3 performance baseline

| Metric | Value | Notes |
|--------|-------|-------|
| Full build wall time, `test_book_mdbook` (31 pages) | **0.86 s real** | `cargo run` debug binary; Pagefind CLI absent |
| User CPU | 0.35 s | |
| Sys CPU | 0.17 s | |
| Pagefind index | N/A | `pagefind` CLI not installed; step fails gracefully (pre-existing) |

Later increments must stay within 10% of this wall time on the same corpus (plan target). Criterion `benches/pagefind_bench.rs` remains the Pagefind-specific bench; re-run when the CLI is available.

## Traceability

| Plan requirement | Evidence |
|------------------|----------|
| `core::build` is orchestration only | `src/core.rs` delegates to `pipeline::run_sync` (+ async `pipeline::index`) |
| Stages: collect → preprocess → render → index | `src/pipeline/mod.rs` |
| Preprocess identity | `src/pipeline/preprocess.rs` + 3 unit tests |
| Markdown rendering extracted | `src/render/markdown.rs` |
| HTML/Tera assembly extracted | `src/render/html.rs` |
| Existing suite untouched in intent | All prior tests still pass (some relocated with their code) |

## Out-of-scope guard

Unrelated WIP (SEO description/canonical, browser-validation plans, `crates/`, template CSS tweaks) remains in `git stash` and was **not** committed on this branch.

## Verdict

**Pass** — Increment A ready for review and PR.
