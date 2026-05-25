use serde::Serialize;

use crate::ir::StimInstr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfWorkload {
    Sample,
    Detect,
    AnalyzeErrors,
}

impl PerfWorkload {
    pub fn as_str(&self) -> &'static str {
        match self {
            PerfWorkload::Sample => "sample",
            PerfWorkload::Detect => "detect",
            PerfWorkload::AnalyzeErrors => "analyze_errors",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerfCircuitSource {
    Generator {
        code: &'static str,
        task: &'static str,
        distance: usize,
        rounds: usize,
        noise: f64,
    },
    Inline {
        text: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerfBenchmarkCase {
    pub label: &'static str,
    pub workload: PerfWorkload,
    pub source: PerfCircuitSource,
    pub shots: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerfRecord {
    pub case_label: String,
    pub tool_variant: String,
    pub workload: String,
    pub qubits: usize,
    pub measurements: usize,
    pub detectors: usize,
    pub observables: usize,
    pub repeat_depth: usize,
    pub repeat_count: usize,
    pub shots: Option<usize>,
    pub wall_time_ns: u128,
    pub peak_memory_bytes: Option<u64>,
}

impl PerfRecord {
    pub fn to_json_line(&self) -> String {
        let mut line = serde_json::to_string(self).expect("serialize perf record");
        line.push('\n');
        line
    }
}

pub fn effective_repeat_count(instrs: &[StimInstr]) -> usize {
    effective_repeat_count_with_multiplier(instrs, 1)
}

fn effective_repeat_count_with_multiplier(instrs: &[StimInstr], multiplier: usize) -> usize {
    let mut total = 0usize;
    for instr in instrs {
        if let StimInstr::Repeat { count, body } = instr {
            let scaled = multiplier.saturating_mul(*count as usize);
            total = total.saturating_add(scaled);
            total = total.saturating_add(effective_repeat_count_with_multiplier(body, scaled));
        }
    }
    total
}

pub fn benchmark_cases() -> Vec<PerfBenchmarkCase> {
    vec![
        PerfBenchmarkCase {
            label: "rep-sample-d13-r13",
            workload: PerfWorkload::Sample,
            source: PerfCircuitSource::Generator {
                code: "repetition_code",
                task: "memory",
                distance: 13,
                rounds: 13,
                noise: 0.001,
            },
            shots: Some(20_000),
        },
        PerfBenchmarkCase {
            label: "surface-detect-d13-r13",
            workload: PerfWorkload::Detect,
            source: PerfCircuitSource::Generator {
                code: "surface_code",
                task: "rotated_memory_x",
                distance: 13,
                rounds: 13,
                noise: 0.001,
            },
            shots: Some(10_000),
        },
        PerfBenchmarkCase {
            label: "repeat-analyze-large",
            workload: PerfWorkload::AnalyzeErrors,
            source: PerfCircuitSource::Inline {
                text: "REPEAT 4096 {\n    X_ERROR(0.001) 0\n    M 0\n    DETECTOR rec[-1]\n}\n",
            },
            shots: None,
        },
        PerfBenchmarkCase {
            label: "loss-protection-sample",
            workload: PerfWorkload::Sample,
            source: PerfCircuitSource::Inline {
                text: "LOSS(1) 0\nMRL 0\nDETECTOR rec[-1]\n",
            },
            shots: Some(128),
        },
    ]
}
