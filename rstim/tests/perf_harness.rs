use rstim::perf::{benchmark_cases, PerfRecord, PerfWorkload};

#[test]
fn benchmark_cases_cover_sampling_detect_and_repeat_analysis() {
    let cases = benchmark_cases();

    assert!(
        cases
            .iter()
            .any(|case| case.workload == PerfWorkload::Sample),
        "expected at least one sample benchmark case"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.workload == PerfWorkload::Detect),
        "expected at least one detect benchmark case"
    );
    assert!(
        cases
            .iter()
            .any(|case| case.workload == PerfWorkload::AnalyzeErrors),
        "expected at least one analyze_errors benchmark case"
    );
    assert!(
        cases.iter().any(|case| case.label.contains("repeat")),
        "expected at least one repeat-focused benchmark case"
    );
}

#[test]
fn perf_record_json_line_contains_required_keys() {
    let record = PerfRecord {
        case_label: "surface-detect-d13".to_string(),
        tool_variant: "rstim-auto".to_string(),
        workload: PerfWorkload::Detect.as_str().to_string(),
        qubits: 25,
        measurements: 48,
        detectors: 24,
        observables: 1,
        repeat_depth: 1,
        repeat_count: 13,
        shots: Some(10_000),
        wall_time_ns: 123_456,
        peak_memory_bytes: None,
    };

    let line = record.to_json_line();

    assert!(line.contains("\"case_label\":\"surface-detect-d13\""));
    assert!(line.contains("\"tool_variant\":\"rstim-auto\""));
    assert!(line.contains("\"workload\":\"detect\""));
    assert!(line.contains("\"repeat_count\":13"));
    assert!(line.ends_with('\n'));
}
