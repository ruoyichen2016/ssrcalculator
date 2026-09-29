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

use crate::accuracy::{
    case_confidence, historical_cases, simulate_case, simulate_case_with_model, HistoricalCase,
};
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

/// Factor fraction trimmed from each tail before averaging.
///
/// A 15% trimmed mean rejects the influence of individual outlier cases.  A
/// nested-LOOCV sweep over 5-20% trims (and the geometric mean) showed a stable
/// ~0.13pp improvement over the plain mean; see docs/VALIDATION.md.
const FACTOR_TRIM: f64 = 0.15;

/// Robust `reference / simulated` factor: 15% trimmed mean with curated cases
/// weighted 2x relative to the approximate bulk.  Confidence weighting improved
/// both the full-set and curated-core nested LOOCV (docs/VALIDATION.md).
fn factor_for(cases: &[&HistoricalCase], power: impl Fn(&HistoricalCase) -> f64) -> f64 {
    let mut pairs: Vec<(f64, f64)> = cases
        .iter()
        .filter_map(|case| {
            let p = power(case);
            if case.reference_hp > 0.0 && p > 0.0 {
                let weight = if case.provenance.is_curated() {
                    2.0
                } else {
                    1.0
                };
                Some((case.reference_hp / p, weight))
            } else {
                None
            }
        })
        .collect();
    if pairs.is_empty() {
        return 1.0;
    }
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let total_weight: f64 = pairs.iter().map(|(_, w)| w).sum();
    let trim = total_weight * FACTOR_TRIM;
    // Trim by weight from each tail.
    let mut acc = 0.0;
    let mut start = 0;
    while start < pairs.len() && acc < trim {
        acc += pairs[start].1;
        start += 1;
    }
    let mut acc = 0.0;
    let mut end = pairs.len();
    while end > start && acc < trim {
        end -= 1;
        acc += pairs[end].1;
    }
    let slice = if end > start {
        &pairs[start..end]
    } else {
        &pairs[..]
    };
    let weight_sum: f64 = slice.iter().map(|(_, w)| w).sum();
    if weight_sum > 1e-9 {
        slice.iter().map(|(r, w)| r * w).sum::<f64>() / weight_sum
    } else {
        pairs.iter().map(|(r, _)| r).sum::<f64>() / pairs.len() as f64
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

/// One nested-LOOCV residual for a historical case.
#[derive(Clone, Debug)]
pub struct RegimeResidual {
    pub name: &'static str,
    pub ship_type: &'static str,
    pub reference_hp: f64,
    pub predicted_hp: f64,
    pub error_pct: f64,
    /// Declared data confidence (0.7 curated core, 0.5 approximate bulk).
    pub confidence: f64,
}

/// Nested-LOOCV result for the regime model.
#[derive(Clone, Debug)]
pub struct RegimeLoocv {
    /// Honest out-of-sample MAPE of Holtrop + per-type factor (case-weighted).
    pub overall_mape: f64,
    /// Type-averaged MAPE (each ship type weighted equally).
    pub macro_mape: f64,
    pub per_type: Vec<(String, usize, f64)>,
    /// Original SpringSharp model with no calibration factor (raw baseline).
    pub original_uncalibrated_mape: f64,
    /// Same nested protocol for original + per-type factor (comparison baseline).
    pub original_baseline_mape: f64,
    /// Holtrop with no calibration factor (nested protocol), for reference.
    pub holtrop_uncalibrated_mape: f64,
    /// Per-case nested-LOOCV residuals.
    pub residuals: Vec<RegimeResidual>,
    /// Cases in the curated core (declared confidence >= 0.65).
    pub curated_cases: usize,
    /// Nested-LOOCV MAPE over the curated core only.
    pub curated_mape: f64,
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
    let mut orig_uncal = Vec::new();
    let mut orig_errs = Vec::new();
    let mut per_type: Vec<(String, usize, f64)> = Vec::new();
    let mut residuals: Vec<RegimeResidual> = Vec::new();

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
        let eou = ((raw_o[i] - test.reference_hp) / test.reference_hp).abs();

        errs.push(eh);
        orig_errs.push(eo);
        holt_uncal.push(eu);
        orig_uncal.push(eou);
        residuals.push(RegimeResidual {
            name: test.name,
            ship_type: test.ship_type,
            reference_hp: test.reference_hp,
            predicted_hp: pred_h,
            error_pct: (pred_h - test.reference_hp) / test.reference_hp.max(1e-9) * 100.0,
            confidence: case_confidence(test.name),
        });

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
    let curated: Vec<f64> = residuals
        .iter()
        .filter(|r| r.confidence >= 0.65)
        .map(|r| (r.error_pct / 100.0).abs())
        .collect();
    let curated_cases = curated.len();
    let curated_mape = if curated.is_empty() {
        f64::NAN
    } else {
        curated.iter().sum::<f64>() / curated.len() as f64
    };
    let macro_mape = if per_type.is_empty() {
        f64::NAN
    } else {
        per_type.iter().map(|(_, _, m)| *m).sum::<f64>() / per_type.len() as f64
    };
    RegimeLoocv {
        overall_mape: mean(&errs),
        macro_mape,
        per_type,
        original_uncalibrated_mape: mean(&orig_uncal),
        original_baseline_mape: mean(&orig_errs),
        holtrop_uncalibrated_mape: mean(&holt_uncal),
        residuals,
        curated_cases,
        curated_mape,
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
        assert!(r.overall_mape <= 0.115, "regime MAPE {:.3}", r.overall_mape);
        assert!(r.overall_mape < r.original_uncalibrated_mape);
        assert!(r.overall_mape < r.original_baseline_mape);
        assert!(r.overall_mape < r.holtrop_uncalibrated_mape);
        let total: usize = r.per_type.iter().map(|(_, n, _)| *n).sum();
        assert_eq!(total, historical_cases().len());
        assert!(r.curated_cases > 0 && r.curated_cases < r.residuals.len());
        assert!(
            r.curated_mape <= r.overall_mape,
            "curated core {:.3} should not be worse than the full set {:.3}",
            r.curated_mape,
            r.overall_mape
        );
    }

    #[test]
    fn nested_residuals_cover_every_case() {
        let r = run_loocv_regime();
        assert_eq!(r.residuals.len(), historical_cases().len());
        for row in &r.residuals {
            assert!(row.predicted_hp.is_finite() && row.predicted_hp > 0.0);
            assert!(row.error_pct.is_finite());
        }
    }

    #[test]
    fn ship_type_matching_is_case_insensitive() {
        assert!(regime_for_ship_type("Battleship").is_some());
        assert!(regime_for_ship_type("destroyer").is_some());
        assert!(regime_for_ship_type("submarine").is_none());
    }
}
