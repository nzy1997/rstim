# Issue 70 Rotated-Surface Fixture Manifest Closure Design

Date: 2026-06-18
Status: Design approved in-session, written for review
Scope: GitHub issue #70, rotated-surface entries in the built-in CSS fixture
manifest sweep

## Summary

Issue #70 asks the existing built-in CSS fixture manifest sweep to include
representative rotated-surface CLI exports:

```text
surface_rotated:d=3 / hx
surface_rotated:d=3 / hz
```

The current `master` branch already satisfies that minimum requirement through
the dependency chain that landed after the issue was opened:

- #61 / PR #84 added the shared manifest sweep in `qec-code/tests/cli.rs`.
- #68 / PR #83 added the `surface_rotated:d=<distance>` built-in CSS family.
- #69 / PR #86 pinned the `surface_rotated:d=3` `hx` and `hz` CLI fixtures and
  added both entries to the manifest sweep.

The right #70 closure is therefore an evidence and issue-triage pass, not a new
code change. The work should document that the current tree already has the
requested regression guard, rerun the issue's verification command, then close
or comment on the GitHub issue with the evidence.

## Goals

- Record why issue #70 is already satisfied on `master`.
- Avoid adding redundant `d=3` tests or duplicate fixture paths.
- Avoid broadening the manifest with optional `d=5` entries unless a later issue
  explicitly asks for that extra golden-output coverage.
- Verify the existing manifest sweep still passes.
- Prepare a concise issue-closing note that points to the existing code and
  verification result.

## Non-Goals

- Do not change production code.
- Do not change the `surface_rotated` family generator.
- Do not add `surface_rotated:d=5` pinned CLI fixtures in this issue.
- Do not regenerate existing fixtures.
- Do not add a second manifest or a registry-like test catalog.
- Do not touch toric manifest work for issues #72 or #73.

## Current State

The manifest in `qec-code/tests/cli.rs` already contains:

```rust
BuiltInCssFixtureCase {
    code_id: "surface_rotated:d=3",
    matrix: "hx",
    fixture: "surface_rotated_d3_hx.json",
},
BuiltInCssFixtureCase {
    code_id: "surface_rotated:d=3",
    matrix: "hz",
    fixture: "surface_rotated_d3_hz.json",
},
```

The pinned qec-code-owned fixtures already exist:

```text
qec-code/tests/fixtures/css/surface_rotated_d3_hx.json
qec-code/tests/fixtures/css/surface_rotated_d3_hz.json
```

The shared regression test is:

```text
built_in_css_fixture_manifest_exports_match_pinned_json
```

It runs the real binary CLI path:

```text
qec-code code css <code-id> <hx|hz>
```

and compares stdout byte-for-byte against the pinned fixture for every manifest
entry.

## Decision

Close issue #70 as already implemented by #69 / PR #86.

This keeps the manifest aligned with issue #61's original design: it remains a
small end-to-end regression yardstick, not a broad family sweep. Adding the
optional `surface_rotated:d=5` representative would create an independent code
diff, but it would not improve the minimum #70 acceptance criteria. The `d=5`
shape and weight behavior is already covered at the library-family layer by
issue #68 tests, while issue #70 specifically cares that representative CLI
exports are pinned in the shared manifest.

If future drift protection needs a larger rotated-surface CLI golden fixture,
that should be a separate follow-up with a clear reason for the extra manifest
noise.

## Evidence To Capture

Run:

```bash
cargo test -p qec-code --test cli built_in_css_fixture_manifest_exports_match_pinned_json
```

Expected result:

```text
test built_in_css_fixture_manifest_exports_match_pinned_json ... ok
```

This confirms:

- the manifest includes every currently pinned built-in CSS export case,
- the rotated-surface `d=3` `hx` fixture still matches CLI stdout,
- the rotated-surface `d=3` `hz` fixture still matches CLI stdout,
- the shared byte-for-byte regression guard has teeth for the #70 surface.

## Issue Closing Note

Use a short GitHub issue comment along these lines:

```text
Issue #70 is satisfied on current master by PR #86, which added
surface_rotated:d=3 / hx and surface_rotated:d=3 / hz to the shared built-in
CSS fixture manifest sweep and pinned the matching qec-code fixtures.

Verified with:

cargo test -p qec-code --test cli built_in_css_fixture_manifest_exports_match_pinned_json

Result: the manifest sweep passes and covers the rotated-surface d=3 hx/hz
entries requested here. I am closing this as covered by #69/#86 rather than
adding optional d=5 fixtures, keeping the manifest small and explicit.
```

## Acceptance Criteria

- This design document is committed.
- `cargo test -p qec-code --test cli built_in_css_fixture_manifest_exports_match_pinned_json`
  passes before the issue is closed.
- The closing note references #69 / PR #86 and the passing verification command.
- No production or test-code changes are made for #70 closure.
