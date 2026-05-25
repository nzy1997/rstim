use crate::ir::{StimInstr, StimTarget};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompiledFeatureFlags {
    pub has_loss: bool,
    pub has_feedback: bool,
    pub has_nested_repeat: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledOp {
    pub name: String,
    pub args: Vec<f64>,
    pub targets: Vec<StimTarget>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledRepeatRegion {
    pub count: u64,
    pub body: Vec<CompiledBlock>,
    pub body_source: Vec<StimInstr>,
    pub measurement_span: usize,
    pub detector_span: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompiledBlock {
    Ops(Vec<CompiledOp>),
    Repeat(CompiledRepeatRegion),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledCircuit {
    pub source: Vec<StimInstr>,
    pub blocks: Vec<CompiledBlock>,
    pub flags: CompiledFeatureFlags,
    pub num_qubits: usize,
    pub num_measurements: usize,
    pub num_detectors: usize,
    pub num_observables: usize,
}

pub fn compile_circuit(instrs: &[StimInstr]) -> Result<CompiledCircuit, String> {
    Ok(CompiledCircuit {
        source: instrs.to_vec(),
        blocks: Vec::new(),
        flags: CompiledFeatureFlags::default(),
        num_qubits: 0,
        num_measurements: 0,
        num_detectors: 0,
        num_observables: 0,
    })
}
