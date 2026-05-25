use rstim::error_analyzer::{AnalyzeBackend, AnalyzeOptions, ErrorAnalyzer};
use rstim::parser::parse_lines;
use rstim::showcase::dem_semantic_summary;

fn analyze_error_probabilities(
    circuit: &str,
    backend: AnalyzeBackend,
) -> Result<std::collections::BTreeMap<String, f64>, String> {
    let instrs = parse_lines(circuit).unwrap();
    let dem = ErrorAnalyzer::circuit_to_dem_with_options(
        &instrs,
        AnalyzeOptions {
            backend,
            ..AnalyzeOptions::default()
        },
    )?;
    Ok(dem_semantic_summary(&dem).error_probabilities)
}

fn analyze_error_probabilities_decomposed(
    circuit: &str,
    backend: AnalyzeBackend,
) -> Result<std::collections::BTreeMap<String, f64>, String> {
    let instrs = parse_lines(circuit).unwrap();
    let dem = ErrorAnalyzer::circuit_to_dem_with_options_decomposed(
        &instrs,
        AnalyzeOptions {
            backend,
            ..AnalyzeOptions::default()
        },
    )?;
    Ok(dem_semantic_summary(&dem).error_probabilities)
}

#[test]
fn compiled_backend_matches_flattened_for_single_top_level_repeat() {
    let circuit = "REPEAT 32 {\n    X_ERROR(0.125) 0\n    MR 0\n    DETECTOR rec[-1]\n}\n";

    assert_eq!(
        analyze_error_probabilities(circuit, AnalyzeBackend::Flattened),
        analyze_error_probabilities(circuit, AnalyzeBackend::Compiled)
    );
}

#[test]
fn auto_backend_falls_back_to_flattened_for_mixed_top_level_structure() {
    let circuit = "R 0\nREPEAT 4 {\n    X_ERROR(0.125) 0\n    MR 0\n    DETECTOR rec[-1]\n}\nMR 0\nDETECTOR rec[-1]\n";

    assert_eq!(
        analyze_error_probabilities(circuit, AnalyzeBackend::Flattened),
        analyze_error_probabilities(circuit, AnalyzeBackend::Auto)
    );
}

#[test]
fn compiled_backend_preserves_decomposed_entry_point_for_supported_repeat() {
    let circuit =
        "REPEAT 8 {\n    DEPOLARIZE2(0.01) 0 1\n    MR 0 1\n    DETECTOR rec[-1]\n    DETECTOR rec[-2]\n}\n";

    assert_eq!(
        analyze_error_probabilities_decomposed(circuit, AnalyzeBackend::Flattened),
        analyze_error_probabilities_decomposed(circuit, AnalyzeBackend::Compiled)
    );
}
