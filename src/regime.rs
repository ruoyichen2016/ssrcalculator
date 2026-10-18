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

/// Five scale-free, country-agnostic dimensionless hydrodynamic invariants ($\Pi$-groups).
///
/// Computed strictly from fundamental physics and hull geometry (`Fn`, `L/∇^{1/3}`,
/// `L/B`, `B/T`, `C_b`), with zero access to country, flag state, or ship name.
#[derive(Clone, Copy, Debug)]
pub struct HydroDimensionlessFeatures {
    /// Froude number $Fn = V / \sqrt{g L}$.
    pub froude_number: f64,
    /// Volumetric slenderness ratio $L / \nabla^{1/3}$.
    pub slenderness_ratio: f64,
    /// Length-to-beam ratio $L / B$.
    pub length_beam_ratio: f64,
    /// Beam-to-draught ratio $B / T$.
    pub beam_draught_ratio: f64,
    /// Block coefficient $C_b = \nabla / (L B T)$.
    pub block_coeff: f64,
}

/// Pure Extracted Hydrodynamic Ideal Types (水动力无量纲理想类型) for style-adaptive matching.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HydroIdealType {
    /// Hydro Type I · Displacement Block Form ($Fn \approx 0.21, C_b \approx 0.60, L/\nabla^{1/3} \approx 5.6$)
    DisplacementBlock,
    /// Hydro Type II · Trans-Froude Fast Capital ($Fn \approx 0.30, C_b \approx 0.57, L/\nabla^{1/3} \approx 6.9$)
    TransFroudeFastCapital,
    /// Hydro Type III · Slender Wave-Cutting Cruiser ($Fn \approx 0.36, C_b \approx 0.51, L/\nabla^{1/3} \approx 8.0$)
    SlenderWaveCutting,
    /// Hydro Type IV · Super-Critical High-Fn Planing/Semi-Planing ($Fn \approx 0.52, C_b \approx 0.48, L/\nabla^{1/3} \approx 8.8$)
    SuperCriticalHighFn,
}

impl HydroDimensionlessFeatures {
    /// Extract dimensionless hydrodynamic invariants from raw geometric/kinematic parameters.
    pub fn from_geometry(
        length_ft: f64,
        beam_ft: f64,
        draught_ft: f64,
        displacement_tons: f64,
        block_coeff: f64,
        speed_kt: f64,
    ) -> Self {
        let l = length_ft.max(1.0);
        let b = beam_ft.max(1.0);
        let t = draught_ft.max(0.5);
        let disp = displacement_tons.max(1.0);
        let cb = block_coeff.clamp(0.30, 0.90);
        let v_fps = speed_kt.max(0.0) * 1.6878098571;
        let g = 32.17405;
        let froude_number = v_fps / (g * l).sqrt();
        let vol_ft3 = disp * 35.0;
        let slenderness_ratio = l / vol_ft3.cbrt();
        Self {
            froude_number,
            slenderness_ratio,
            length_beam_ratio: l / b,
            beam_draught_ratio: b / t,
            block_coeff: cb,
        }
    }

    /// Extract dimensionless hydrodynamic invariants from a historical benchmark case.
    pub fn from_case(case: &HistoricalCase) -> Self {
        Self::from_geometry(
            case.length_ft,
            case.beam_ft,
            case.draught_ft,
            case.displacement_tons,
            case.block_coefficient,
            case.speed_kt,
        )
    }

    /// Extract dimensionless hydrodynamic invariants from a full `ShipModel`.
    pub fn from_ship(ship: &ShipModel) -> Self {
        let disp = if ship.hull.displacement > 1.0 {
            ship.hull.displacement
        } else {
            (ship.hull.length * ship.hull.beam_bulges * ship.hull.draught * ship.hull.block / 35.0)
                .max(1.0)
        };
        Self::from_geometry(
            ship.hull.length,
            ship.hull.beam_bulges,
            ship.hull.draught,
            disp,
            ship.hull.block,
            ship.engine.speed_max,
        )
    }

