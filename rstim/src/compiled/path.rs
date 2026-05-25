use crate::compiled::{CompiledBlock, CompiledCircuit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledPathDecision {
    FastPath,
    Fallback(&'static str),
}

pub fn choose_sampler_path(compiled: &CompiledCircuit) -> CompiledPathDecision {
    if compiled.flags.has_loss {
        return CompiledPathDecision::Fallback("loss instructions require the interpreted path");
    }
    if compiled.flags.has_feedback {
        return CompiledPathDecision::Fallback(
            "feedback instructions require the interpreted path",
        );
    }

    CompiledPathDecision::FastPath
}

pub fn choose_analyzer_path(_compiled: &CompiledCircuit) -> CompiledPathDecision {
    CompiledPathDecision::Fallback("compiled analyzer not implemented yet")
}

pub fn has_single_top_level_repeat(compiled: &CompiledCircuit) -> bool {
    matches!(compiled.blocks.as_slice(), [CompiledBlock::Repeat(_)])
}
