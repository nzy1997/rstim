use rstim::compiled::{CompiledPathDecision, choose_analyzer_path, compile_circuit};
use rstim::parser::parse_lines;
use rstim::perf::{
    PerfRecord, PerfVariant, PerfWorkload, benchmark_case_variants, benchmark_cases,
    benchmark_variants,
};
use serde_json::Value;

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
    let json: Value = serde_json::from_str(line.trim_end()).unwrap();

    assert!(line.ends_with('\n'));
    assert_eq!(json["case_label"], "surface-detect-d13");
    assert_eq!(json["tool_variant"], "rstim-auto");
    assert_eq!(json["workload"], "detect");
    assert_eq!(json["repeat_count"], 13);
    assert_eq!(json["shots"], 10_000);
    assert_eq!(json["peak_memory_bytes"], Value::Null);
}

#[test]
fn benchmark_variants_cover_stim_and_both_rstim_backends() {
    let variants = benchmark_variants();

    assert!(variants.contains(&PerfVariant::StimCli));
    assert!(variants.contains(&PerfVariant::RstimInterpreted));
    assert!(variants.contains(&PerfVariant::RstimCompiled));
    assert!(variants.contains(&PerfVariant::RstimAnalyzerFlattened));
    assert!(variants.contains(&PerfVariant::RstimAnalyzerCompiled));
}

#[test]
fn benchmark_cases_include_a_compiled_analyzer_compatible_repeat_case() {
    let cases = benchmark_cases();

    let compatible = cases.iter().filter(|case| case.workload == PerfWorkload::AnalyzeErrors).any(
        |case| match case.source {
            rstim::perf::PerfCircuitSource::Inline { text } => {
                let instrs = parse_lines(text).unwrap();
                let compiled = compile_circuit(&instrs).unwrap();
                choose_analyzer_path(&compiled) == CompiledPathDecision::FastPath
            }
            rstim::perf::PerfCircuitSource::Generator { .. } => false,
        },
    );

    assert!(
        compatible,
        "expected at least one analyze_errors benchmark case to support the compiled analyzer path"
    );
}

#[test]
fn loss_protection_case_skips_compiled_sampler_variant() {
    let case = benchmark_cases()
        .into_iter()
        .find(|case| case.label == "loss-protection-sample")
        .expect("loss protection case");
    let text = match case.source {
        rstim::perf::PerfCircuitSource::Inline { text } => text,
        _ => panic!("loss protection case should be inline"),
    };
    let instrs = parse_lines(text).unwrap();

    let variants = benchmark_case_variants(case, &instrs).unwrap();

    assert!(variants.contains(&PerfVariant::StimCli));
    assert!(variants.contains(&PerfVariant::RstimInterpreted));
    assert!(!variants.contains(&PerfVariant::RstimCompiled));
}
