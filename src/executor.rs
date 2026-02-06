use rand::Rng;
use yao_rs::{Gate, Circuit, State, apply, measure::measure_and_collapse, put, control};

use crate::ir::{StimInstr, StimTarget};
use crate::recorder::Recorder;

pub struct Executor {
    instrs: Vec<StimInstr>,
}

pub struct ExecOutput {
    pub measurements: Vec<bool>,
    pub detectors: Vec<bool>,
    pub observables: Vec<(u32, bool)>,
}

impl Executor {
    pub fn from_instrs(instrs: Vec<StimInstr>) -> Result<Self, String> {
        Ok(Self { instrs })
    }

    pub fn run(&mut self, rng: &mut impl Rng) -> Result<ExecOutput, String> {
        let n = max_qubit(&self.instrs)?;
        let mut state = State::zero_state(&vec![2; n]);
        let mut recorder = Recorder::default();
        let mut detectors = Vec::new();
        let mut observables = Vec::new();

        for instr in &self.instrs {
            match instr.name.as_str() {
                "H" | "X" | "Y" | "Z" | "S" => {
                    let gate = match instr.name.as_str() {
                        "H" => Gate::H,
                        "X" => Gate::X,
                        "Y" => Gate::Y,
                        "Z" => Gate::Z,
                        _ => Gate::S,
                    };
                    for t in &instr.targets {
                        let q = expect_qubit(t)?;
                        let c = Circuit::new(vec![2; n], vec![put(vec![q], gate.clone())])
                            .map_err(|e| format!("circuit: {e:?}"))?;
                        state = apply(&c, &state);
                    }
                }
                "CX" | "CNOT" => {
                    let pairs = qubit_pairs(&instr.targets)?;
                    for (c, t) in pairs {
                        let ckt = Circuit::new(vec![2; n], vec![control(vec![c], vec![t], Gate::X)])
                            .map_err(|e| format!("circuit: {e:?}"))?;
                        state = apply(&ckt, &state);
                    }
                }
                "CZ" => {
                    let pairs = qubit_pairs(&instr.targets)?;
                    for (c, t) in pairs {
                        let ckt = Circuit::new(vec![2; n], vec![control(vec![c], vec![t], Gate::Z)])
                            .map_err(|e| format!("circuit: {e:?}"))?;
                        state = apply(&ckt, &state);
                    }
                }
                "M" => {
                    for t in &instr.targets {
                        let q = expect_qubit(t)?;
                        let result = measure_and_collapse(&mut state, Some(&[q]), rng);
                        let bit = result[0] == 1;
                        recorder.push(bit);
                    }
                }
                "DETECTOR" => {
                    let bit = xor_recs(&recorder, &instr.targets)?;
                    detectors.push(bit);
                }
                "OBSERVABLE_INCLUDE" => {
                    let index = instr.args.get(0).copied().unwrap_or(0.0) as u32;
                    let bit = xor_recs(&recorder, &instr.targets)?;
                    observables.push((index, bit));
                }
                _ => return Err(format!("unsupported instruction {}", instr.name)),
            }
        }

        Ok(ExecOutput {
            measurements: recorder_bits(recorder),
            detectors,
            observables,
        })
    }
}

fn recorder_bits(r: Recorder) -> Vec<bool> {
    let mut out = Vec::new();
    for i in 1..=r.len() {
        out.push(r.rec(-(i as i32)).unwrap());
    }
    out.reverse();
    out
}

fn max_qubit(instrs: &[StimInstr]) -> Result<usize, String> {
    let mut max_q: Option<u32> = None;
    for i in instrs {
        for t in &i.targets {
            if let StimTarget::Qubit(q) = t {
                max_q = Some(max_q.map_or(*q, |m| m.max(*q)));
            }
        }
    }
    Ok(max_q.map(|m| (m as usize) + 1).unwrap_or(0))
}

fn expect_qubit(t: &StimTarget) -> Result<usize, String> {
    match t {
        StimTarget::Qubit(q) => Ok(*q as usize),
        _ => Err("expected qubit target".to_string()),
    }
}

fn qubit_pairs(targets: &[StimTarget]) -> Result<Vec<(usize, usize)>, String> {
    if targets.len() % 2 != 0 {
        return Err("odd number of targets".to_string());
    }
    let mut out = Vec::new();
    let mut it = targets.iter();
    while let (Some(a), Some(b)) = (it.next(), it.next()) {
        out.push((expect_qubit(a)?, expect_qubit(b)?));
    }
    Ok(out)
}

fn xor_recs(r: &Recorder, targets: &[StimTarget]) -> Result<bool, String> {
    let mut acc = false;
    for t in targets {
        match t {
            StimTarget::Rec(o) => {
                let bit = r.rec(*o).ok_or("rec out of range")?;
                acc ^= bit;
            }
            _ => return Err("detector target must be rec".to_string()),
        }
    }
    Ok(acc)
}