    /// Compute normalized soft affinities $(a_1, a_2, a_3, a_4)$ over the 4 `HydroIdealType` centroids.
    pub fn ideal_type_affinities(&self) -> [f64; 4] {
        // Centroids (Fn, L/∇^{1/3}, L/B, B/T, C_b)
        const CENTROIDS: [(f64, f64, f64, f64, f64); 4] = [
            (0.21, 5.6, 6.5, 3.1, 0.60), // Type I: Displacement Block
            (0.30, 6.9, 7.6, 3.3, 0.57), // Type II: Trans-Froude Fast Capital
            (0.36, 8.0, 9.2, 3.2, 0.51), // Type III: Slender Wave-Cutting
            (0.52, 8.8, 9.8, 3.1, 0.48), // Type IV: Super-Critical High-Fn
        ];
        const SCALES: (f64, f64, f64, f64, f64) = (0.07, 1.1, 1.5, 0.5, 0.05);

        let mut weights = [0.0_f64; 4];
        let mut sum = 0.0;
        for (idx, c) in CENTROIDS.iter().enumerate() {
            let d2 = ((self.froude_number - c.0) / SCALES.0).powi(2)
                + ((self.slenderness_ratio - c.1) / SCALES.1).powi(2)
                + ((self.length_beam_ratio - c.2) / SCALES.2).powi(2)
                + ((self.beam_draught_ratio - c.3) / SCALES.3).powi(2)
                + ((self.block_coeff - c.4) / SCALES.4).powi(2);
            let w = (-0.5 * d2).exp();
            weights[idx] = w;
            sum += w;
        }
        if sum > 1e-12 {
            for w in &mut weights {
                *w /= sum;
            }
        } else {
            weights = [0.25, 0.25, 0.25, 0.25];
        }
        weights
    }

    /// Dominant hydrodynamic ideal type.
    pub fn dominant_ideal_type(&self) -> HydroIdealType {
        let aff = self.ideal_type_affinities();
        let mut best_idx = 0;
        for i in 1..4 {
            if aff[i] > aff[best_idx] {
                best_idx = i;
            }
        }
        match best_idx {
            0 => HydroIdealType::DisplacementBlock,
            1 => HydroIdealType::TransFroudeFastCapital,
            2 => HydroIdealType::SlenderWaveCutting,
            _ => HydroIdealType::SuperCriticalHighFn,
        }
    }

    /// Dimensionless hydrodynamic similarity kernel $K(\Pi_a, \Pi_b) \in (0, 1]$ combining
    /// direct $\Pi$-group distance and soft Hydro Ideal-Type affinity cosine similarity.
    pub fn similarity(&self, other: &Self) -> f64 {
        let dfn = (self.froude_number - other.froude_number) / 0.055;
        let dsl = (self.slenderness_ratio - other.slenderness_ratio) / 0.80;
        let dlb = (self.length_beam_ratio - other.length_beam_ratio) / 1.15;
        let dbt = (self.beam_draught_ratio - other.beam_draught_ratio) / 0.50;
        let dcb = (self.block_coeff - other.block_coeff) / 0.045;
        let dist2 =
            dfn * dfn + 0.90 * dsl * dsl + 0.70 * dlb * dlb + 0.45 * dbt * dbt + 0.80 * dcb * dcb;
        let kernel = (-0.5 * dist2).exp();

        let a1 = self.ideal_type_affinities();
        let a2 = other.ideal_type_affinities();
        let dot: f64 = a1.iter().zip(a2.iter()).map(|(x, y)| x * y).sum();
        let n1: f64 = a1.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
        let n2: f64 = a2.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
        let ideal_sim = (dot / (n1 * n2)).clamp(0.0, 1.0);

        0.08 + 0.92 * (0.65 * kernel + 0.35 * ideal_sim.powi(3))
    }
}

/// Factor fraction trimmed from each tail before averaging.
///
/// A 15% trimmed mean rejects the influence of individual outlier cases.  A
/// nested-LOOCV sweep over 5-20% trims (and the geometric mean) showed a stable
/// ~0.13pp improvement over the plain mean; see docs/VALIDATION.md.
const FACTOR_TRIM: f64 = 0.15;

