# Handover: mdBook parity (increments A–E, G) — 0.2.0 released

**Date**: 2026-08-12
**UTC Time**: 11:37:17 UTC
**Change Slug**: `mdbook-parity`
**Branch**: `main` at `fdc4625`
**Prior handover**: `.agent/handoffs/2026-08-08-mdbook-parity-a.md` (increment A, branch stage)
**Session file**: `.agent/sessions/2026-08-08-mdbook-parity-a.md`

## Progress Summary

**Completed.** The full V-model ran end to end: research → design → specification interview →
implementation (A–E, G) → verification → validation → release. md-book **0.2.0** is on `main`,
CI is green, and the only downstream consumer (`docs.terraphim.ai`, built from
`terraphim-ai/docs`) is deployed and verified on it.

Increments delivered:

| Inc | Content |
|-----|---------|
| A | Pipeline decomposition (collect / preprocess / render / index), byte-identical output |
| B | `SUMMARY.md` book model — prefix/suffix chapters, part titles, drafts, separators, numbering |
| C | Subcommands (`build`, `serve`, `watch`, `init`) and mdBook-compatible path resolution |
| D | `path_to_root` relocatability, embedded assets, stable heading IDs, 404 page, code-copy |
| E | Themes, keyboard shortcuts, print page, redirects, sidebar folding, conditional mermaid |
| G | Page description, canonical URL, skip link, search UI gated on a real index |

**State**: released and in production. Nothing is in flight; no branch is open.

## Artefact Index

| Phase | Artefact |
|-------|----------|
| Research (1) | `docs/plans/mdbook-parity-research.md` |
| Design (2) + Spec (2.5) | `docs/plans/mdbook-parity-implementation-plan.md` (spec findings appended as "Specification Interview Findings — Increment B") |
| Structural review | `docs/plans/review-mdbook-parity-a.md` |
| Verification (4) | `docs/plans/mdbook-parity-verification-report.md` |
| Validation (5) | `docs/plans/mdbook-parity-validation-report.md` |
| Security exception | `.cargo/audit.toml` (RUSTSEC-2026-0194/0195) |
| Operational continuity | this file; prior handover `2026-08-08-mdbook-parity-a.md` |

No `decisions/` or `contracts/` directories exist in this repo; design decisions live in the
implementation plan's "Key Design Decisions" table.

**External continuity**: `~/cto-executive-system/memory/handoffs/PENDING.yaml` was **not** updated
— it currently holds an unrelated pending handover (terraphim-migration Phase 4) and is a
single-slot buffer, not a queue. Overwriting it would destroy that content. The repo-local
artefact is canonical.

## Current State

### Known-good (verified this session, not asserted)

| Fact | Evidence |
|------|----------|
| 0.2.0 on `main`, GitHub and Gitea identical | both at `fdc4625` |
| CI green | latest `CI` run on `main` = `success`, 19/19 jobs |
| Tests | 174 (96 unit, 46 integration, 12 e2e, 4 structure, 16 mdBook conformance) |
| Line coverage | 87.1% (`cargo llvm-cov`) |
| Open Gitea issues | 0 |
| Live docs site | 575 pages; `/`, `/PERFORMANCE_BENCHMARKING_README.html`, `/src/Architecture.html`, `/src/SUMMARY.html` all 200 |
| Index layout | live HTML has `index-container` + `card-grid`, no `class="sidebar"` |
| Accessibility | 0 axe violations on chapter, index and 404 |

### Partially working / accepted gaps

- **`{{#include}}` is not implemented** (SC3). Closed deliberately on evidence: 0 uses across the
  129 files of `terraphim-ai/docs`. This is why 0.2.0 is a minor, not a major — see the validation
  report's Release Readiness section.
- **`mathjax-support`** parses from `book.toml` but does nothing. `terraphim-ai/book.toml` sets it,
  so it reads as supported and is not. Either implement or reject the key loudly.
- **`render/markdown.rs` function coverage 55.6%** (D-008) — feature-gated branches untested.
- **`quick-xml` advisories** documented, not fixed; reachable only via `syntect → plist` parsing
  files md-book itself ships. Revisit when `plist` takes its semver-major bump.

### Risky

- **UBS static analysis cannot run** (D-007). `ubs doctor` verifies js/python/cpp/golang and fails
  only on rust: three distinct digests exist (installer pin `5c0df5f4…`, upstream `08e99d1e…`,
  July cache `26249823…`) because `ubs` fetches modules from an unpinned `master` while pinning
  digests in a released installer. **The integrity check was not disabled.** Substitute evidence
  (ast-grep + clippy) is in the verification report. Upstream issue — do not "fix" locally.
