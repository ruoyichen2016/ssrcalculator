//! Regime-aware modern prediction.
//!
//! The modern default is the Holtrop-Mennen resistance/power model with a
//! per-ship-type calibration factor fitted to the public historical dataset.
//! This beats the reconstructed original SpringSharp model and the reduced/
//! structural ensemble on *nested* leave-one-out cross-validation, and it maps
//! directly onto the existing engine (set `resistance_model` + `power_factor`).
//!
//! The original SpringSharp model is retained as a comparison baseline; see
//! [`RegimeLoocv::original_baseline_mape`].

use crate::accuracy::{historical_cases, simulate_case, simulate_case_with_model, HistoricalCase};
use crate::model::{ResistanceModel, ShipModel};

/// One row of the production regime table.
#[derive(Clone, Copy, Debug)]
pub struct RegimeCalibration {
    pub ship_type: &'static str,
    pub count: usize,
    pub model: ResistanceModel,
    pub model_label: &'static str,
    /// Calibration factor applied to the base model (`reference / simulated`).
    pub factor: f64,
    /// In-sample MAPE after the factor (diagnostic only).
    pub train_mape: f64,
}

fn raw_original(case: &HistoricalCase) -> f64 {
    simulate_case(case).engine.hp_max
}

fn raw_holtrop(case: &HistoricalCase) -> f64 {
    simulate_case_with_model(case, ResistanceModel::HoltropMennen)
        .engine
        .hp_max
}

/// Mean `reference / simulated` factor over a set of cases.
fn factor_for(cases: &[&HistoricalCase], power: impl Fn(&HistoricalCase) -> f64) -> f64 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for case in cases {
        let p = power(case);
        if case.reference_hp > 0.0 && p > 0.0 {
            sum += case.reference_hp / p;
            n += 1.0;
        }
    }
    if n > 0.0 {
        sum / n
    } else {
        1.0
    }
}

fn mape(cases: &[&HistoricalCase], factor: f64, power: impl Fn(&HistoricalCase) -> f64) -> f64 {
    if cases.is_empty() {
        return f64::NAN;
    }
    cases
        .iter()
        .map(|c| ((power(c) * factor - c.reference_hp) / c.reference_hp).abs())
        .sum::<f64>()
        / cases.len() as f64
}

/// Fit the production regime table on all built-in cases.
///
/// The base model is Holtrop-Mennen for every ship type; the factor corrects
/// the systematic bias of that model against the public historical references.
pub fn fit_regime_calibration() -> Vec<RegimeCalibration> {
    let cases = historical_cases();
    let mut types: Vec<&'static str> = cases.iter().map(|c| c.ship_type).collect();
    types.sort_unstable();
    types.dedup();
    types
        .into_iter()
        .filter_map(|ty| {
            let subset: Vec<&HistoricalCase> = cases.iter().filter(|c| c.ship_type == ty).collect();
            if subset.is_empty() {
                return None;
            }
            let factor = factor_for(&subset, raw_holtrop);
            let train = mape(&subset, factor, raw_holtrop);
            Some(RegimeCalibration {
                ship_type: ty,
                count: subset.len(),
                model: ResistanceModel::HoltropMennen,
                model_label: "Holtrop-Mennen",
                factor,
                train_mape: train,
            })
        })
        .collect()
}

/// Regime row for a ship type, if known.  Matching is case-insensitive and
/// tolerates qualifiers (e.g. "Battleship" matches "battleship").
pub fn regime_for_ship_type(ship_type: &str) -> Option<RegimeCalibration> {
    let needle = ship_type.to_ascii_lowercase();
    fit_regime_calibration()
        .into_iter()
        .find(|r| needle.contains(r.ship_type) || r.ship_type.contains(needle.trim()))
}

/// Apply the regime model + factor to a design.
pub fn apply_regime_calibration(ship: &mut ShipModel) -> Option<RegimeCalibration> {
    let entry = regime_for_ship_type(&ship.ship_type)?;
    ship.engine.resistance_model = entry.model;
    ship.engine.power_factor = entry.factor;
    ship.calculate();
    Some(entry)
}