fn weighted_trimmed_mean(mut pairs: Vec<(f64, f64)>) -> f64 {
    if pairs.is_empty() {
        return 1.0;
    }
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let total_weight: f64 = pairs.iter().map(|(_, w)| w).sum();
    let trim = total_weight * FACTOR_TRIM;
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

/// Robust `reference / simulated` factor: 15% trimmed mean with curated cases
/// weighted 2x relative to the approximate bulk.  Confidence weighting improved
/// both the full-set and curated-core nested LOOCV (docs/VALIDATION.md).
fn factor_for(cases: &[&HistoricalCase], power: impl Fn(&HistoricalCase) -> f64) -> f64 {
    let pairs: Vec<(f64, f64)> = cases
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
    weighted_trimmed_mean(pairs)
}

/// Dimensionless style-adaptive calibration factor for a target $\Pi_{\text{hydro}}$ state
/// evaluated strictly over a training pool `cases`.
///
/// Blends the global per-class factor with the dimensionless kernel-weighted trimmed mean
/// so that hulls in distinct Froude / slenderness regimes (e.g. pre-dreadnought vs fast
/// battleship, or protected cruiser vs fast light cruiser) receive physically matched factors.
fn adaptive_factor_for_features(
    target: &HydroDimensionlessFeatures,
    cases: &[&HistoricalCase],
    power: impl Fn(&HistoricalCase) -> f64,
) -> f64 {
    let base_factor = factor_for(cases, &power);
    let pairs: Vec<(f64, f64)> = cases
        .iter()
        .filter_map(|case| {
            let p = power(case);
            if case.reference_hp > 0.0 && p > 0.0 {
                let provenance_w = if case.provenance.is_curated() {
                    2.0
                } else {
                    1.0
                };
                let feat = HydroDimensionlessFeatures::from_case(case);
                let sim_w = target.similarity(&feat);
                Some((case.reference_hp / p, provenance_w * sim_w))
            } else {
                None
            }
        })
        .collect();
    if pairs.is_empty() {
        return base_factor;
    }
    let local_factor = weighted_trimmed_mean(pairs);
    // Shrink local dimensionless factor gently toward class baseline for stability.
    0.85 * local_factor + 0.15 * base_factor
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

/// Apply the regime model + dimensionless style-adaptive factor to a design.
pub fn apply_regime_calibration(ship: &mut ShipModel) -> Option<RegimeCalibration> {
    let mut entry = regime_for_ship_type(&ship.ship_type)?;
    let cases = historical_cases();
    let subset: Vec<&HistoricalCase> = cases
        .iter()
        .filter(|c| c.ship_type == entry.ship_type)
        .collect();
    if subset.len() >= 2 && ship.hull.length > 10.0 && ship.engine.speed_max > 1.0 {
        let feat = HydroDimensionlessFeatures::from_ship(ship);
        entry.factor = adaptive_factor_for_features(&feat, &subset, raw_holtrop);
    }
    ship.engine.resistance_model = entry.model;
    ship.engine.power_factor = entry.factor;
    ship.engine.original_power_factor = 1.0;
    ship.engine.blend_weight = 1.0;
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
    /// Original SpringSharp model with no calibration factor.
    pub original_hp: f64,
    /// Original SpringSharp + per-type factor (nested) prediction.
    pub original_calibrated_hp: f64,
    /// Holtrop-Mennen with no calibration factor.
    pub holtrop_hp: f64,
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

/// How strongly the Holtrop and original models are blended for a ship type.
#[derive(Clone, Copy, Debug)]
pub struct BlendCalibration {
    pub ship_type: &'static str,
    pub count: usize,
    /// Weight of the Holtrop result; the original gets `1 - weight`.
    pub weight: f64,
    pub holtrop_factor: f64,
    pub original_factor: f64,
    /// In-sample (weight-selected) MAPE, diagnostic only.
    pub train_mape: f64,
}

/// How much the per-type blend weight is shrunk toward the global weight.
const BLEND_SHRINK: f64 = 8.0;

/// Best Holtrop weight on a 0.05 grid for a set of (regime, original, ref).
fn best_blend_weight(rows: &[(f64, f64, f64)]) -> f64 {
    if rows.is_empty() {
        return 1.0;
    }
    let mut best = (1.0_f64, f64::INFINITY);
    for k in 0..=20 {
        let w = k as f64 / 20.0;
        let m = rows
            .iter()
            .map(|(a, b, r)| ((w * a + (1.0 - w) * b - r) / r).abs())
            .sum::<f64>()
            / rows.len() as f64;
        if m < best.1 {
            best = (w, m);
        }
    }
    best.0
}

fn blend_rows() -> Vec<(f64, f64, f64, &'static str, f64)> {
    run_loocv_regime()
        .residuals
        .iter()
        .map(|x| {
            (
                x.predicted_hp,
                x.original_calibrated_hp,
                x.reference_hp,
                x.ship_type,
                x.confidence,
            )
        })
        .collect()
}

/// Fit the production blend: per-type Holtrop weight (shrunk toward the global
/// weight) and both per-type calibration factors.
pub fn fit_blend_calibration() -> Vec<BlendCalibration> {
    let rows = blend_rows();
    let all: Vec<(f64, f64, f64)> = rows.iter().map(|r| (r.0, r.1, r.2)).collect();
    let global_w = best_blend_weight(&all);
    let cases = historical_cases();
    let mut types: Vec<&'static str> = rows.iter().map(|r| r.3).collect();
    types.sort_unstable();
    types.dedup();
    types
        .into_iter()
        .map(|ty| {
            let subset: Vec<(f64, f64, f64)> = rows
                .iter()
                .filter(|r| r.3 == ty)
                .map(|r| (r.0, r.1, r.2))
                .collect();
            let n = subset.len() as f64;
            let w_type = if subset.len() >= 4 {
                best_blend_weight(&subset)
            } else {
                global_w
            };
            let weight = (n * w_type + BLEND_SHRINK * global_w) / (n + BLEND_SHRINK);
            let type_cases: Vec<&HistoricalCase> =
                cases.iter().filter(|c| c.ship_type == ty).collect();
            let holtrop_factor = factor_for(&type_cases, raw_holtrop);
            let original_factor = factor_for(&type_cases, raw_original);
            let train_mape = if subset.is_empty() {
                f64::NAN
            } else {
                subset
                    .iter()
                    .map(|(a, b, r)| ((weight * a + (1.0 - weight) * b - r) / r).abs())
                    .sum::<f64>()
                    / n
            };
            BlendCalibration {
                ship_type: ty,
                count: subset.len(),
                weight,
                holtrop_factor,
                original_factor,
                train_mape,
            }
        })
        .collect()
}

/// Blend row for a ship type (case-insensitive, tolerant of qualifiers).
pub fn blend_for_ship_type(ship_type: &str) -> Option<BlendCalibration> {
    let needle = ship_type.to_ascii_lowercase();
    fit_blend_calibration()
        .into_iter()
        .find(|r| needle.contains(r.ship_type) || r.ship_type.contains(needle.trim()))
}

/// Apply the blended regime model (and both dimensionless style-adaptive factors) to a design.
pub fn apply_blend_calibration(ship: &mut ShipModel) -> Option<BlendCalibration> {
    let mut entry = blend_for_ship_type(&ship.ship_type)?;
    let cases = historical_cases();
    let subset: Vec<&HistoricalCase> = cases
        .iter()
        .filter(|c| c.ship_type == entry.ship_type)
        .collect();
    if subset.len() >= 2 && ship.hull.length > 10.0 && ship.engine.speed_max > 1.0 {
        let feat = HydroDimensionlessFeatures::from_ship(ship);
        entry.holtrop_factor = adaptive_factor_for_features(&feat, &subset, raw_holtrop);
        entry.original_factor = adaptive_factor_for_features(&feat, &subset, raw_original);
    }
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = entry.holtrop_factor;
    ship.engine.original_power_factor = entry.original_factor;
    ship.engine.blend_weight = entry.weight;
    ship.calculate();
    Some(entry)
}

/// Nested-LOOCV result for the blended model.
#[derive(Clone, Debug)]
pub struct BlendLoocv {
    pub overall_mape: f64,
    pub macro_mape: f64,
    pub curated_mape: f64,
    pub per_type: Vec<(String, usize, f64)>,
}

/// Nested estimation of the blend: the weights and factors come from the
/// out-of-fold predictions, and the blend weight is re-chosen without the test
/// case (shrunk per-type toward the global weight).
pub fn run_loocv_blend() -> BlendLoocv {
    let rows = blend_rows();
    let mut errs: Vec<(String, f64, f64)> = Vec::new();
    for i in 0..rows.len() {
        let (a, b, r, ty, conf) = rows[i];
        let train: Vec<&(f64, f64, f64, &'static str, f64)> = rows
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, x)| x)
            .collect();
        let global_w =
            best_blend_weight(&train.iter().map(|x| (x.0, x.1, x.2)).collect::<Vec<_>>());
        let same: Vec<&(f64, f64, f64, &'static str, f64)> =
            train.iter().copied().filter(|x| x.3 == ty).collect();
        let w_type = if same.len() >= 4 {
            best_blend_weight(&same.iter().map(|x| (x.0, x.1, x.2)).collect::<Vec<_>>())
        } else {
            global_w
        };
        let n = same.len() as f64;
        let w = (n * w_type + BLEND_SHRINK * global_w) / (n + BLEND_SHRINK);
        errs.push((
            ty.to_string(),
            ((w * a + (1.0 - w) * b - r) / r).abs(),
            conf,
        ));
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let all: Vec<f64> = errs.iter().map(|e| e.1).collect();
    let curated: Vec<f64> = errs.iter().filter(|e| e.2 >= 0.65).map(|e| e.1).collect();
    let mut types: Vec<String> = errs.iter().map(|e| e.0.clone()).collect();
    types.sort();
    types.dedup();
    let per_type = types
        .into_iter()
        .map(|ty| {
            let es: Vec<f64> = errs.iter().filter(|e| e.0 == ty).map(|e| e.1).collect();
            (ty, es.len(), mean(&es))
        })
        .collect::<Vec<_>>();
    BlendLoocv {
        overall_mape: mean(&all),
        macro_mape: per_type.iter().map(|(_, _, m)| *m).sum::<f64>() / per_type.len().max(1) as f64,
        curated_mape: mean(&curated),
        per_type,
    }
}

/// Paired comparison of the modern regime model against baselines.
#[derive(Clone, Debug)]
pub struct BaselineComparison {
    pub n: usize,
    pub regime_mape: f64,
    pub original_mape: f64,
    pub original_calibrated_mape: f64,
    pub holtrop_mape: f64,
    pub regime_median: f64,
    pub original_median: f64,
    /// Cases where the regime error is strictly smaller than the original.
    pub regime_wins: usize,
    pub original_wins: usize,
    pub ties: usize,
    /// Two-sided sign-test p-value (regime vs original raw).
    pub sign_test_p: f64,
    /// (ship type, n, regime MAPE, original MAPE, regime wins)
    pub per_type: Vec<(String, usize, f64, f64, usize)>,
}

fn paired_median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

/// Two-sided sign-test p-value for `k` successes in `n` trials (p = 0.5).
pub fn sign_test_p(k: usize, n: usize) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let mut probs = vec![0.0_f64; n + 1];
    probs[0] = 0.5_f64.powi(n as i32);
    for i in 0..n {
        probs[i + 1] = probs[i] * (n - i) as f64 / (i + 1) as f64;
    }
    let low: f64 = probs[..=k.min(n)].iter().sum();
    let high: f64 = probs[k.min(n)..].iter().sum();
    (2.0 * low.min(high)).min(1.0)
}

/// Compare the nested regime predictions with the original SpringSharp model
/// (raw and per-type calibrated) and uncalibrated Holtrop, case by case.
pub fn compare_regime_vs_original() -> BaselineComparison {
    let report = run_loocv_regime();
    let mut regime_abs = Vec::new();
    let mut original_abs = Vec::new();
    let mut original_cal_abs = Vec::new();
    let mut holtrop_abs = Vec::new();
    let mut wins = 0_usize;
    let mut losses = 0_usize;
    let mut ties = 0_usize;
    let mut per_type: Vec<(String, Vec<f64>, Vec<f64>, usize)> = Vec::new();
    for row in &report.residuals {
        let ref_hp = row.reference_hp.max(1e-9);
        let re = (row.predicted_hp - row.reference_hp).abs() / ref_hp;
        let oe = (row.original_hp - row.reference_hp).abs() / ref_hp;
        let oce = (row.original_calibrated_hp - row.reference_hp).abs() / ref_hp;
        let he = (row.holtrop_hp - row.reference_hp).abs() / ref_hp;
        if re + 1e-12 < oe {
            wins += 1;
        } else if oe + 1e-12 < re {
            losses += 1;
        } else {
            ties += 1;
        }
        regime_abs.push(re);
        original_abs.push(oe);
        original_cal_abs.push(oce);
        holtrop_abs.push(he);
        let ty = row.ship_type.to_string();
        if let Some(e) = per_type.iter_mut().find(|e| e.0 == ty) {
            e.1.push(re);
            e.2.push(oe);
            if re + 1e-12 < oe {
                e.3 += 1;
            }
        } else {
            per_type.push((ty, vec![re], vec![oe], usize::from(re + 1e-12 < oe)));
        }
    }
    let mean = |v: &[f64]| {
        if v.is_empty() {
            f64::NAN
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };
    let per_type = per_type
        .into_iter()
        .map(|(ty, re, oe, w)| (ty, re.len(), mean(&re), mean(&oe), w))
        .collect();
    BaselineComparison {
        n: report.residuals.len(),
        regime_mape: mean(&regime_abs),
        original_mape: mean(&original_abs),
        original_calibrated_mape: mean(&original_cal_abs),
        holtrop_mape: mean(&holtrop_abs),
        regime_median: paired_median(regime_abs),
        original_median: paired_median(original_abs),
        regime_wins: wins,
        original_wins: losses,
        ties,
        sign_test_p: sign_test_p(wins, wins + losses),
        per_type,
    }
}

/// Nested leave-one-out evaluation.
///
/// For every held-out case the factor is fitted on the remaining cases of the
/// same ship type using dimensionless hydrodynamic feature similarity (falling
/// back to all training cases when fewer than two same-type cases remain), so
/// the reported error does not use the test case or any national/ship-name label.
pub fn run_loocv_regime() -> RegimeLoocv {
    let cases = historical_cases();
    let raw_h: Vec<f64> = cases.iter().map(raw_holtrop).collect();
    let raw_o: Vec<f64> = cases.iter().map(raw_original).collect();
    let feats: Vec<HydroDimensionlessFeatures> = cases
        .iter()
        .map(HydroDimensionlessFeatures::from_case)
        .collect();

    let mut errs = Vec::new();
    let mut holt_uncal = Vec::new();
    let mut orig_uncal = Vec::new();
    let mut orig_errs = Vec::new();
    let mut per_type: Vec<(String, usize, f64)> = Vec::new();
    let mut residuals: Vec<RegimeResidual> = Vec::new();

    for i in 0..cases.len() {
        let test = &cases[i];
        let target_feat = &feats[i];
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

        let fh = adaptive_factor_for_features(target_feat, pool, raw_holtrop);
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
            original_hp: raw_o[i],
            original_calibrated_hp: pred_o,
            holtrop_hp: raw_h[i],
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
    fn sign_test_is_sane() {
        assert!((sign_test_p(0, 0) - 1.0).abs() < 1e-12);
        assert!(sign_test_p(50, 100) > 0.9);
        assert!(sign_test_p(84, 123) < 1e-3);
    }

    #[test]
    fn blend_beats_regime_alone() {
        let b = run_loocv_blend();
        let r = run_loocv_regime();
        assert!(
            b.overall_mape < r.overall_mape,
            "blend {:.3} should beat regime {:.3}",
            b.overall_mape,
            r.overall_mape
        );
        assert!(b.curated_mape < r.curated_mape);
        let cal = fit_blend_calibration();
        assert!(!cal.is_empty());
        for c in &cal {
            assert!((0.0..=1.0).contains(&c.weight));
            assert!(c.holtrop_factor > 0.0 && c.original_factor > 0.0);
        }
    }

    #[test]
    fn regime_beats_original_overall() {
        let c = compare_regime_vs_original();
        assert_eq!(c.n, historical_cases().len());
        assert!(
            c.regime_mape < c.original_mape,
            "regime {:.3} should beat original {:.3}",
            c.regime_mape,
            c.original_mape
        );
        assert!(c.regime_mape < c.original_calibrated_mape);
        assert!(c.regime_median < c.original_median);
        assert!(c.regime_wins > c.original_wins);
        assert!(c.sign_test_p < 0.01, "p={}", c.sign_test_p);
        // Honest caveat: the original remains competitive for cruisers.
        let cruiser = c
            .per_type
            .iter()
            .find(|e| e.0 == "cruiser")
            .expect("cruiser group");
        assert!(cruiser.2 < 0.12 && cruiser.3 < 0.12);
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

    #[test]
    fn hydro_ideal_types_and_country_invariance() {
        let mut s1 = crate::presets::get_preset("iowa").expect("iowa");
        let mut s2 = s1.clone();
        s1.country = "Republic of Alpha".into();
        s1.name = "Alpha-1".into();
        s2.country = "Empire of Beta".into();
        s2.name = "Beta-9".into();

        let f1 = HydroDimensionlessFeatures::from_ship(&s1);
        let f2 = HydroDimensionlessFeatures::from_ship(&s2);
        assert_eq!(f1.dominant_ideal_type(), f2.dominant_ideal_type());
        assert_eq!(
            f1.dominant_ideal_type(),
            HydroIdealType::TransFroudeFastCapital
        );

        let r1 = apply_regime_calibration(&mut s1).expect("regime s1");
        let r2 = apply_regime_calibration(&mut s2).expect("regime s2");
        assert!((r1.factor - r2.factor).abs() < 1e-12);
        assert!((s1.engine.hp_max - s2.engine.hp_max).abs() < 1e-9);

        let dd = crate::presets::get_preset("fletcher").expect("fletcher");
        let f_dd = HydroDimensionlessFeatures::from_ship(&dd);
        assert_eq!(
            f_dd.dominant_ideal_type(),
            HydroIdealType::SuperCriticalHighFn
        );
    }
}
