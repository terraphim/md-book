# Handover: mdBook parity A–E

**Date**: 2026-08-08  
**Branch**: `task/2-summary-book-model`  
**PRs**: Gitea #7, GitHub #27 (A-only was #6/#26)

## Completed
- A: pipeline decomposition (byte-identical)
- B: SUMMARY.md book model + dual-review security fixes
- C: CLI subcommands + path resolution
- D: path_to_root, heading IDs, 404, code-copy
- E: themes, keyboard, print, redirects

## Known gaps (plan-explicit or deferred)
- Full Shoelace vendor (CDN still used for Shoelace)
- Fold interactive collapse UI (config present)
- additional-css/js file copy (config keys present; inject partial)
- P2 {{#include}} preprocessing
- Increment F pulldown-cmark

## Unrelated stash
`git stash list` may still hold SEO/template WIP — do not drop.

## Resume
1. Merge PR #7 after CI
2. Close Gitea issues #1–#5
3. Optional: vendor Shoelace, wire additional-css/js fully, fold JS
