//! Savitsky (1964) planing-hull power prediction.
//!
//! Implements the classic planing equations:
//! - lift coefficient balance `C_Lβ = τ^1.1 (0.0120 λ^0.5 + 0.0055 λ^2.5 / Cv²)`
//! - center of pressure `C_p = 0.75 - 1/(5.21 Cv²/λ² + 2.39)`
//! - planing equilibrium `LCG/B = C_p · λ`
//! - drag `D = Δ tan τ + R_f / cos τ` with ITTC-1957 friction
//!
//! Validation is against the public planing/semi-planing craft cases in
//! `data/examples/planing_ships.csv` and `more_planing.csv` (PT boats, MTBs,
//! S-boats).  These are approximate historical references, not basin data; see
//! `docs/VALIDATION.md`.

use crate::accuracy::ExternalCase;

const G: f64 = 32.174; // ft/s^2
const RHO: f64 = 1.9905; // slugs/ft^3 (seawater)
const NU: f64 = 1.2817e-5; // ft^2/s
const KNOT_TO_FPS: f64 = 1.68781;

/// Inputs for the Savitsky solver (Imperial units).
#[derive(Clone, Copy, Debug)]
pub struct PlaningInput {
    pub length_ft: f64,
    pub beam_ft: f64,
    pub weight_lb: f64,
    pub speed_kt: f64,
    pub deadrise_deg: f64,
    /// Longitudinal centre of gravity, measured from the transom (ft).
    pub lcg_ft: f64,
    /// Overall propulsive coefficient.
    pub propulsive_coefficient: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct PlaningResult {
    pub trim_deg: f64,
    pub wetted_length_ratio: f64,
    pub wetted_length_ft: f64,
    pub drag_lb: f64,
    pub shaft_hp: f64,
}

/// Solve the Savitsky planing equilibrium.  Returns `None` for invalid inputs.
pub fn savitsky_planing(input: &PlaningInput) -> Option<PlaningResult> {
    if input.length_ft <= 0.0
        || input.beam_ft <= 0.0
        || input.weight_lb <= 0.0
        || input.speed_kt <= 0.0
        || input.deadrise_deg < 0.0
        || input.lcg_ft <= 0.0
    {
        return None;
    }
    let v = input.speed_kt * KNOT_TO_FPS;
    let b = input.beam_ft;
    let cv = v / (G * b).sqrt();
    if cv <= 0.0 {
        return None;
    }
    let cl_beta = input.weight_lb / (0.5 * RHO * v * v * b * b);
    let lcg_over_b = input.lcg_ft / b;

    // Equilibrium trim: find tau such that C_p(tau) * lambda(tau) == LCG/B.
    // C_p * lambda decreases with trim, so g(tau) is decreasing.
    let (lo, hi) = (0.25_f64, 15.0_f64);
    let g = |tau_deg: f64| -> Option<f64> {
        let lambda = solve_lambda(tau_deg, cv, cl_beta)?;
        let cp = center_of_pressure(cv, lambda);
        Some(cp * lambda - lcg_over_b)
    };
    let mut a = lo;
    let mut bnd = hi;
    let ga = g(a)?;
    let gb = g(bnd)?;
    if ga * gb > 0.0 {
        // No sign change: pick the end with the smaller residual.
        let tau = if ga.abs() <= gb.abs() { a } else { bnd };
        return finish(input, v, b, tau);
    }
    for _ in 0..80 {
        let mid = 0.5 * (a + bnd);
        let gm = g(mid)?;
        if gm > 0.0 {
            a = mid;
        } else {
            bnd = mid;
        }
    }
    let tau = 0.5 * (a + bnd);
    let _ = ga;
    finish(input, v, b, tau)
}

fn solve_lambda(tau_deg: f64, cv: f64, cl_beta: f64) -> Option<f64> {
    let t = tau_deg.to_radians();
    let _ = t;
    // Savitsky's lift equation uses trim in degrees for the 1.1 exponent.
    let f = |lambda: f64| -> f64 {
        tau_deg.powf(1.1) * (0.0120 * lambda.sqrt() + 0.0055 * lambda.powf(2.5) / (cv * cv))
            - cl_beta
    };
    let mut lo = 0.02_f64;
    let mut hi = 50.0_f64;
    if f(lo) > 0.0 || f(hi) < 0.0 {
        return None;
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

fn center_of_pressure(cv: f64, lambda: f64) -> f64 {
    let denom = 5.21 * cv * cv / (lambda * lambda) + 2.39;
    0.75 - 1.0 / denom
}

fn finish(input: &PlaningInput, v: f64, b: f64, tau_deg: f64) -> Option<PlaningResult> {
    let cl_beta = input.weight_lb / (0.5 * RHO * v * v * b * b);
    let lambda = solve_lambda(tau_deg, v / (G * b).sqrt(), cl_beta)?;
    let wetted_length = lambda * b;
    let lwl = wetted_length.max(1e-6);
    let re = v * lwl / NU;
    let cf = if re > 1e5 {
        0.075 / (re.log10() - 2.0).powi(2)
    } else {
        0.01
    };
    let beta = input.deadrise_deg.to_radians();
    let wetted_area = lambda * b * b / beta.cos();
    let friction = 0.5 * RHO * v * v * wetted_area * cf;
    let tau = tau_deg.to_radians();
    let drag = input.weight_lb * tau.tan() + friction / tau.cos();
    let eta = input.propulsive_coefficient.clamp(0.05, 1.0);
    let shaft_hp = drag * v / (550.0 * eta);
    Some(PlaningResult {
        trim_deg: tau_deg,
        wetted_length_ratio: lambda,
        wetted_length_ft: wetted_length,
        drag_lb: drag,
        shaft_hp,
    })
}

/// Default deadrise/LCG assumptions for a public planing case.
pub fn assumed_inputs(case: &ExternalCase) -> PlaningInput {
    let deadrise = if case.ship_type.contains("motor")
        || case.ship_type.contains("mtb")
        || case.ship_type.contains("torpedo")
    {
        12.0
    } else {
        14.0
    };
    PlaningInput {
        length_ft: case.length_ft,
        beam_ft: case.beam_ft,
        weight_lb: case.displacement_tons * 2240.0,
        speed_kt: case.speed_kt,
        deadrise_deg: deadrise,
        lcg_ft: 0.42 * case.length_ft,
        propulsive_coefficient: 0.55,
    }
}

/// Per-case validation row.
#[derive(Clone, Debug)]
pub struct PlaningValidationRow {
    pub name: String,
    pub reference_hp: f64,
    pub savitsky_hp: f64,
    pub empirical_hp: f64,
}

#[derive(Clone, Debug)]
pub struct PlaningValidation {
    pub rows: Vec<PlaningValidationRow>,
    pub savitsky_raw_mape: f64,
    pub savitsky_factor: f64,
    pub savitsky_calibrated_mape: f64,
    pub empirical_raw_mape: f64,
    pub empirical_factor: f64,
    pub empirical_calibrated_mape: f64,
    pub cases: usize,
}

/// Validate Savitsky and the existing empirical planing model against a set of
/// public planing cases.  Calibration factors are `mean(reference / raw)`.
pub fn validate_planing_cases(cases: &[ExternalCase]) -> PlaningValidation {
    let eligible: Vec<&ExternalCase> = cases
        .iter()
        .filter(|c| crate::planing::is_planing_eligible(c.speed_kt, &c.ship_type))
        .collect();
    let mut rows = Vec::new();
    for case in &eligible {
        let input = assumed_inputs(case);
        let sav = savitsky_planing(&input).map(|r| r.shaft_hp).unwrap_or(0.0);
        let emp = crate::planing::planing_hp_estimate(
            case.length_ft,
            case.beam_ft,
            case.displacement_tons * 2240.0,
            case.speed_kt,
        );
        rows.push(PlaningValidationRow {
            name: case.name.clone(),
            reference_hp: case.reference_hp,
            savitsky_hp: sav,
            empirical_hp: emp,
        });
    }
    let factor = |pick: fn(&PlaningValidationRow) -> f64| -> f64 {
        let mut s = 0.0;
        let mut n = 0.0;
        for r in &rows {
            let p = pick(r);
            if p > 0.0 && r.reference_hp > 0.0 {
                s += r.reference_hp / p;
                n += 1.0;
            }
        }
        if n > 0.0 {
            s / n
        } else {
            1.0
        }
    };
    let mape = |pick: fn(&PlaningValidationRow) -> f64, f: f64| -> f64 {
        if rows.is_empty() {
            return f64::NAN;
        }
        rows.iter()
            .map(|r| ((pick(r) * f - r.reference_hp) / r.reference_hp).abs())
            .sum::<f64>()
            / rows.len() as f64
    };
    let sf = factor(|r| r.savitsky_hp);
    let ef = factor(|r| r.empirical_hp);
    PlaningValidation {
        savitsky_raw_mape: mape(|r| r.savitsky_hp, 1.0),
        savitsky_factor: sf,
        savitsky_calibrated_mape: mape(|r| r.savitsky_hp, sf),
        empirical_raw_mape: mape(|r| r.empirical_hp, 1.0),
        empirical_factor: ef,
        empirical_calibrated_mape: mape(|r| r.empirical_hp, ef),
        cases: rows.len(),
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn savitsky_pt_boat_is_finite_and_physical() {
        let input = PlaningInput {
            length_ft: 80.0,
            beam_ft: 20.0,
            weight_lb: 50.0 * 2240.0,
            speed_kt: 40.0,
            deadrise_deg: 12.0,
            lcg_ft: 0.42 * 80.0,
            propulsive_coefficient: 0.55,
        };
        let r = savitsky_planing(&input).expect("solvable");
        assert!(r.trim_deg > 0.0 && r.trim_deg < 15.0);
        assert!(r.wetted_length_ratio > 0.5 && r.wetted_length_ratio < 10.0);
        assert!(r.shaft_hp.is_finite() && r.shaft_hp > 0.0);
    }

    #[test]
    fn savitsky_power_increases_with_speed_and_weight() {
        let base = PlaningInput {
            length_ft: 80.0,
            beam_ft: 20.0,
            weight_lb: 50.0 * 2240.0,
            speed_kt: 40.0,
            deadrise_deg: 12.0,
            lcg_ft: 33.6,
            propulsive_coefficient: 0.55,
        };
        let p0 = savitsky_planing(&base).unwrap().shaft_hp;
        let faster = PlaningInput {
            speed_kt: 45.0,
            ..base
        };
        let heavier = PlaningInput {
            weight_lb: 60.0 * 2240.0,
            ..base
        };
        assert!(savitsky_planing(&faster).unwrap().shaft_hp > p0);
        assert!(savitsky_planing(&heavier).unwrap().shaft_hp > p0);
    }

    #[test]
    fn savitsky_rejects_bad_inputs() {
        let bad = PlaningInput {
            length_ft: 0.0,
            beam_ft: 20.0,
            weight_lb: 1000.0,
            speed_kt: 40.0,
            deadrise_deg: 12.0,
            lcg_ft: 30.0,
            propulsive_coefficient: 0.55,
        };
        assert!(savitsky_planing(&bad).is_none());
    }

    #[test]
    fn validation_runs_on_public_planing_cases() {
        let mut cases = Vec::new();
        for path in [
            "data/examples/planing_ships.csv",
            "data/examples/more_planing.csv",
        ] {
            let text = std::fs::read_to_string(path).unwrap();
            cases.extend(crate::accuracy::parse_external_cases(&text).unwrap());
        }
        let v = validate_planing_cases(&cases);
        assert_eq!(v.cases, cases.len());
        assert!(v.savitsky_raw_mape.is_finite());
        assert!(v.savitsky_factor > 0.0 && v.savitsky_factor.is_finite());
        assert!(v.savitsky_calibrated_mape.is_finite());
    }
}