- **Branch ruleset requires 1 approving review and the sole author cannot self-approve.** Every
  merge needs a bypass. Three commits (`94ca7bf`, `3704b0d`, `8bcc6b7`) were pushed directly to
  `main` during this work; the user decided to leave them and use PRs from now on. Honour that.
- **Plaintext tokens in `terraphim-ai`'s git remotes** (`origin`, `gitea-private`). Not md-book's
  repo, but it is a live credential exposure and should be rotated.

## Resume Procedure

```bash
cd /Users/alex/projects/terraphim/md-book
git fetch --all && git log -1 --oneline        # expect fdc4625 on main
git status --short                             # expect clean

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --features "tokio,search,syntax-highlighting"   # expect 174 pass

# smoke-test the real consumer's book end to end
cargo run --release -- build ../terraphim-ai/docs -d /tmp/mdbook-smoke
python3 -m http.server -d /tmp/mdbook-smoke 8099   # index must show card-grid, no sidebar

gh run list --branch main --workflow CI --limit 1  # expect success
gtr list-issues --owner terraphim --repo md-book --state open   # expect none
```

Then read, in this order: the validation report's "Conditions on Approval" (what is still open and
why), then the verification report's defect register (17 entries with origin phase).

## Next Steps

1. **Immediate**: nothing is required. The release is done and verified. If the next session picks
   this up, start from the two open defects (D-007 external, D-008 accepted), not from new features.
2. **Follow-up**: decide `mathjax-support` — implement it or reject the config key with an error.
   It is currently a silent lie to `terraphim-ai/book.toml`.
3. **Deferred**: `{{#include}}`, which is the gate on ever calling this a major release. Alongside
   it, `serve` deserves e2e coverage beyond the unit-level `warp::test` suite.
4. **Housekeeping**: `git stash drop stash@{0}` (SEO work, fully superseded — the research doc's
   correction note explains why it was read from a dirty tree).

## Open Questions and Risks

- **Does the docs site want to be everything under `docs/`, or a curated subset?** Today it is
  everything: 575 pages, URLs shaped `/src/<Chapter>.html` because the workflow builds with
  `-i . -o book` from `docs/`. Switching to a `SUMMARY.md`-driven build would cut it to ~59 pages
  **and flatten every URL**, breaking inbound links. If that is ever wanted it is a deliberate
  content decision with redirects, not a build-flag change. This is exactly the mistake made and
  reverted this session (terraphim-ai PR #960 → reverted by #961).
- Will the branch ruleset be adjusted, or will every merge keep needing a bypass?

## Notes for the Next Session

Things that cost real time to discover and must not be rediscovered:

1. **Tera autoescape only applies to templates whose registered name ends in `.html`.** Templates
   were registered as `page.html.tera` etc., so autoescape was inactive **everywhere** — a critical
   HTML-injection hole via authored `SUMMARY.md` labels (D-005) that two of my own 5/5 structural
   reviews missed and an independent `pi-rust` review caught. Registration is now
   `("page.html", "page.html.tera")`. Do not "tidy" those names.
2. **Route order in `server.rs` matters**: the catch-all `fs::file` fallback matched `/live-reload`,
   so the WebSocket upgrade was unreachable and live reload had **silently never worked**. It is now
   `reload.or(static_files)`. Found only by writing the server tests (0% → 86.7%).
3. **Path containment anchors on the book root, not `src/`** — anchoring on `src/` rejected
   legitimate books with a file beside `book.toml` (D-011). Output containment is separate:
   `source_to_output` keeps only `Component::Normal`.
4. **`.index-container` is `display: block !important; max-width: 1400px`.** Putting the chapter
   `.sidebar` (which is `grid-area` + `height: 100vh` + `overflow-y: auto`) inside it produces a
   fixed-width column with its own scrollbar — the V-005 regression the user reported. The landing
   page navigates via `.card-grid`, not a sidebar. The wide-monitor column rules in
   `styles.css` (`.main-article>p { column-width: 40ch }`) are pre-existing and deliberate:
   **do not touch that file** without an explicit request.
5. **Three of the five validation defects trace to Phase 2 design, not implementation.** The design
   specified escaping for chapter content but never said authored SUMMARY text is untrusted, and
   never said the landing page needs navigation. Verification passed throughout because every test
   asserted what the design specified. That gap is the argument for the validation phase existing.
6. **Gitea is reached over HTTPS with `GITEA_TOKEN`; SSH is dead.** Use `gtr` for issues and PRs.
