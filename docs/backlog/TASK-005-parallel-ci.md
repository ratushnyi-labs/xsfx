# TASK-005 — Parallelize CI/CD pipeline

**Status:** `DONE`
**Created:** 2026-03-25
**Updated:** 2026-03-25

## Raw Request

> the pipeline is quite complex and very slow. maybe you can think how to parallel it and speedup?
> i like how in https://github.com/ratushnyi-labs/trim it is done.
> for up here is would be matrix to build stubs then matrix to build clis and then self compress.
> on tag make release

## Refined Description

**Scope:** Rewrite CI/CD from sequential single-runner Docker builds to parallel matrix strategy using native OS runners, inspired by ratushnyi-labs/trim. Add self-compression step and tag-triggered releases.
**Non-goals:** Removing local Docker build support (build.sh/entrypoint preserved).
**Impacted UCs:** None (infrastructure only)
**Impacted BR/WF:** None
**Dependencies:** None
**Risks / Open Questions:** None

## Estimation

**Level:** `LOW`
**Justification:** Workflow YAML rewrite only. No source code changes. Clear pattern from trim repo to follow.

## Speck Reference

N/A — infrastructure-only task, no code changes.
