# Issue 70 Rotated-Surface Fixture Manifest Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close GitHub issue #70 with verified evidence that current `master` already includes the requested rotated-surface entries in the built-in CSS fixture manifest sweep.

**Architecture:** No production or test code changes are needed. The workflow verifies the existing manifest entries and pinned fixtures, then records the result on GitHub before closing the issue.

**Tech Stack:** Rust `cargo test`, qec-code CLI test suite, GitHub CLI `gh issue`.

## Global Constraints

- Do not change production code.
- Do not change test code.
- Do not add `surface_rotated:d=5` pinned CLI fixtures for issue #70.
- Do not regenerate existing fixtures.
- Do not touch toric manifest work for issues #72 or #73.
- Keep the closure evidence tied to #69 / PR #86 and the passing manifest sweep.

---

## File Structure

No code files should be modified.

Reference files to inspect:

- `docs/superpowers/specs/2026-06-18-issue-70-rotated-surface-fixture-manifest-closure-design.md`: approved closure design and acceptance criteria.
- `qec-code/tests/cli.rs`: existing manifest and `built_in_css_fixture_manifest_exports_match_pinned_json` test.
- `qec-code/tests/fixtures/css/surface_rotated_d3_hx.json`: existing `surface_rotated:d=3 / hx` pinned CLI fixture.
- `qec-code/tests/fixtures/css/surface_rotated_d3_hz.json`: existing `surface_rotated:d=3 / hz` pinned CLI fixture.

Files to modify during execution:

- None.

## Task 1: Verify Existing Manifest Coverage

**Files:**
- Inspect: `qec-code/tests/cli.rs`
- Inspect: `qec-code/tests/fixtures/css/surface_rotated_d3_hx.json`
- Inspect: `qec-code/tests/fixtures/css/surface_rotated_d3_hz.json`
- Test: `qec-code/tests/cli.rs`

**Interfaces:**
- Consumes: approved design document and current `master` checkout.
- Produces: terminal evidence that issue #70's requested manifest sweep passes.

- [ ] **Step 1: Confirm the manifest entries are present**

Run:

```bash
rg -n 'surface_rotated:d=3|surface_rotated_d3_hx\.json|surface_rotated_d3_hz\.json' qec-code/tests/cli.rs
```

Expected: output includes manifest entries for:

```text
code_id: "surface_rotated:d=3"
matrix: "hx"
fixture: "surface_rotated_d3_hx.json"
matrix: "hz"
fixture: "surface_rotated_d3_hz.json"
```

- [ ] **Step 2: Confirm the pinned fixture files exist**

Run:

```bash
ls -1 qec-code/tests/fixtures/css/surface_rotated_d3_hx.json qec-code/tests/fixtures/css/surface_rotated_d3_hz.json
```

Expected:

```text
qec-code/tests/fixtures/css/surface_rotated_d3_hx.json
qec-code/tests/fixtures/css/surface_rotated_d3_hz.json
```

- [ ] **Step 3: Run the issue #70 verification command**

Run:

```bash
cargo test -p qec-code --test cli built_in_css_fixture_manifest_exports_match_pinned_json
```

Expected: the command succeeds and includes:

```text
test built_in_css_fixture_manifest_exports_match_pinned_json ... ok
```

- [ ] **Step 4: Confirm no code changes were introduced**

Run:

```bash
git status --short
```

Expected: no modified production or test files. If the only change is this plan file, keep it separate from the issue closure evidence.

## Task 2: Close Issue #70 With Evidence

**Files:**
- Inspect: `docs/superpowers/specs/2026-06-18-issue-70-rotated-surface-fixture-manifest-closure-design.md`

**Interfaces:**
- Consumes: Task 1 verification output.
- Produces: GitHub issue #70 comment and closed issue state.

- [ ] **Step 1: Post the closure evidence comment**

Run:

```bash
gh issue comment 70 --repo nzy1997/rstim --body "Issue #70 is satisfied on current master by PR #86, which added surface_rotated:d=3 / hx and surface_rotated:d=3 / hz to the shared built-in CSS fixture manifest sweep and pinned the matching qec-code fixtures.

Verified with:

cargo test -p qec-code --test cli built_in_css_fixture_manifest_exports_match_pinned_json

Result: the manifest sweep passes and covers the rotated-surface d=3 hx/hz entries requested here. Closing this as covered by #69/#86 rather than adding optional d=5 fixtures, keeping the manifest small and explicit."
```

Expected: GitHub prints the created issue comment URL.

- [ ] **Step 2: Close issue #70**

Run:

```bash
gh issue close 70 --repo nzy1997/rstim --reason completed
```

Expected: GitHub reports that issue #70 is closed.

- [ ] **Step 3: Verify issue #70 is closed**

Run:

```bash
gh issue view 70 --repo nzy1997/rstim --json number,title,state --jq '{number,title,state}'
```

Expected:

```json
{"number":70,"title":"Add rotated-surface entries to the built-in CSS fixture manifest sweep","state":"CLOSED"}
```

- [ ] **Step 4: Commit the plan if it is still uncommitted**

Run:

```bash
git status --short
```

Expected: if `docs/superpowers/plans/2026-06-18-issue-70-rotated-surface-fixture-manifest-closure.md` is untracked, commit only that file:

```bash
git add docs/superpowers/plans/2026-06-18-issue-70-rotated-surface-fixture-manifest-closure.md
git commit -m "docs: plan issue 70 closure"
```

Expected: a docs-only commit is created.
