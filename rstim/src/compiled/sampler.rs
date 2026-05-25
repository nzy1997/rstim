use rand::Rng;

use crate::compiled::CompiledCircuit;
use crate::sampler::{BatchOutput, SampleOptions};

pub fn sample_compiled_batch(
    _compiled: &CompiledCircuit,
    _n_shots: usize,
    _rng: &mut impl Rng,
    _options: SampleOptions,
) -> Result<BatchOutput, String> {
    Err("compiled sampler not implemented yet".to_string())
}
