use crate::compiled::CompiledCircuit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledPathDecision {
    FastPath,
    Fallback(&'static str),
}

pub fn choose_sampler_path(_compiled: &CompiledCircuit) -> CompiledPathDecision {
    CompiledPathDecision::Fallback("compiled sampler not implemented yet")
}

pub fn choose_analyzer_path(_compiled: &CompiledCircuit) -> CompiledPathDecision {
    CompiledPathDecision::Fallback("compiled analyzer not implemented yet")
}
