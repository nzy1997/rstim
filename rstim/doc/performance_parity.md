# Performance Parity Benchmark Guide

This document tracks the rerun workflow for the performance parity foundation.

## Baseline Harness

Run the baseline harness with:

```sh
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

`peak_memory_bytes` is captured from the benchmark process on Unix platforms by
reading `getrusage(RUSAGE_SELF)`.

## Milestone Acceptance

The first complete performance milestone is only accepted when:

1. semantic regressions stay green
2. benchmark cases still cover sample, detect, and analyze_errors
3. the benchmark comparison clearly shows the intended improvement
