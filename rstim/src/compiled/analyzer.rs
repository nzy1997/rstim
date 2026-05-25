use crate::compiled::CompiledCircuit;
use crate::dem::DetectorErrorModel;
use crate::error_analyzer::AnalyzeOptions;

pub fn analyze_compiled_circuit(
    _compiled: &CompiledCircuit,
    _options: AnalyzeOptions,
) -> Result<DetectorErrorModel, String> {
    Err("compiled analyzer not implemented yet".to_string())
}
