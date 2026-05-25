use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

use rstim::parser::parse_lines;
use rstim::perf::{
    PerfCircuitSource, PerfRecord, PerfWorkload, benchmark_cases, effective_repeat_count,
};
use rstim::stats::summarize;

fn rstim_bin() -> PathBuf {
    let exe = std::env::current_exe().expect("current exe");
    let profile_dir = exe
        .parent()
        .and_then(|parent| parent.parent())
        .expect("example binary should live under target/<profile>/examples");
    let profile_name = profile_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("debug");
    let binary = profile_dir.join(format!("rstim{}", std::env::consts::EXE_SUFFIX));
    assert!(
        binary.exists(),
        "rstim binary not found at {}. Build it first with `cargo build -p rstim --{} --bin rstim`.",
        binary.display(),
        profile_name
    );
    binary
}

fn source_text(source: PerfCircuitSource) -> String {
    match source {
        PerfCircuitSource::Inline { text } => text.to_string(),
        PerfCircuitSource::Generator {
            code,
            task,
            distance,
            rounds,
            noise,
        } => {
            let output = Command::new(rstim_bin())
                .args([
                    "gen",
                    "--code",
                    code,
                    "--task",
                    task,
                    "--distance",
                    &distance.to_string(),
                    "--rounds",
                    &rounds.to_string(),
                    "--after_clifford_depolarization",
                    &noise.to_string(),
                ])
                .output()
                .expect("run rstim gen");
            assert!(
                output.status.success(),
                "rstim gen failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout).expect("utf8 circuit")
        }
    }
}

fn run_case(
    case_label: &str,
    workload: PerfWorkload,
    text: &str,
    shots: Option<usize>,
) -> PerfRecord {
    let instrs = parse_lines(text).expect("parse benchmark circuit");
    let summary = summarize(&instrs);

    let mut child = Command::new(rstim_bin())
        .args(match workload {
            PerfWorkload::Sample => vec![
                "sample".to_string(),
                "--shots".to_string(),
                shots.unwrap_or(1).to_string(),
                "--out_format".to_string(),
                "01".to_string(),
            ],
            PerfWorkload::Detect => vec![
                "detect".to_string(),
                "--shots".to_string(),
                shots.unwrap_or(1).to_string(),
                "--out_format".to_string(),
                "01".to_string(),
            ],
            PerfWorkload::AnalyzeErrors => vec!["analyze_errors".to_string()],
        })
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn rstim");

    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(text.as_bytes())
        .expect("write circuit");

    let start = Instant::now();
    let output = child.wait_with_output().expect("wait");
    let elapsed = start.elapsed();
    assert!(
        output.status.success(),
        "rstim workload failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    PerfRecord {
        case_label: case_label.to_string(),
        tool_variant: "rstim-auto".to_string(),
        workload: workload.as_str().to_string(),
        qubits: summary.num_qubits,
        measurements: summary.num_measurements,
        detectors: summary.num_detectors,
        observables: summary.num_observables,
        repeat_depth: summary.max_repeat_depth,
        repeat_count: effective_repeat_count(&instrs),
        shots,
        wall_time_ns: elapsed.as_nanos(),
        peak_memory_bytes: None,
    }
}

fn main() {
    for case in benchmark_cases() {
        let text = source_text(case.source);
        let record = run_case(case.label, case.workload, &text, case.shots);
        print!("{}", record.to_json_line());
    }
}
