use rstim::compiled::{
    choose_analyzer_path, choose_sampler_path, compile_circuit, path::has_single_top_level_repeat,
    CompiledPathDecision,
};
use rstim::parser::parse_lines;

#[test]
fn sampler_path_uses_fast_path_for_simple_repeat_circuit() {
    let compiled =
        compile_circuit(&parse_lines("REPEAT 8 {\n  X_ERROR(0.001) 0\n  M 0\n}\n").unwrap())
            .unwrap();

    assert_eq!(
        choose_sampler_path(&compiled),
        CompiledPathDecision::FastPath
    );
}

#[test]
fn sampler_path_falls_back_for_loss_circuit() {
    let compiled = compile_circuit(&parse_lines("LOSS(1) 0\nMRL 0\n").unwrap()).unwrap();

    assert_eq!(
        choose_sampler_path(&compiled),
        CompiledPathDecision::Fallback("loss instructions require the interpreted path")
    );
}

#[test]
fn sampler_path_falls_back_for_feedback_circuit() {
    let compiled = compile_circuit(&parse_lines("M 0\nCX rec[-1] 0\n").unwrap()).unwrap();

    assert_eq!(
        choose_sampler_path(&compiled),
        CompiledPathDecision::Fallback("feedback instructions require the interpreted path")
    );
}

#[test]
fn analyzer_path_falls_back_until_the_loop_aware_backend_exists() {
    let compiled = compile_circuit(
        &parse_lines("REPEAT 8 {\n  X_ERROR(0.001) 0\n  M 0\n  DETECTOR rec[-1]\n}\n").unwrap(),
    )
    .unwrap();

    assert_eq!(
        choose_analyzer_path(&compiled),
        CompiledPathDecision::Fallback("compiled analyzer not implemented yet")
    );
}

#[test]
fn has_single_top_level_repeat_requires_an_exact_single_repeat_block() {
    let single_repeat = compile_circuit(&parse_lines("REPEAT 8 {\n  M 0\n}\n").unwrap()).unwrap();
    let prefixed_repeat =
        compile_circuit(&parse_lines("R 0\nREPEAT 8 {\n  M 0\n}\n").unwrap()).unwrap();
    let two_repeats =
        compile_circuit(&parse_lines("REPEAT 2 {\n  M 0\n}\nREPEAT 3 {\n  M 0\n}\n").unwrap())
            .unwrap();

    assert!(has_single_top_level_repeat(&single_repeat));
    assert!(!has_single_top_level_repeat(&prefixed_repeat));
    assert!(!has_single_top_level_repeat(&two_repeats));
}
