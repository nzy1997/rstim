use crate::error::DecodeError;
use crate::gf2::PreparedLinearSystem;
use crate::matrix::ParityCheckMatrix;
use crate::vector::{Correction, Syndrome};

#[derive(Debug)]
pub(crate) struct LsdWorkspace {
    prepared: PreparedLinearSystem,
    column_order: Vec<usize>,
    local_rows: Vec<Vec<usize>>,
    local_to_global_bits: Vec<usize>,
    local_to_global_checks: Vec<usize>,
    local_reliability: Vec<f64>,
    candidate_bits: Vec<bool>,
}

impl LsdWorkspace {
    pub(crate) fn new(pcm: &ParityCheckMatrix) -> Self {
        Self {
            prepared: PreparedLinearSystem::from_pcm(pcm),
            column_order: (0..pcm.num_bits()).collect(),
            local_rows: Vec::new(),
            local_to_global_bits: Vec::new(),
            local_to_global_checks: Vec::new(),
            local_reliability: Vec::new(),
            candidate_bits: vec![false; pcm.num_bits()],
        }
    }
}

pub(crate) fn decode_lsd_with_workspace(
    pcm: &ParityCheckMatrix,
    target_syndrome: &Syndrome,
    reliability: &[f64],
    lsd_order: usize,
    workspace: &mut LsdWorkspace,
) -> Result<Correction, DecodeError> {
    debug_assert_eq!(target_syndrome.len(), pcm.num_checks());
    debug_assert_eq!(reliability.len(), pcm.num_bits());
    let _ = (
        &workspace.local_rows,
        &workspace.local_to_global_bits,
        &workspace.local_to_global_checks,
        &workspace.local_reliability,
        &workspace.candidate_bits,
    );

    match lsd_order {
        0 => solve_order_zero(pcm, target_syndrome, reliability, workspace),
        1 => solve_order_zero(pcm, target_syndrome, reliability, workspace),
        _ => Err(DecodeError::UnsupportedLsdOrder { order: lsd_order }),
    }
}

fn solve_order_zero(
    pcm: &ParityCheckMatrix,
    target_syndrome: &Syndrome,
    reliability: &[f64],
    workspace: &mut LsdWorkspace,
) -> Result<Correction, DecodeError> {
    sort_unreliable_columns(&mut workspace.column_order, reliability);
    workspace
        .prepared
        .solve_with_column_order(target_syndrome, &workspace.column_order)
        .map_err(|_| DecodeError::NoLsdSolution)
        .and_then(|correction| verify_residual(pcm, target_syndrome, correction))
}

fn sort_unreliable_columns(column_order: &mut Vec<usize>, reliability: &[f64]) {
    column_order.clear();
    column_order.extend(0..reliability.len());
    column_order.sort_by(|&a, &b| {
        reliability[a]
            .partial_cmp(&reliability[b])
            .unwrap()
            .then_with(|| a.cmp(&b))
    });
}

fn verify_residual(
    pcm: &ParityCheckMatrix,
    target_syndrome: &Syndrome,
    correction: Correction,
) -> Result<Correction, DecodeError> {
    if pcm.multiply(&correction) == *target_syndrome {
        Ok(correction)
    } else {
        Err(DecodeError::NoLsdSolution)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn correction_cost(bits: &[bool], reliability: &[f64]) -> f64 {
    bits.iter()
        .zip(reliability.iter())
        .filter_map(|(&bit, &cost)| bit.then_some(cost))
        .sum()
}

#[cfg_attr(not(test), allow(dead_code))]
fn is_better_candidate(candidate: &[bool], best: &[bool], reliability: &[f64]) -> bool {
    let candidate_cost = correction_cost(candidate, reliability);
    let best_cost = correction_cost(best, reliability);
    if candidate_cost < best_cost - f64::EPSILON {
        return true;
    }
    if (candidate_cost - best_cost).abs() <= f64::EPSILON {
        return candidate < best;
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::error::DecodeError;
    use crate::matrix::ParityCheckMatrix;
    use crate::vector::{Correction, Syndrome};

    use super::{LsdWorkspace, decode_lsd_with_workspace, is_better_candidate};

    #[test]
    fn order_zero_matches_existing_reliability_ordered_residual_solve() {
        let pcm = ParityCheckMatrix::from_sparse_rows(2, 3, vec![vec![1, 2], vec![0]]).unwrap();
        let target_syndrome = Syndrome::from(vec![true, false]);
        let reliability = vec![1.0, 0.2, 0.4];
        let mut workspace = LsdWorkspace::new(&pcm);

        let correction =
            decode_lsd_with_workspace(&pcm, &target_syndrome, &reliability, 0, &mut workspace)
                .unwrap();

        assert_eq!(pcm.multiply(&correction), target_syndrome);
    }

    #[test]
    fn order_zero_maps_unsatisfiable_system_to_lsd_failure() {
        let pcm = ParityCheckMatrix::from_sparse_rows(2, 1, vec![vec![0], vec![0]]).unwrap();
        let target_syndrome = Syndrome::from(vec![true, false]);
        let reliability = vec![1.0];
        let mut workspace = LsdWorkspace::new(&pcm);

        let error =
            decode_lsd_with_workspace(&pcm, &target_syndrome, &reliability, 0, &mut workspace)
                .unwrap_err();

        assert_eq!(error, DecodeError::NoLsdSolution);
    }

    #[test]
    fn candidate_tie_break_prefers_lexicographically_smaller_bits() {
        let candidate = vec![false, false, true];
        let best = vec![false, true, false];
        let reliability = vec![1.0, 0.0, 0.0];

        assert!(is_better_candidate(&candidate, &best, &reliability));
    }

    #[test]
    fn candidate_cost_break_prefers_lower_reliability_weight() {
        let candidate = vec![false, false, true];
        let best = vec![false, true, false];
        let reliability = vec![1.0, 0.5, 0.1];

        assert!(is_better_candidate(&candidate, &best, &reliability));
    }

    #[test]
    fn candidate_cost_break_rejects_higher_reliability_weight() {
        let candidate = vec![false, false, true];
        let best = vec![false, true, false];
        let reliability = vec![1.0, 0.1, 0.5];

        assert!(!is_better_candidate(&candidate, &best, &reliability));
    }

    #[test]
    fn correction_cost_uses_only_set_bits() {
        let bits = Correction::from(vec![true, false, true]);
        let reliability = vec![0.25, 100.0, 0.75];

        assert_eq!(super::correction_cost(bits.as_slice(), &reliability), 1.0);
    }
}
