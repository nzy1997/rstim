# Feature Test Report: rstim RSMP v1

**Date:** 2026-07-24
**Project type:** Rust CLI and library
**Features tested:** `pack_samples`, `unpack_samples`, `--verify_only`, corruption handling, readiness and compression evidence
**Use Case:** Ephemeral — archive circuit measurements, recover decoder inputs, and evaluate production readiness
**Profile:** Ephemeral — CLI-comfortable QEC researcher without RSMP implementation knowledge
**Expected Outcome:** Losslessly archive measurements, recover measurements/detectors/observables with the original circuit, reject invalid inputs safely, and pass the documented readiness gate
**Verdict:** fail
**Critical Issues:** 1

## Summary

| Feature | Discoverable | Setup | Works | Expected Outcome Met | Doc Quality |
|---|---|---|---|---|---|
| Pack/unpack | partial | yes | yes | yes | good, but poorly linked |
| Verify-only | partial | yes | yes | yes | good |
| Failure-safe publication | no | yes | yes | yes | detailed in RSMP guide |
| Compression evidence | no | yes | partial | evidence is internally consistent; current-tree validation is non-hermetic | detailed evidence bundle |
| Readiness gate | yes | yes | partial | fails when an unrelated current release binary exists | good diagnostics |

## Per-Feature Details

### Pack and unpack

- **Role:** QEC researcher archiving measurement samples for later decoding.
- **What they tried:** Packed the committed four-shot compatibility measurements from both `b8` and `01`, then unpacked measurements, detectors, observables, and Stim-style `dets`.
- **Functionality:** Both inputs produced equivalent archives. Recovered measurement bytes matched exactly. Detector rows were `000000000`, `000000000`, `111111111`, `111111111`; observable rows were `0`, `1`, `1`, `0`.
- **Expected vs Actual Outcome:** Exact round trip and circuit-derived decoder data matched.
- **Friction points:** The root README and general documentation do not link the RSMP CLI guide. The tiny teaching case is 8 raw bytes versus a 356-byte archive because fixed envelope overhead dominates.

### Verify-only and failures

- **What they tried:** Verified single- and two-block archives; supplied a different circuit; truncated an archive; declared the wrong shot count; combined `--verify_only` with outputs; requested an unsupported format.
- **Functionality:** Valid archives produced stable PASS summaries. Invalid inputs returned the expected stable errors. Existing output sentinels were preserved and no temporary files remained after failure.
- **Expected vs Actual Outcome:** Met.
- **Friction points:** Subcommand help has little option description and does not show allowed format values. The guide should explicitly document `-` and the one-stdin/one-stdout rule.

### Compression evidence and readiness

- **What they tried:** Ran `make rsmp-v1-readiness`, independently checked the committed evidence hashes and arithmetic, and repeated readiness in a clean exported checkout.
- **Functionality:** Eleven focused Rust/CLI/compatibility/corruption checks passed in the working tree. The committed evidence files and gates are internally consistent. Full readiness passed in a clean checkout.
- **Blocking issue:** In the normal working tree, readiness fails with `environment rstim_binary sha256 mismatch` whenever `target/release/rstim` exists and differs from the historical evidence producer. The checker conditionally interprets a historical relative producer path as the current repository binary, so its outcome depends on an unrelated local build artifact.
- **Expected vs Actual Outcome:** The evidence is valid as historical pinned evidence, but the readiness command is not hermetic in an ordinary developer checkout.

## Issues Found

1. **High — readiness depends on an unrelated local release binary.** The same committed source and evidence pass in a clean export but fail when a current `target/release/rstim` exists.
2. **Medium — RSMP is difficult to discover.** The root README, getting-started path, showcase index, and general CLI documentation do not link `rstim/doc/rsmp-cli.md`.
3. **Low — the natural workflow is missing.** Documentation starts from committed fixture files instead of showing `sample → pack_samples → verify_only/unpack_samples`.
4. **Low — CLI help omits operational constraints.** Allowed format values, `-` stream semantics, and stream-conflict rules require the separate guide.

## Suggestions

1. Make committed-evidence validation independent of whether the historical producer path happens to exist in the current checkout; keep regeneration as a separate explicit mode.
2. Link the RSMP guide from the root documentation and add one natural end-to-end workflow.
3. In the showcase, label the four-shot example as a mechanics demonstration and use only the d11/r100 evidence for compression claims.
4. Present large-scale storage and runtime numbers as linear extrapolations from one pinned 1024-shot observation, not as measurements or guarantees.
