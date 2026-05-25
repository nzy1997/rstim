# Performance Parity Benchmark Guide

This document tracks the rerun workflow for the performance parity foundation.

## Baseline Harness

Build the matching `rstim` binary, then run the baseline harness:

```sh
cargo build -p rstim --release --bin rstim
cargo run -p rstim --release --example performance_parity_foundation
```

Use debug builds only to smoke-test wiring. Use `--release` for any timing or
comparison evidence, and build the matching debug binary first if you do a
smoke run:

```sh
cargo build -p rstim --bin rstim
cargo run -p rstim --example performance_parity_foundation
```

Each output line is JSON and includes:

- `case_label`
- `tool_variant`
- `workload`
- `qubits`
- `measurements`
- `detectors`
- `observables`
- `repeat_depth`
- `repeat_count`
- `shots`
- `wall_time_ns`
- `peak_memory_bytes`

In this Task 1 scaffold, `peak_memory_bytes` is reserved for future per-case
child-process measurement and currently emits `null`.

## Milestone Acceptance

The first complete performance milestone is only accepted when:

1. semantic regressions stay green
2. benchmark cases still cover sample, detect, and analyze_errors
3. the benchmark comparison comes from `--release` runs and clearly shows the
   intended improvement
