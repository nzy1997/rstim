use rstim::parser::parse_lines;
use rstim::ir::StimTarget;

#[test]
fn parses_simple_gate() {
    let instrs = parse_lines("H 0\n").unwrap();
    assert_eq!(instrs.len(), 1);
    assert_eq!(instrs[0].name, "H");
    assert_eq!(instrs[0].targets, vec![StimTarget::Qubit(0)]);
}

#[test]
fn parses_detector_with_rec() {
    let instrs = parse_lines("DETECTOR rec[-1]\n").unwrap();
    assert_eq!(instrs[0].name, "DETECTOR");
}
