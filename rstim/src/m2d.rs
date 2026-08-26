use crate::data_path::{build_reference_sample, ReferenceSampleMode};
use crate::ir::StimInstr;
use crate::measurement_transform::{CheckedMeasurementLayout, MeasurementTransformLimits};
use crate::sim::bit_table::BitTable;
use std::collections::BTreeMap;

pub struct M2dOutput {
    pub detections: BitTable,
    pub observable_flips: BitTable,
}

/// One loss-independent parity check for a single shot.
///
/// `source_detectors` identifies the original circuit detectors whose XOR
/// defines this check. A singleton is an unaffected original detector; two or
/// more sources form a supercheck.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LossAwareDetectorCheck {
    pub source_detectors: Vec<usize>,
    pub value: bool,
}

impl LossAwareDetectorCheck {
    pub fn is_supercheck(&self) -> bool {
        self.source_detectors.len() > 1
    }
}

/// Shot-conditioned detector information after removing lost measurement
/// degrees of freedom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LossAwareDetectorShot {
    /// Measurement-record indices known to be lost for this shot.
    pub lost_measurements: Vec<usize>,
    /// Fixed-width validity mask for the original circuit detectors.
    pub detector_valid: Vec<bool>,
    /// A maximal independent basis of loss-independent detector checks.
    pub checks: Vec<LossAwareDetectorCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LossAwareM2dOutput {
    pub shots: Vec<LossAwareDetectorShot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M2dOptions {
    pub reference_sample_mode: ReferenceSampleMode,
    pub ran_without_feedback: bool,
}

impl Default for M2dOptions {
    fn default() -> Self {
        Self {
            reference_sample_mode: ReferenceSampleMode::SimulateNoiseless,
            ran_without_feedback: false,
        }
    }
}

/// Convert raw measurement bits to detection events.
/// meas_table: BitTable(num_measurements, num_shots)
/// Returns M2dOutput with detections and observable_flips BitTables.
pub fn measurements_to_detections(
    instrs: &[StimInstr],
    meas_table: &BitTable,
) -> Result<M2dOutput, String> {
    measurements_to_detections_with_options(instrs, meas_table, None, M2dOptions::default())
}

pub fn measurements_to_detections_with_options(
    instrs: &[StimInstr],
    meas_table: &BitTable,
    sweep_table: Option<&BitTable>,
    options: M2dOptions,
) -> Result<M2dOutput, String> {
    let normalization = if options.ran_without_feedback {
        Some(crate::transforms::normalize_feedbackless_m2d(instrs)?)
    } else {
        None
    };
    let work_instrs = normalization
        .as_ref()
        .map(|normalized| normalized.circuit.as_slice())
        .unwrap_or(instrs);
    let empty_corrections: &[Vec<usize>] = &[];
    let measurement_corrections = normalization
        .as_ref()
        .map(|normalized| normalized.measurement_corrections.as_slice())
        .unwrap_or(empty_corrections);

    let shared_reference = match options.reference_sample_mode {
        ReferenceSampleMode::SimulateNoiseless if sweep_table.is_none() => Some(
            build_reference_sample(work_instrs, ReferenceSampleMode::SimulateNoiseless)?,
        ),
        ReferenceSampleMode::AssumeAllZero => {
            Some(vec![false; crate::stats::num_measurements(work_instrs)])
        }
        ReferenceSampleMode::SimulateNoiseless => None,
    };
    measurements_to_detections_impl(
        work_instrs,
        meas_table,
        sweep_table,
        shared_reference.as_deref(),
        measurement_corrections,
    )
}

/// Convert measurements containing interleaved loss-visible `flag,value`
/// records into shot-conditioned detectors and superchecks.
///
/// A set flag marks its paired value record as lost. Detectors and observables
/// may not use flag records as parity terms because flags are metadata, not
/// stabilizer measurements.
pub fn measurements_to_loss_aware_detections(
    instrs: &[StimInstr],
    meas_table: &BitTable,
) -> Result<LossAwareM2dOutput, String> {
    let layout = CheckedMeasurementLayout::from_circuit_with_limits(
        instrs,
        MeasurementTransformLimits::default(),
    )
    .map_err(|err| err.to_string())?;
    validate_loss_flag_references(&layout)?;
    let loss_mask = loss_mask_from_loss_visible_measurements(&layout, meas_table)?;
    measurements_to_loss_aware_detections_with_layout(instrs, meas_table, &loss_mask, &layout)
}

/// Convert measurements with an explicit, same-shape measurement-loss mask.
///
/// This entry point supports circuits whose loss metadata arrives out of band.
/// A `true` mask entry means the corresponding measurement value is unknown;
/// its stored 0/1 bit is only a placeholder.
pub fn measurements_to_loss_aware_detections_with_loss_mask(
    instrs: &[StimInstr],
    meas_table: &BitTable,
    measurement_loss_mask: &BitTable,
) -> Result<LossAwareM2dOutput, String> {
    let layout = CheckedMeasurementLayout::from_circuit_with_limits(
        instrs,
        MeasurementTransformLimits::default(),
    )
    .map_err(|err| err.to_string())?;
    validate_loss_flag_references(&layout)?;
    measurements_to_loss_aware_detections_with_layout(
        instrs,
        meas_table,
        measurement_loss_mask,
        &layout,
    )
}

fn measurements_to_loss_aware_detections_with_layout(
    instrs: &[StimInstr],
    meas_table: &BitTable,
    measurement_loss_mask: &BitTable,
    layout: &CheckedMeasurementLayout,
) -> Result<LossAwareM2dOutput, String> {
    validate_loss_mask_shape(meas_table, measurement_loss_mask, layout.num_measurements())?;
    for pair in layout.loss_visible_measurements() {
        for shot in 0..measurement_loss_mask.num_minor() {
            if measurement_loss_mask.get(pair.flag, shot) {
                return Err(format!(
                    "measurement_loss_mask marks loss-flag record {} as lost; mark its value record {} instead",
                    pair.flag, pair.value
                ));
            }
        }
    }

    let raw = measurements_to_detections(instrs, meas_table)?;
    let mut shots = Vec::with_capacity(meas_table.num_minor());
    for shot in 0..meas_table.num_minor() {
        shots.push(build_loss_aware_shot(
            layout.detector_rows(),
            &raw.detections,
            measurement_loss_mask,
            shot,
        ));
    }
    Ok(LossAwareM2dOutput { shots })
}

fn loss_mask_from_loss_visible_measurements(
    layout: &CheckedMeasurementLayout,
    meas_table: &BitTable,
) -> Result<BitTable, String> {
    if meas_table.num_major() != layout.num_measurements() {
        return Err(format!(
            "meas_table has {} bits but circuit has {} measurements",
            meas_table.num_major(),
            layout.num_measurements()
        ));
    }
    let mut mask = BitTable::try_new(meas_table.num_major(), meas_table.num_minor())
        .map_err(|err| format!("failed to allocate measurement loss mask: {err:?}"))?;
    for pair in layout.loss_visible_measurements() {
        for shot in 0..meas_table.num_minor() {
            if meas_table.get(pair.flag, shot) {
                mask.set(pair.value, shot, true);
            }
        }
    }
    Ok(mask)
}

fn validate_loss_flag_references(layout: &CheckedMeasurementLayout) -> Result<(), String> {
    for pair in layout.loss_visible_measurements() {
        if let Some(detector) = layout
            .detector_rows()
            .iter()
            .position(|row| row.binary_search(&pair.flag).is_ok())
        {
            return Err(format!(
                "detector {detector} references loss-flag record {}; detectors must reference measurement values",
                pair.flag
            ));
        }
        if let Some(observable) = layout
            .observable_rows()
            .iter()
            .position(|row| row.binary_search(&pair.flag).is_ok())
        {
            return Err(format!(
                "observable {observable} references loss-flag record {}; observables must reference measurement values",
                pair.flag
            ));
        }
    }
    Ok(())
}

fn validate_loss_mask_shape(
    meas_table: &BitTable,
    loss_mask: &BitTable,
    expected_measurements: usize,
) -> Result<(), String> {
    if meas_table.num_major() != expected_measurements {
        return Err(format!(
            "meas_table has {} bits but circuit has {expected_measurements} measurements",
            meas_table.num_major()
        ));
    }
    if loss_mask.num_major() != expected_measurements {
        return Err(format!(
            "measurement_loss_mask has {} bits but circuit has {expected_measurements} measurements",
            loss_mask.num_major()
        ));
    }
    if loss_mask.num_minor() != meas_table.num_minor() {
        return Err(format!(
            "measurement_loss_mask has {} shots but meas_table has {} shots",
            loss_mask.num_minor(),
            meas_table.num_minor()
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct LossPivot {
    measurement_terms: Vec<usize>,
    detector_sources: Vec<usize>,
}

fn build_loss_aware_shot(
    detector_rows: &[Vec<usize>],
    raw_detections: &BitTable,
    loss_mask: &BitTable,
    shot: usize,
) -> LossAwareDetectorShot {
    let lost_measurements: Vec<usize> = (0..loss_mask.num_major())
        .filter(|&measurement| loss_mask.get(measurement, shot))
        .collect();
    let mut detector_valid = Vec::with_capacity(detector_rows.len());
    let mut pivots = BTreeMap::<usize, LossPivot>::new();
    let mut checks = Vec::new();

    for (detector, row) in detector_rows.iter().enumerate() {
        let mut measurement_terms: Vec<usize> = row
            .iter()
            .copied()
            .filter(|&measurement| loss_mask.get(measurement, shot))
            .collect();
        detector_valid.push(measurement_terms.is_empty());
        let mut detector_sources = vec![detector];
        let mut became_pivot = false;

        while let Some(&pivot_measurement) = measurement_terms.first() {
            let Some(pivot) = pivots.get(&pivot_measurement) else {
                pivots.insert(
                    pivot_measurement,
                    LossPivot {
                        measurement_terms: std::mem::take(&mut measurement_terms),
                        detector_sources: std::mem::take(&mut detector_sources),
                    },
                );
                became_pivot = true;
                break;
            };
            measurement_terms = symmetric_difference(&measurement_terms, &pivot.measurement_terms);
            detector_sources = symmetric_difference(&detector_sources, &pivot.detector_sources);
        }

        if !became_pivot {
            let value = detector_sources
                .iter()
                .fold(false, |acc, &source| acc ^ raw_detections.get(source, shot));
            checks.push(LossAwareDetectorCheck {
                source_detectors: detector_sources,
                value,
            });
        }
    }

    LossAwareDetectorShot {
        lost_measurements,
        detector_valid,
        checks,
    }
}

fn symmetric_difference(left: &[usize], right: &[usize]) -> Vec<usize> {
    let mut result = Vec::with_capacity(left.len() + right.len());
    let (mut a, mut b) = (0, 0);
    while a < left.len() || b < right.len() {
        match (left.get(a), right.get(b)) {
            (Some(&x), Some(&y)) if x == y => {
                a += 1;
                b += 1;
            }
            (Some(&x), Some(&y)) if x < y => {
                result.push(x);
                a += 1;
            }
            (Some(_), Some(&y)) => {
                result.push(y);
                b += 1;
            }
            (Some(&x), None) => {
                result.push(x);
                a += 1;
            }
            (None, Some(&y)) => {
                result.push(y);
                b += 1;
            }
            (None, None) => break,
        }
    }
    result
}

pub(crate) fn measurements_to_detections_with_reference(
    instrs: &[StimInstr],
    meas_table: &BitTable,
    reference_sample: &[bool],
) -> Result<M2dOutput, String> {
    measurements_to_detections_impl(instrs, meas_table, None, Some(reference_sample), &[])
}

fn measurements_to_detections_impl(
    work_instrs: &[StimInstr],
    meas_table: &BitTable,
    sweep_table: Option<&BitTable>,
    shared_reference: Option<&[bool]>,
    measurement_corrections: &[Vec<usize>],
) -> Result<M2dOutput, String> {
    let layout = CheckedMeasurementLayout::from_circuit_with_limits(
        work_instrs,
        MeasurementTransformLimits::default(),
    )
    .map_err(|err| err.to_string())?;
    let n_meas = layout.num_measurements();
    if let Some(reference) = shared_reference {
        if reference.len() != n_meas {
            return Err(reference_measurement_count_mismatch(
                reference.len(),
                n_meas,
            ));
        }
    }
    let n_shots = meas_table.num_minor();

    if meas_table.num_major() != n_meas {
        return Err(format!(
            "meas_table has {} bits but circuit has {} measurements",
            meas_table.num_major(),
            n_meas
        ));
    }
    if let Some(sweep_table) = sweep_table {
        if sweep_table.num_minor() != n_shots {
            return Err(format!(
                "sweep shots {} do not match measurement shots {}",
                sweep_table.num_minor(),
                n_shots
            ));
        }
    }

    let n_dets = layout.num_detectors();
    let n_obs = layout.num_observables();

    let mut dets = BitTable::try_new(n_dets, n_shots)
        .map_err(|err| format!("failed to allocate detection table: {err:?}"))?;
    let mut obs = BitTable::try_new(n_obs, n_shots)
        .map_err(|err| format!("failed to allocate observable table: {err:?}"))?;

    for shot in 0..n_shots {
        let per_shot_reference;
        let reference = if let Some(reference) = shared_reference {
            reference
        } else {
            let sweep_table = sweep_table.expect("validated above");
            let sweep_row: Vec<bool> = (0..sweep_table.num_major())
                .map(|i| sweep_table.get(i, shot))
                .collect();
            per_shot_reference =
                crate::executor::reference_sample_with_sweep_bits(work_instrs, Some(&sweep_row))?;
            per_shot_reference.as_slice()
        };
        let mut flips = Vec::with_capacity(n_meas);
        for i in 0..n_meas {
            let mut flip = meas_table.get(i, shot) ^ reference[i];
            if let Some(extra_terms) = measurement_corrections.get(i) {
                for &j in extra_terms {
                    flip ^= flips[j];
                }
            }
            flips.push(flip);
        }

        for (d, rec_offsets) in layout.detector_rows().iter().enumerate() {
            let val = rec_offsets.iter().fold(false, |acc, &r| acc ^ flips[r]);
            if val {
                dets.set(d, shot, true);
            }
        }
        for (o, rec_offsets) in layout.observable_rows().iter().enumerate() {
            let val = rec_offsets.iter().fold(false, |acc, &r| acc ^ flips[r]);
            if val {
                obs.set(o, shot, true);
            }
        }
    }

    Ok(M2dOutput {
        detections: dets,
        observable_flips: obs,
    })
}

fn reference_measurement_count_mismatch(reference_len: usize, n_meas: usize) -> String {
    format!("reference has {reference_len} bits but circuit has {n_meas} measurements")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precomputed_reference_is_used_instead_of_rebuilding_from_the_circuit() {
        let instrs = crate::parser::parse_lines("X 0\nM 0\nDETECTOR rec[-1]\n").unwrap();
        let mut measurements = BitTable::try_new(1, 1).unwrap();
        measurements.set(0, 0, true);

        let output =
            measurements_to_detections_with_reference(&instrs, &measurements, &[false]).unwrap();

        assert!(output.detections.get(0, 0));
    }

    #[test]
    fn reference_measurement_count_mismatch_is_actionable() {
        assert_eq!(
            reference_measurement_count_mismatch(2, 3),
            "reference has 2 bits but circuit has 3 measurements"
        );
    }
}
