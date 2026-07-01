# Progress

- Repo: apify/impit, branch `claude/issue-479-fixes-r2554a`, base `master`, HEAD 9d2204f.
- Triage: PROCEED, complexity medium (blast-radius override on public response contract).
- Verify: both code claims VALID (Python line stale 544→542). Issue's suggested `from_utf8_lossy`
  is incomplete (regresses #434). Chosen fix: UTF-8-first, latin-1 fallback.
- Explore: shared helper home = `impit/src/response_parsing/mod.rs`, re-exported via
  `impit::utils`. Existing regression guard: `impit-node/test/basics.test.ts:569` +
  `mock.server.ts:105-118` (sends bare 0xE4, expects `ä`).

## Oracle
- Commands: standalone `rustc --edition 2021 --test .devforge/oracle_header_decode.rs && run`.
- Reason: full `cargo build/test` BLOCKED — pinned git dep github.com/apify/h2 → 403 via org
  egress proxy, cargo git cache empty. Org policy denial; not routed around. Standalone rustc
  test proves the pure decode algorithm; binding compile/integration deferred to CI.

## Resolved registry (from config.json + registry.base.json)
- verify_request: brainstorming/opus | architect: writing-plans/opus | implementer: feature-dev/opus
- reviewers: staff-review/sonnet
- final_reviewers: thermonuclear/sonnet, code-review/sonnet
- limits: inner_iterations 3, final_review_rounds 2 | plan_mode_gate true

## State
- Phase: design-gate (awaiting human approval before any source edit).

## Finish (rev 2)
- PR #492 opened: https://github.com/apify/impit/pull/492 (base master).
- Oracle green throughout (rustfmt/rustc/rustdoc + ruff/py_compile). Binding compile + JS/Py test
  execution are CI-gated (h2 egress 403 blocks local build).
- Reviewer staff-review: PASS. Final reviewers thermonuclear + code-review: both PASS after 2
  final-reopen rounds (fixed: clone() dropping rawHeaders, index.d.ts, wire-order/casing overclaims).
- Approvals: design gate rev1 + rev2 (chat), create-PR (chat) 2026-07-01.
- Phase: done.
