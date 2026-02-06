use crate::ir::{StimInstr, StimTarget};

pub fn parse_lines(input: &str) -> Result<Vec<StimInstr>, String> {
    let mut out = Vec::new();
    for (line_no, raw) in input.lines().enumerate() {
        let line = raw.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let name = parts
            .next()
            .ok_or_else(|| format!("line {}: empty", line_no + 1))?;
        let mut instr = StimInstr::new(name, vec![], vec![]);
        for token in parts {
            if let Some(t) = parse_target(token)? {
                instr.targets.push(t);
            }
        }
        out.push(instr);
    }
    Ok(out)
}

fn parse_target(token: &str) -> Result<Option<StimTarget>, String> {
    if token.starts_with("rec[") && token.ends_with(']') {
        let inner = &token[4..token.len() - 1];
        let val: i32 = inner
            .parse()
            .map_err(|_| format!("bad rec target {token}"))?;
        return Ok(Some(StimTarget::Rec(val)));
    }
    if let Ok(q) = token.parse::<u32>() {
        return Ok(Some(StimTarget::Qubit(q)));
    }
    Err(format!("unsupported target {token}"))
}
