# Session Start: mdBook parity Increment A

**Date**: 2026-08-08
**UTC Start**: 09:52:25 UTC
**Change Slug**: mdbook-parity-a
**Branch**: task/1-pipeline-decomposition
**Scope**: Increment A — decompose build pipeline into collect/preprocess/render/index
**Linked Issue**: https://git.terraphim.cloud/terraphim/md-book/issues/1

## Repository Context
- Current branch: task/1-pipeline-decomposition (from main @ 82de35d)
- Recent commits: docs for parity research/plan/spec findings
- Working tree: clean for A; unrelated SEO/template WIP stashed as stash@{0}
- Relevant artefacts: docs/plans/mdbook-parity-{research,implementation-plan}.md

## Planned Artifact Chain
- Research: docs/plans/mdbook-parity-research.md (done)
- Design: docs/plans/mdbook-parity-implementation-plan.md (done, approved)
- Spec: Phase 2.5 findings for B appended to plan (A has no separate spec)
- Verification: docs/verification/verification-report-mdbook-parity-a.md
- Validation: docs/validation/validation-report-mdbook-parity-a.md
- Review: docs/plans/review-mdbook-parity-a.md
- Handover: .agent/handoffs/2026-08-08-mdbook-parity-a.md

## Subtasks
1. [M] A1 Extract stages into pipeline/ + render/
2. [S] A2 Preprocess identity seam
3. [S] A3 Baseline benchmark / byte-identical gate
4. [S] Verification + PR

## Risks and Blockers
- Zoned::now() year in templates may affect byte compare across year boundary only
- Pagefind index may be non-deterministic; compare HTML excluding pagefind/
- Unrelated WIP stashed — do not drop stash

## Notes
- Research/design already approved; skip re-research
- Zero behaviour change is the gate
