# RStim MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Parse `.stim` files, execute a Clifford-only subset, track `rec[]`, and emit `DETECTOR`/`OBSERVABLE_INCLUDE` outputs.

**Architecture:** Add a `rstim` library with `ir`, `parser`, `recorder`, and `executor` modules. Use `yao-rs` for gate representation and state evolution; keep Stim annotations and measurement record in `rstim`.

**Tech Stack:** Rust 2024, `yao-rs` (path dependency), `rand` for sampling, standard library.

### Task 1: Convert crate to library + keep a tiny binary

**Files:**
- Create: `src/lib.rs`
- Modify: `src/main.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing test**

Create `tests/lib_smoke.rs`:
```rust
use rstim::version;

#[test]
fn version_is_nonempty() {
    assert!(!version().is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -q`
Expected: FAIL with `unresolved import rstim` or `version` not found.

**Step 3: Write minimal implementation**

Create `src/lib.rs`:
```rust
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

Modify `src/main.rs`:
```rust
fn main() {
    println!("rstim {}", rstim::version());
}
```

Modify `Cargo.toml` to include library (default is fine, no change required beyond dependencies added later).

**Step 4: Run test to verify it passes**

Run: `cargo test -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/lib.rs src/main.rs tests/lib_smoke.rs
git commit -m "chore: add rstim library skeleton"
```

### Task 2: Define core IR and targets

**Files:**
- Create: `src/ir.rs`
- Modify: `src/lib.rs`
- Test: `tests/ir_smoke.rs`

**Step 1: Write the failing test**

Create `tests/ir_smoke.rs`:
```rust
use rstim::ir::{StimInstr, StimTarget, Annotation};

#[test]
fn build_simple_instr() {
    let instr = StimInstr::new("H", vec![], vec![StimTarget::Qubit(0)]);
    assert_eq!(instr.name, "H");
}

#[test]
fn build_detector_annotation() {
    let ann = Annotation::detector(vec![0.0, 1.0], vec![-1, -2]);
    assert_eq!(ann.rec_offsets.len(), 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -q`
Expected: FAIL with missing module `ir`.

**Step 3: Write minimal implementation**

Create `src/ir.rs`:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StimTarget {
    Qubit(u32),
    Rec(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StimInstr {
    pub name: String,
    pub tag: Option<String>,
    pub args: Vec<f64>,
    pub targets: Vec<StimTarget>,
}

impl StimInstr {
    pub fn new(name: &str, args: Vec<f64>, targets: Vec<StimTarget>) -> Self {
        Self { name: name.to_string(), tag: None, args, targets }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub kind: AnnotationKind,
    pub coords: Vec<f64>,
    pub rec_offsets: Vec<i32>,
    pub observable_index: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationKind {
    Detector,
    ObservableInclude,
}

impl Annotation {
    pub fn detector(coords: Vec<f64>, rec_offsets: Vec<i32>) -> Self {
        Self { kind: AnnotationKind::Detector, coords, rec_offsets, observable_index: None }
    }
    pub fn observable_include(index: u32, rec_offsets: Vec<i32>) -> Self {
        Self { kind: AnnotationKind::ObservableInclude, coords: vec![], rec_offsets, observable_index: Some(index) }
    }
}
```

Modify `src/lib.rs`:
```rust
pub mod ir;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/ir.rs src/lib.rs tests/ir_smoke.rs
git commit -m "feat: add stim IR types"
```

### Task 3: Implement measurement recorder

**Files:**
- Create: `src/recorder.rs`
- Modify: `src/lib.rs`
- Test: `tests/recorder.rs`

**Step 1: Write the failing test**

Create `tests/recorder.rs`:
```rust
use rstim::recorder::Recorder;

#[test]
fn recorder_rec_offsets_work() {
    let mut r = Recorder::default();
    r.push(false);
    r.push(true);
    assert_eq!(r.rec(-1).unwrap(), true);
    assert_eq!(r.rec(-2).unwrap(), false);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -q`
Expected: FAIL with missing module `recorder`.

**Step 3: Write minimal implementation**

Create `src/recorder.rs`:
```rust
#[derive(Default, Debug, Clone)]
pub struct Recorder {
    bits: Vec<bool>,
}

impl Recorder {
    pub fn push(&mut self, bit: bool) { self.bits.push(bit); }
    pub fn len(&self) -> usize { self.bits.len() }

    pub fn rec(&self, offset: i32) -> Option<bool> {
        if offset >= 0 { return None; }
        let idx = (self.bits.len() as i32) + offset;
        if idx < 0 { return None; }
        self.bits.get(idx as usize).copied()
    }
}
```

Modify `src/lib.rs`:
```rust
pub mod recorder;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/recorder.rs src/lib.rs tests/recorder.rs
git commit -m "feat: add measurement recorder"
```

### Task 4: Add a minimal `.stim` parser (subset)

**Files:**
- Create: `src/parser.rs`
- Modify: `src/lib.rs`
- Test: `tests/parser.rs`

**Step 1: Write the failing test**

Create `tests/parser.rs`:
```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -q`
Expected: FAIL with missing module `parser`.

**Step 3: Write minimal implementation**

Create `src/parser.rs`:
```rust
use crate::ir::{StimInstr, StimTarget};

pub fn parse_lines(input: &str) -> Result<Vec<StimInstr>, String> {
    let mut out = Vec::new();
    for (line_no, raw) in input.lines().enumerate() {
        let line = raw.split('#').next().unwrap().trim();
        if line.is_empty() { continue; }
        let mut parts = line.split_whitespace();
        let name = parts.next().ok_or_else(|| format!("line {}: empty", line_no + 1))?;
        let mut instr = StimInstr::new(name, vec![], vec![]);
        for token in parts {
            if let Some(t) = parse_target(token)? { instr.targets.push(t); }
        }
        out.push(instr);
    }
    Ok(out)
}

fn parse_target(token: &str) -> Result<Option<StimTarget>, String> {
    if token.starts_with("rec[") && token.ends_with(']') {
        let inner = &token[4..token.len()-1];
        let val: i32 = inner.parse().map_err(|_| format!("bad rec target {token}"))?;
        return Ok(Some(StimTarget::Rec(val)));
    }
    if let Ok(q) = token.parse::<u32>() {
        return Ok(Some(StimTarget::Qubit(q)));
    }
    Err(format!("unsupported target {token}"))
}
```

Modify `src/lib.rs`:
```rust
pub mod parser;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/parser.rs src/lib.rs tests/parser.rs
git commit -m "feat: add minimal stim parser"
```

### Task 5: Executor with yao-rs backend and detector/observable output

**Files:**
- Modify: `Cargo.toml`
- Create: `src/executor.rs`
- Modify: `src/lib.rs`
- Test: `tests/executor.rs`

**Step 1: Write the failing test**

Create `tests/executor.rs`:
```rust
use rand::SeedableRng;
use rand::rngs::StdRng;
use rstim::{executor::Executor, parser::parse_lines};

#[test]
fn detector_matches_measurement() {
    let program = "H 0\nM 0\nDETECTOR rec[-1]\n";
    let instrs = parse_lines(program).unwrap();
    let mut ex = Executor::from_instrs(instrs).unwrap();
    let mut rng = StdRng::seed_from_u64(1);
    let out = ex.run(&mut rng).unwrap();
    assert_eq!(out.detectors.len(), 1);
    assert_eq!(out.detectors[0], out.measurements[0]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -q`
Expected: FAIL with missing module `executor`.

**Step 3: Write minimal implementation**

Modify `Cargo.toml` to add dependencies:
```toml
[dependencies]
yao-rs = { path = "../yao-rs" }
rand = "0.8"
```

Create `src/executor.rs`:
```rust
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

        Ok(ExecOutput { measurements: recorder_bits(recorder), detectors, observables })
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
    let mut max_q = None;
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
    if targets.len() % 2 != 0 { return Err("odd number of targets".to_string()); }
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
```

Modify `src/lib.rs`:
```rust
pub mod executor;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml src/executor.rs src/lib.rs tests/executor.rs
git commit -m "feat: add executor with yao-rs backend"
```

### Task 6: Parse args for `OBSERVABLE_INCLUDE` and add error context

**Files:**
- Modify: `src/parser.rs`
- Test: `tests/parser_args.rs`

**Step 1: Write the failing test**

Create `tests/parser_args.rs`:
```rust
use rstim::parser::parse_lines;

#[test]
fn parses_observable_args() {
    let instrs = parse_lines("OBSERVABLE_INCLUDE(2) rec[-1]\n").unwrap();
    assert_eq!(instrs[0].args[0], 2.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -q`
Expected: FAIL with missing args parse.

**Step 3: Write minimal implementation**

Modify `src/parser.rs` to parse optional `(arg1,arg2)` right after name:
```rust
// after reading name
let (name, rest) = split_name_and_args(name)?;
let mut instr = StimInstr::new(name, args, vec![]);
```

Provide helper:
```rust
fn split_name_and_args(token: &str) -> Result<(&str, Vec<f64>), String> {
    if let Some(idx) = token.find('(') {
        let name = &token[..idx];
        let args_str = token[idx+1..token.len()-1].trim();
        let args = if args_str.is_empty() { vec![] } else {
            args_str.split(',').map(|s| s.trim().parse::<f64>()
                .map_err(|_| format!("bad arg {s}"))).collect::<Result<Vec<_>,_>>()?
        };
        Ok((name, args))
    } else {
        Ok((token, vec![]))
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/parser.rs tests/parser_args.rs
git commit -m "feat: parse instruction args"
```

---

Plan complete and saved to `docs/plans/2026-02-06-rstim-mvp-implementation.md`. Two execution options:

1. Subagent-Driven (this session)
2. Parallel Session (separate)

Which approach?
