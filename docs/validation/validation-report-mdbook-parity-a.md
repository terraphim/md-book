# Validation Report: mdBook parity Increment A

**Date**: 2026-08-08  
**Change slug**: mdbook-parity-a  
**Issue**: terraphim/md-book#1

## Acceptance criteria (from plan gate)

| Criterion | Evidence | Status |
|-----------|----------|--------|
| `make ci-local` / QA green | `make qa` + clippy/tests as in verification report | Pass |
| Output byte-identical to pre-A | SHA-256: 72/72 match | Pass |
| No behaviour change for existing books | Directory-walk path unchanged; `"Guide"` section retained for A | Pass |
| Preprocess seam ready for P2 | Identity function called before mdast parse | Pass |

## User-visible behaviour

None intended. Users building with `-i/-o` see the same HTML, sidebar, prev/next, and assets as before.

## Deferred product validation

- Structural SUMMARY-driven navigation → Increment B  
- CLI subcommands → Increment C  
- Relocatable/offline output → Increment D  
- Themes/print → Increment E  

## Sign-off

Automated gate evidence complete. Stakeholder UAT not required for a pure refactor with byte-identical output.
