use rand::Rng;

use crate::compiled::CompiledCircuit;
use crate::data_path::build_reference_sample;
use crate::m2d::{M2dOptions, measurements_to_detections_with_options};
use crate::sampler::{BatchOutput, SampleOptions};
use crate::sim::frame::FrameSimulator;

pub fn sample_compiled_batch(
    compiled: &CompiledCircuit,
    n_shots: usize,
    rng: &mut impl Rng,
    options: SampleOptions,
) -> Result<BatchOutput, String> {
    let ref_sample = build_reference_sample(&compiled.source, options.reference_sample_mode)?;
    let mut frame = FrameSimulator::new(compiled.num_qubits, n_shots);
    frame.run_compiled_blocks(&compiled.blocks, &ref_sample, rng)?;

    let measurements = frame.measurements(&ref_sample);
    let m2d = measurements_to_detections_with_options(
        &compiled.source,
        &measurements,
        None,
        M2dOptions {
            reference_sample_mode: options.reference_sample_mode,
            ran_without_feedback: false,
        },
    )?;

    Ok(BatchOutput {
        measurements,
        detections: m2d.detections,
        observable_flips: m2d.observable_flips,
    })
}
