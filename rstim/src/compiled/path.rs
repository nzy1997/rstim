use crate::compiled::CompiledCircuit;
#[cfg(test)]
use crate::compiled::CompiledBlock;

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

#[cfg(test)]
fn has_single_top_level_repeat(compiled: &CompiledCircuit) -> bool {
    matches!(compiled.blocks.as_slice(), [CompiledBlock::Repeat(_)])
}

#[cfg(test)]
mod tests {
    use super::has_single_top_level_repeat;
    use crate::compiled::compile_circuit;
    use crate::parser::parse_lines;

    #[test]
    fn has_single_top_level_repeat_requires_an_exact_single_repeat_block() {
        let single_repeat =
            compile_circuit(&parse_lines("REPEAT 8 {\n  M 0\n}\n").unwrap()).unwrap();
        let prefixed_repeat =
            compile_circuit(&parse_lines("R 0\nREPEAT 8 {\n  M 0\n}\n").unwrap()).unwrap();
        let two_repeats =
            compile_circuit(&parse_lines("REPEAT 2 {\n  M 0\n}\nREPEAT 3 {\n  M 0\n}\n").unwrap())
                .unwrap();

        assert!(has_single_top_level_repeat(&single_repeat));
        assert!(!has_single_top_level_repeat(&prefixed_repeat));
        assert!(!has_single_top_level_repeat(&two_repeats));
    }
}
