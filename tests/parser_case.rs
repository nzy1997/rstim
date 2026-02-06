use rstim::parser::parse_lines;

#[test]
fn parses_case_insensitive_names() {
    let instrs = parse_lines("h 0\nDeTeCtOr rec[-1]\n").unwrap();
    assert_eq!(instrs[0].name, "H");
    assert_eq!(instrs[1].name, "DETECTOR");
}

#[test]
fn rejects_non_negative_rec() {
    let err = parse_lines("DETECTOR rec[0]\n").unwrap_err();
    assert!(err.contains("rec"));
}