/// Nested-LOOCV result for the regime model.
#[derive(Clone, Debug)]
pub struct RegimeLoocv {
    /// Honest out-of-sample MAPE of Holtrop + per-type factor.
    pub overall_mape: f64,
    pub per_type: Vec<(String, usize, f64)>,
    /// Same nested protocol for original + per-type factor (comparison baseline).
    pub original_baseline_mape: f64,
    /// Holtrop with no calibration factor (nested protocol), for reference.
    pub holtrop_uncalibrated_mape: f64,
}

/// Nested leave-one-out evaluation.
///
/// For every held-out case the factor is fitted on the remaining cases of the
/// same ship type (falling back to all training cases when fewer than two
/// same-type cases remain), so the reported error does not use the test case.
pub fn run_loocv_regime() -> RegimeLoocv {
    let cases = historical_cases();
    let raw_h: Vec<f64> = cases.iter().map(raw_holtrop).collect();
    let raw_o: Vec<f64> = cases.iter().map(raw_original).collect();

    let mut errs = Vec::new();
    let mut holt_uncal = Vec::new();
    let mut orig_errs = Vec::new();
    let mut per_type: Vec<(String, usize, f64)> = Vec::new();

    for i in 0..cases.len() {
        let test = &cases[i];
        let train: Vec<&HistoricalCase> = cases
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, c)| c)
            .collect();
        let same: Vec<&HistoricalCase> = train
            .iter()
            .copied()
            .filter(|c| c.ship_type == test.ship_type)
            .collect();
        let pool = if same.len() >= 2 { &same } else { &train };

        let fh = factor_for(pool, raw_holtrop);
        let fo = factor_for(pool, raw_original);
        let pred_h = raw_h[i] * fh;
        let pred_o = raw_o[i] * fo;
        let eh = ((pred_h - test.reference_hp) / test.reference_hp).abs();
        let eo = ((pred_o - test.reference_hp) / test.reference_hp).abs();
        let eu = ((raw_h[i] - test.reference_hp) / test.reference_hp).abs();

        errs.push(eh);
        orig_errs.push(eo);
        holt_uncal.push(eu);

        let ty = test.ship_type.to_string();
        if let Some(entry) = per_type.iter_mut().find(|(t, _, _)| *t == ty) {
            entry.1 += 1;
            entry.2 += eh;
        } else {
            per_type.push((ty, 1, eh));
        }
    }
    for entry in &mut per_type {
        entry.2 /= entry.1 as f64;
    }
    per_type.sort_by(|a, b| a.0.cmp(&b.0));

    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    RegimeLoocv {
        overall_mape: mean(&errs),
        per_type,
        original_baseline_mape: mean(&orig_errs),
        holtrop_uncalibrated_mape: mean(&holt_uncal),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regime_table_is_positive_and_covers_all_types() {
        let table = fit_regime_calibration();
        assert_eq!(table.len(), 4);
        for row in &table {
            assert!(row.factor.is_finite() && row.factor > 0.0);
            assert!(row.train_mape.is_finite() && row.train_mape >= 0.0);
            assert_eq!(row.model, ResistanceModel::HoltropMennen);
        }
    }

    #[test]
    fn nested_loocv_beats_comparison_baselines() {
        let r = run_loocv_regime();
        assert!(r.overall_mape.is_finite());
        assert!(r.overall_mape <= 0.095, "regime MAPE {:.3}", r.overall_mape);
        assert!(r.overall_mape < r.original_baseline_mape);
        assert!(r.overall_mape < r.holtrop_uncalibrated_mape);
        let total: usize = r.per_type.iter().map(|(_, n, _)| *n).sum();
        assert_eq!(total, historical_cases().len());
    }

    #[test]
    fn ship_type_matching_is_case_insensitive() {
        assert!(regime_for_ship_type("Battleship").is_some());
        assert!(regime_for_ship_type("destroyer").is_some());
        assert!(regime_for_ship_type("submarine").is_none());
    }
}
