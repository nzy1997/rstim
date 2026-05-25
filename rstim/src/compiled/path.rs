use crate::compiled::{CompiledBlock, CompiledCircuit, CompiledRepeatRegion};
use crate::ir::StimInstr;

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

pub fn choose_analyzer_path(compiled: &CompiledCircuit) -> CompiledPathDecision {
    if compiled.flags.has_loss {
        return CompiledPathDecision::Fallback("loss instructions require the flattened analyzer");
    }
    if compiled.flags.has_feedback {
        return CompiledPathDecision::Fallback(
            "feedback instructions require the flattened analyzer",
        );
    }
    if compiled.flags.has_nested_repeat {
        return CompiledPathDecision::Fallback(
            "nested repeat blocks require the flattened analyzer",
        );
    }
    if let [CompiledBlock::Repeat(region)] = compiled.blocks.as_slice() {
        if supports_reset_periodic_body(region) {
            return CompiledPathDecision::FastPath;
        }
        return CompiledPathDecision::Fallback(
            "compiled analyzer currently supports only reset-based single top-level repeat regions",
        );
    }

    CompiledPathDecision::Fallback(
        "compiled analyzer currently supports only a single top-level repeat region",
    )
}

fn supports_reset_periodic_body(region: &CompiledRepeatRegion) -> bool {
    let mut has_reset_measurement = false;
    for instr in &region.body_source {
        let StimInstr::Op { name, .. } = instr else {
            return false;
        };
        match name.as_str() {
            "MR" | "MRX" | "MRY" | "MRZ" => {
                has_reset_measurement = true;
            }
            "M" | "MX" | "MY" | "MZ" | "MXX" | "MYY" | "MZZ" | "MPP" | "SPP" | "SPP_DAG" => {
                return false;
            }
            _ => {}
        }
    }
    has_reset_measurement
}

#[cfg(test)]
mod tests {
    use super::supports_reset_periodic_body;
    use crate::compiled::compile_circuit;
    use crate::compiled::CompiledBlock;
    use crate::parser::parse_lines;

    #[test]
    fn supports_reset_periodic_body_requires_reset_measurements() {
        let reset_repeat =
            compile_circuit(&parse_lines("REPEAT 8 {\n  X_ERROR(0.1) 0\n  MR 0\n}\n").unwrap())
                .unwrap();
        let plain_measure_repeat =
            compile_circuit(&parse_lines("REPEAT 8 {\n  X_ERROR(0.1) 0\n  M 0\n}\n").unwrap())
                .unwrap();

        let [CompiledBlock::Repeat(reset_region)] = reset_repeat.blocks.as_slice() else {
            panic!("expected top-level repeat");
        };
        let [CompiledBlock::Repeat(plain_region)] = plain_measure_repeat.blocks.as_slice() else {
            panic!("expected top-level repeat");
        };

        assert!(supports_reset_periodic_body(reset_region));
        assert!(!supports_reset_periodic_body(plain_region));
    }
}
