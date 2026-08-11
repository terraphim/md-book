# Structural review: mdBook parity Increment A

**Date**: 2026-08-08  
**Branch**: `task/1-pipeline-decomposition`  
**Scope**: Pipeline decomposition only

## Findings

No P0/P1 issues found.

### Notes (informational)

1. **Module cycle is intentional and safe**: `pipeline` uses `core::{Args, PageInfo}`; `core` calls `pipeline::run_sync`. Same-crate cycle; no runtime risk.
2. **`write_syntax_css` still hard-codes Solarized (light)** — deferred to Increment E by plan; left with TODO comment.
3. **`"Guide"` section title** — deferred to B; intentionally preserved for byte-identical gate.
4. **`render_markdown` takes `Option<&SyntaxSet>`** — feature-gated; clean call sites with `cfg`.

## Security

No new attack surface. No path handling changes beyond move of existing walkdir logic.

## Performance

No algorithmic change. Wall clock ~0.86 s for `test_book_mdbook` (debug).

## Verdict

Approve for merge after CI green on PR.
