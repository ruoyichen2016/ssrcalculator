//! Reduced-form and structural-form statistical calibration.
//!
//! - **Reduced form**: a power-law regression
//!   `P = exp(b0) * Displacement^b1 * Speed^b2 * Length^b3`
//!   fitted in log space, optionally with ship-type dummy variables.
//!
//! - **Structural form**: the Holtrop-Mennen physical model is decomposed into
//!   viscous, wave, and correlation components; linear least squares then
//!   estimates component multipliers from historical speed/power points,
//!   optionally segmented by Froude number.
//!
//! - **Ensemble**: reduced-form and structural-form predictions are combined
//!   with inverse-error weighting.

use crate::accuracy::{
    case_confidence, froude_number, historical_cases, simulate_case_with_model, ExternalCase,
};
use crate::model::{ResistanceModel, ShipModel};
use crate::resistance::{holtrop_resistance, input_from_hull, HoltropResistance};

#[derive(Clone, Debug)]
pub struct ReducedFormFit {
    pub intercept: f64,
    pub log_displacement: f64,
    pub log_speed: f64,
    pub log_length: f64,
    pub mean_abs_pct_error: f64,
    pub sigma_abs_pct_error: f64,
}

impl ReducedFormFit {
    pub fn predict(&self, displacement: f64, speed_kt: f64, length_ft: f64) -> f64 {
        (self.intercept
            + self.log_displacement * displacement.ln()
            + self.log_speed * speed_kt.ln()
            + self.log_length * length_ft.ln())
        .exp()
    }
}

#[derive(Clone, Debug)]
pub struct ReducedFormByTypeFit {
    pub intercept: f64,
    pub log_displacement: f64,
    pub log_speed: f64,
    pub log_length: f64,
    pub d_carrier: f64,
    pub d_cruiser: f64,
    pub d_destroyer: f64,
    pub mean_abs_pct_error: f64,
    pub sigma_abs_pct_error: f64,
}

impl ReducedFormByTypeFit {
    pub fn predict(
        &self,
        displacement: f64,
        speed_kt: f64,
        length_ft: f64,
        ship_type: &str,
    ) -> f64 {
        let d_carrier = if ship_type == "carrier" { 1.0 } else { 0.0 };
        let d_cruiser = if ship_type == "cruiser" { 1.0 } else { 0.0 };
        let d_destroyer = if ship_type == "destroyer" { 1.0 } else { 0.0 };
        (self.intercept
            + self.log_displacement * displacement.ln()
            + self.log_speed * speed_kt.ln()
            + self.log_length * length_ft.ln()
            + self.d_carrier * d_carrier
            + self.d_cruiser * d_cruiser
            + self.d_destroyer * d_destroyer)
            .exp()
    }
}

#[derive(Clone, Debug)]
pub struct StructuralFormFit {
    pub viscous_factor: f64,
    pub wave_factor: f64,
    pub correlation_factor: f64,
    pub mean_abs_pct_error: f64,
    pub sigma_abs_pct_error: f64,
}

impl StructuralFormFit {
    pub fn predict_kw(&self, fric_kw: f64, wave_kw: f64, corr_kw: f64) -> f64 {
        self.viscous_factor * fric_kw
            + self.wave_factor * wave_kw
            + self.correlation_factor * corr_kw
    }

    pub fn predict_hp(&self, fric_kw: f64, wave_kw: f64, corr_kw: f64) -> f64 {
        self.predict_kw(fric_kw, wave_kw, corr_kw) / 0.7456999
    }
}

#[derive(Clone, Debug)]
pub struct StructuralSegment {
    pub name: &'static str,
    pub min_froude: f64,
    pub max_froude: f64,
    pub viscous_factor: f64,
    pub wave_factor: f64,
    pub correlation_factor: f64,
    pub mean_abs_pct_error: f64,
    pub sigma_abs_pct_error: f64,
}

#[derive(Clone, Debug)]
pub struct EnsembleFit {
    pub reduced_weight: f64,
    pub structural_weight: f64,
    pub mean_abs_pct_error: f64,
    pub sigma_abs_pct_error: f64,
}

impl EnsembleFit {
    pub fn predict_hp(&self, reduced_hp: f64, structural_hp: f64) -> f64 {
        self.reduced_weight * reduced_hp + self.structural_weight * structural_hp
    }
}

/// Solve an overdetermined linear least-squares problem using Householder QR.
///
/// This follows the numerical-linear-algebra advice from Introduction to
/// Algorithms style texts: avoid forming normal equations when the design
/// matrix is ill-conditioned.  QR is more stable for dummy-variable models.
fn solve_qr(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let m = a.len();
    let n = a.first()?.len();
    if m < n || b.len() != m {
        return None;
    }
    let mut r = a.to_vec();
    let mut qtb = b.to_vec();

    for k in 0..n {
        let norm: f64 = r
            .iter()
            .skip(k)
            .map(|row| row[k] * row[k])
            .sum::<f64>()
            .sqrt();
        if norm < 1e-12 {
            return None;
        }
        let alpha = if r[k][k] >= 0.0 { -norm } else { norm };
        let mut v = vec![0.0; m];
        for i in k..m {
            v[i] = r[i][k];
        }
        v[k] -= alpha;
        let vnorm: f64 = v.iter().skip(k).map(|x| x * x).sum::<f64>().sqrt();
        if vnorm < 1e-12 {
            continue;
        }
        for i in k..m {
            v[i] /= vnorm;
        }
        for j in k..n {
            let dot: f64 = (k..m).map(|i| v[i] * r[i][j]).sum();
            for i in k..m {
                r[i][j] -= 2.0 * v[i] * dot;
            }
        }
        let dot: f64 = (k..m).map(|i| v[i] * qtb[i]).sum();
        for i in k..m {
            qtb[i] -= 2.0 * v[i] * dot;
        }
    }

    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = qtb[i];
        for j in i + 1..n {
            sum -= r[i][j] * x[j];
        }
        if r[i][i].abs() < 1e-12 {
            return None;
        }
        x[i] = sum / r[i][i];
    }
    Some(x)
}

fn mean_sd(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (f64::INFINITY, f64::INFINITY);
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    (mean, var.sqrt())
}

/// Fit a global reduced-form log-power regression.
pub fn fit_reduced_form() -> ReducedFormFit {
    let cases = historical_cases();
    let mut a: Vec<Vec<f64>> = Vec::new();
    let mut b: Vec<f64> = Vec::new();
    for case in cases.iter() {
        if case.reference_hp <= 0.0 || case.displacement_tons <= 0.0 || case.speed_kt <= 0.0 {
            continue;
        }
        a.push(vec![
            1.0,
            case.displacement_tons.ln(),
            case.speed_kt.ln(),
            case.length_ft.ln(),
        ]);
        b.push(case.reference_hp.ln());
    }
    let n = b.len();
    if n < 4 {
        return ReducedFormFit {
            intercept: 0.0,
            log_displacement: 0.0,
            log_speed: 0.0,
            log_length: 0.0,
            mean_abs_pct_error: f64::INFINITY,
            sigma_abs_pct_error: f64::INFINITY,
        };
    }
    let coef = solve_qr(&a, &b).unwrap_or_default();
    let fit = ReducedFormFit {
        intercept: coef[0],
        log_displacement: coef[1],
        log_speed: coef[2],
        log_length: coef[3],
        mean_abs_pct_error: 0.0,
        sigma_abs_pct_error: 0.0,
    };
    let mut errs = Vec::new();
    for case in cases.iter() {
        if case.reference_hp <= 0.0 {
            continue;
        }
        let pred = fit.predict(case.displacement_tons, case.speed_kt, case.length_ft);
        if pred > 0.0 {
            errs.push((pred - case.reference_hp) / case.reference_hp);
        }
    }
    let (mean, sd) = mean_sd(&errs);
    ReducedFormFit {
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
        ..fit
    }
}

/// Fit a reduced-form regression with ship-type dummy variables.
/// Reference category is `battleship`; dummies are carrier/cruiser/destroyer.
pub fn fit_reduced_form_by_type() -> ReducedFormByTypeFit {
    let cases = historical_cases();
    let mut a: Vec<Vec<f64>> = Vec::new();
    let mut b: Vec<f64> = Vec::new();
    for case in cases.iter() {
        if case.reference_hp <= 0.0 || case.displacement_tons <= 0.0 || case.speed_kt <= 0.0 {
            continue;
        }
        a.push(vec![
            1.0,
            case.displacement_tons.ln(),
            case.speed_kt.ln(),
            case.length_ft.ln(),
            if case.ship_type == "carrier" {
                1.0
            } else {
                0.0
            },
            if case.ship_type == "cruiser" {
                1.0
            } else {
                0.0
            },
            if case.ship_type == "destroyer" {
                1.0
            } else {
                0.0
            },
        ]);
        b.push(case.reference_hp.ln());
    }
    let n = b.len();
    if n < 7 {
        return ReducedFormByTypeFit {
            intercept: 0.0,
            log_displacement: 0.0,
            log_speed: 0.0,
            log_length: 0.0,
            d_carrier: 0.0,
            d_cruiser: 0.0,
            d_destroyer: 0.0,
            mean_abs_pct_error: f64::INFINITY,
            sigma_abs_pct_error: f64::INFINITY,
        };
    }
    let coef = solve_qr(&a, &b).unwrap_or_default();
    let fit = ReducedFormByTypeFit {
        intercept: coef[0],
        log_displacement: coef[1],
        log_speed: coef[2],
        log_length: coef[3],
        d_carrier: coef[4],
        d_cruiser: coef[5],
        d_destroyer: coef[6],
        mean_abs_pct_error: 0.0,
        sigma_abs_pct_error: 0.0,
    };
    let mut errs = Vec::new();
    for case in cases.iter() {
        if case.reference_hp <= 0.0 {
            continue;
        }
        let pred = fit.predict(
            case.displacement_tons,
            case.speed_kt,
            case.length_ft,
            case.ship_type,
        );
        if pred > 0.0 {
            errs.push((pred - case.reference_hp) / case.reference_hp);
        }
    }
    let (mean, sd) = mean_sd(&errs);
    ReducedFormByTypeFit {
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
        ..fit
    }
}

/// Split a Holtrop resistance result into three structural power components.
fn structural_kw(
    case: &crate::accuracy::HistoricalCase,
    res: &HoltropResistance,
) -> (f64, f64, f64) {
    let v = case.speed_kt * 0.514444;
    let fric_kw = (res.friction_n + res.appendage_n) * v / 1000.0;
    let wave_kw = (res.wave_n + res.bulb_n + res.transom_n) * v / 1000.0;
    let corr_kw = res.correlation_n * v / 1000.0;
    (fric_kw, wave_kw, corr_kw)
}

/// Fit global structural component multipliers.
pub fn fit_structural_form() -> StructuralFormFit {
    let cases = historical_cases();
    let mut rows = Vec::new();
    let mut targets = Vec::new();

    for case in cases.iter() {
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        rows.push(vec![f, w]);
        targets.push(case.reference_hp * 0.7456999 - c);
    }

    let n = targets.len();
    if n < 2 {
        return StructuralFormFit {
            viscous_factor: 1.0,
            wave_factor: 1.0,
            correlation_factor: 1.0,
            mean_abs_pct_error: f64::INFINITY,
            sigma_abs_pct_error: f64::INFINITY,
        };
    }
    let coef = solve_qr(&rows, &targets).unwrap_or_else(|| vec![1.0, 1.0]);

    let mut errs = Vec::new();
    for case in cases.iter() {
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let pred = coef[0] * f + coef[1] * w + c;
        let target = case.reference_hp * 0.7456999;
        if target > 0.0 {
            errs.push((pred - target) / target);
        }
    }
    let (mean, sd) = mean_sd(&errs);
    StructuralFormFit {
        viscous_factor: coef[0],
        wave_factor: coef[1],
        correlation_factor: 1.0,
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
    }
}

/// Fit structural component multipliers separately for each Froude band.
pub fn fit_structural_form_segmented() -> Vec<StructuralSegment> {
    let bands: [(&'static str, f64, f64); 3] = [
        ("low Fn (<=0.18)", f64::NEG_INFINITY, 0.18),
        ("medium Fn (0.18-0.28)", 0.18, 0.28),
        ("high Fn (>0.28)", 0.28, f64::INFINITY),
    ];
    bands
        .iter()
        .map(|(name, lo, hi)| {
            let all_cases = historical_cases();
            let subset: Vec<_> = all_cases
                .iter()
                .filter(|c| {
                    let fn_ = froude_number(c.speed_kt, c.length_ft);
                    fn_ > *lo && fn_ <= *hi
                })
                .collect();
            if subset.is_empty() {
                return StructuralSegment {
                    name,
                    min_froude: *lo,
                    max_froude: *hi,
                    viscous_factor: 1.0,
                    wave_factor: 1.0,
                    correlation_factor: 1.0,
                    mean_abs_pct_error: f64::INFINITY,
                    sigma_abs_pct_error: f64::INFINITY,
                };
            }
            let mut rows = Vec::new();
            let mut targets = Vec::new();
            for case in &subset {
                let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
                let input = input_from_hull(&ship.hull, case.speed_kt);
                let res = holtrop_resistance(&input);
                let (f, w, c) = structural_kw(case, &res);
                rows.push(vec![f, w]);
                targets.push(case.reference_hp * 0.7456999 - c);
            }
            let coef = solve_qr(&rows, &targets).unwrap_or_else(|| vec![1.0, 1.0]);
            let mut errs = Vec::new();
            for (i, case) in subset.iter().enumerate() {
                let pred = coef[0] * rows[i][0]
                    + coef[1] * rows[i][1]
                    + (case.reference_hp * 0.7456999 - targets[i]);
                let target = case.reference_hp * 0.7456999;
                if target > 0.0 {
                    errs.push((pred - target) / target);
                }
            }
            let (mean, sd) = mean_sd(&errs);
            StructuralSegment {
                name,
                min_froude: *lo,
                max_froude: *hi,
                viscous_factor: coef[0],
                wave_factor: coef[1],
                correlation_factor: 1.0,
                mean_abs_pct_error: mean.abs(),
                sigma_abs_pct_error: sd,
            }
        })
        .collect()
}

/// Fit an inverse-error weighted ensemble of reduced and structural forms.
pub fn fit_ensemble() -> EnsembleFit {
    let rf = fit_reduced_form();
    let sf = fit_structural_form();
    let w_r = 1.0 / rf.mean_abs_pct_error.max(1e-9);
    let w_s = 1.0 / sf.mean_abs_pct_error.max(1e-9);
    let w_r = w_r / (w_r + w_s);
    let w_s = 1.0 - w_r;

    let mut errs = Vec::new();
    for case in historical_cases().iter() {
        if case.reference_hp <= 0.0 {
            continue;
        }
        let r_pred = rf.predict(case.displacement_tons, case.speed_kt, case.length_ft);
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let s_pred = sf.predict_hp(f, w, c);
        let pred = w_r * r_pred + w_s * s_pred;
        errs.push((pred - case.reference_hp) / case.reference_hp);
    }
    let (mean, sd) = mean_sd(&errs);
    EnsembleFit {
        reduced_weight: w_r,
        structural_weight: w_s,
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
    }
}

/// Apply the global fitted structural multipliers.
pub fn apply_structural_calibration(ship: &mut ShipModel) {
    let fit = fit_structural_form();
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.blend_weight = 1.0;
    ship.engine.holtrop_viscous_factor = fit.viscous_factor;
    ship.engine.holtrop_wave_factor = fit.wave_factor;
    ship.engine.holtrop_correlation_factor = fit.correlation_factor;
}

/// Apply Froude-segmented structural multipliers to a ship.
pub fn apply_structural_calibration_segmented(ship: &mut ShipModel) {
    let segments = fit_structural_form_segmented();
    let fn_ship = froude_number(ship.engine.speed_max, ship.hull.length);
    let seg = segments
        .iter()
        .find(|s| fn_ship > s.min_froude && fn_ship <= s.max_froude)
        .or_else(|| segments.first());
    if let Some(seg) = seg {
        ship.engine.resistance_model = ResistanceModel::HoltropMennen;
        ship.engine.blend_weight = 1.0;
        ship.engine.holtrop_viscous_factor = seg.viscous_factor;
        ship.engine.holtrop_wave_factor = seg.wave_factor;
        ship.engine.holtrop_correlation_factor = seg.correlation_factor;
    }
}

// --- Additional cross-validation and regularized estimators ---

/// Solve least squares using SVD of A^T A (Jacobi symmetric eigensolver).
/// This is provided for cross-checking the QR solution on small datasets.
fn solve_svd(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = a.first()?.len();
    let m = a.len();
    let mut ata = vec![vec![0.0; n]; n];
    let mut atb = vec![0.0; n];
    for i in 0..m {
        for r in 0..n {
            for c in 0..n {
                ata[r][c] += a[i][r] * a[i][c];
            }
            atb[r] += a[i][r] * b[i];
        }
    }
    let (eigenvalues, eigenvectors) = jacobi_eigen(&ata)?;
    let mut x = vec![0.0; n];
    for j in 0..n {
        let lambda = eigenvalues[j];
        if lambda < 1e-12 {
            continue;
        }
        let dot: f64 = (0..n).map(|r| eigenvectors[r][j] * atb[r]).sum();
        let scale = dot / lambda;
        for r in 0..n {
            x[r] += eigenvectors[r][j] * scale;
        }
    }
    Some(x)
}

/// Jacobi eigenvalue decomposition for a symmetric matrix.
/// Returns (eigenvalues, eigenvectors-as-columns).
fn jacobi_eigen(a: &[Vec<f64>]) -> Option<(Vec<f64>, Vec<Vec<f64>>)> {
    let n = a.len();
    if n == 0 {
        return None;
    }
    let mut a = a.to_vec();
    let mut v = vec![vec![0.0; n]; n];
    for i in 0..n {
        v[i][i] = 1.0;
    }
    for _ in 0..200 {
        let mut p = 0;
        let mut q = 1;
        let mut max = 0.0;
        for i in 0..n {
            for j in i + 1..n {
                if a[i][j].abs() > max {
                    max = a[i][j].abs();
                    p = i;
                    q = j;
                }
            }
        }
        if max < 1e-12 {
            break;
        }
        let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
        let t = if theta >= 0.0 {
            1.0 / (theta + (1.0 + theta * theta).sqrt())
        } else {
            -1.0 / (-theta + (1.0 + theta * theta).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;
        for k in 0..n {
            if k == p || k == q {
                continue;
            }
            let akp = a[k][p];
            let akq = a[k][q];
            a[k][p] = c * akp - s * akq;
            a[p][k] = a[k][p];
            a[k][q] = s * akp + c * akq;
            a[q][k] = a[k][q];
        }
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;
        for k in 0..n {
            let vkp = v[k][p];
            let vkq = v[k][q];
            v[k][p] = c * vkp - s * vkq;
            v[k][q] = s * vkp + c * vkq;
        }
    }
    let eigenvalues = (0..n).map(|i| a[i][i]).collect();
    Some((eigenvalues, v))
}

fn reduced_form_fit_from_cases(cases: &[crate::accuracy::HistoricalCase]) -> ReducedFormFit {
    let mut a: Vec<Vec<f64>> = Vec::new();
    let mut b: Vec<f64> = Vec::new();
    for case in cases {
        if case.reference_hp <= 0.0 || case.displacement_tons <= 0.0 || case.speed_kt <= 0.0 {
            continue;
        }
        a.push(vec![
            1.0,
            case.displacement_tons.ln(),
            case.speed_kt.ln(),
            case.length_ft.ln(),
        ]);
        b.push(case.reference_hp.ln());
    }
    let coef = solve_qr(&a, &b).unwrap_or_default();
    let fit = ReducedFormFit {
        intercept: coef[0],
        log_displacement: coef[1],
        log_speed: coef[2],
        log_length: coef[3],
        mean_abs_pct_error: 0.0,
        sigma_abs_pct_error: 0.0,
    };
    let mut errs = Vec::new();
    for case in cases {
        if case.reference_hp > 0.0 {
            let pred = fit.predict(case.displacement_tons, case.speed_kt, case.length_ft);
            errs.push((pred - case.reference_hp) / case.reference_hp);
        }
    }
    let (mean, sd) = mean_sd(&errs);
    ReducedFormFit {
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
        ..fit
    }
}

fn structural_form_fit_from_cases(cases: &[crate::accuracy::HistoricalCase]) -> StructuralFormFit {
    let mut rows = Vec::new();
    let mut targets = Vec::new();
    for case in cases {
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        rows.push(vec![f, w]);
        targets.push(case.reference_hp * 0.7456999 - c);
    }
    let coef = solve_qr(&rows, &targets).unwrap_or_else(|| vec![1.0, 1.0]);
    let mut errs = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let target = case.reference_hp * 0.7456999;
        let pred = coef[0] * rows[i][0] + coef[1] * rows[i][1] + (target - targets[i]);
        if target > 0.0 {
            errs.push((pred - target) / target);
        }
    }
    let (mean, sd) = mean_sd(&errs);
    StructuralFormFit {
        viscous_factor: coef[0],
        wave_factor: coef[1],
        correlation_factor: 1.0,
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
    }
}

fn reduced_form_fit_from_cases_weighted(
    cases: &[crate::accuracy::HistoricalCase],
) -> ReducedFormFit {
    let mut a: Vec<Vec<f64>> = Vec::new();
    let mut b: Vec<f64> = Vec::new();
    for case in cases {
        if case.reference_hp <= 0.0 {
            continue;
        }
        let w = case_confidence(case.name).sqrt();
        a.push(vec![
            w,
            w * case.displacement_tons.ln(),
            w * case.speed_kt.ln(),
            w * case.length_ft.ln(),
        ]);
        b.push(w * case.reference_hp.ln());
    }
    let coef = solve_qr(&a, &b).unwrap_or_default();
    let fit = ReducedFormFit {
        intercept: coef[0],
        log_displacement: coef[1],
        log_speed: coef[2],
        log_length: coef[3],
        mean_abs_pct_error: 0.0,
        sigma_abs_pct_error: 0.0,
    };
    let mut errs = Vec::new();
    for case in cases {
        let pred = fit.predict(case.displacement_tons, case.speed_kt, case.length_ft);
        errs.push((pred - case.reference_hp) / case.reference_hp);
    }
    let (mean, sd) = mean_sd(&errs);
    ReducedFormFit {
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
        ..fit
    }
}

fn structural_form_fit_from_cases_weighted(
    cases: &[crate::accuracy::HistoricalCase],
) -> StructuralFormFit {
    let mut rows = Vec::new();
    let mut targets = Vec::new();
    for case in cases {
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let weight = case_confidence(case.name).sqrt();
        rows.push(vec![weight * f, weight * w]);
        targets.push(weight * (case.reference_hp * 0.7456999 - c));
    }
    let coef = solve_qr(&rows, &targets).unwrap_or_else(|| vec![1.0, 1.0]);
    let mut errs = Vec::new();
    for case in cases {
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let pred = coef[0] * f + coef[1] * w + c;
        let target = case.reference_hp * 0.7456999;
        if target > 0.0 {
            errs.push((pred - target) / target);
        }
    }
    let (mean, sd) = mean_sd(&errs);
    StructuralFormFit {
        viscous_factor: coef[0],
        wave_factor: coef[1],
        correlation_factor: 1.0,
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
    }
}

#[derive(Clone, Debug)]
pub struct LoocvFold {
    pub name: &'static str,
    pub reference_hp: f64,
    pub reduced_hp: f64,
    pub structural_hp: f64,
}

#[derive(Clone, Debug)]
pub struct LoocvReport {
    pub reduced_mape: f64,
    pub structural_mape: f64,
    pub ensemble_mape: f64,
    pub optimal_reduced_weight: f64,
    pub folds: Vec<LoocvFold>,
}

/// Leave-one-out cross-validation report.
///
/// For each fold the model is fitted on the other 14 cases and used to predict
/// the held-out case. The ensemble weight is then chosen by minimizing the
/// resulting LOOCV ensemble MAPE.
pub fn run_loocv() -> LoocvReport {
    let cases = historical_cases();
    let mut folds = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let train: Vec<_> = cases
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, c)| *c)
            .collect();
        let rf = reduced_form_fit_from_cases(&train);
        let sf = structural_form_fit_from_cases(&train);
        let reduced_hp = rf.predict(case.displacement_tons, case.speed_kt, case.length_ft);
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let structural_hp = sf.predict_hp(f, w, c);
        folds.push(LoocvFold {
            name: case.name,
            reference_hp: case.reference_hp,
            reduced_hp,
            structural_hp,
        });
    }

    let n = folds.len() as f64;
    let reduced_mape = folds
        .iter()
        .map(|f| ((f.reduced_hp - f.reference_hp) / f.reference_hp).abs())
        .sum::<f64>()
        / n;
    let structural_mape = folds
        .iter()
        .map(|f| ((f.structural_hp - f.reference_hp) / f.reference_hp).abs())
        .sum::<f64>()
        / n;

    let mut best_w = 0.5_f64;
    let mut best_mape = f64::INFINITY;
    for k in 0..=100 {
        let w = k as f64 / 100.0;
        let mape = folds
            .iter()
            .map(|f| {
                let pred = w * f.reduced_hp + (1.0 - w) * f.structural_hp;
                ((pred - f.reference_hp) / f.reference_hp).abs()
            })
            .sum::<f64>()
            / n;
        if mape < best_mape {
            best_mape = mape;
            best_w = w;
        }
    }

    LoocvReport {
        reduced_mape,
        structural_mape,
        ensemble_mape: best_mape,
        optimal_reduced_weight: best_w,
        folds,
    }
}

/// Per-ship-type LOOCV MAPE using the globally LOOCV-optimised ensemble
/// weight.  This is the out-of-sample accuracy input for the per-type quality
/// gate.
pub fn loocv_by_type() -> Vec<(String, usize, f64)> {
    let report = run_loocv();
    let w = report.optimal_reduced_weight;
    let cases = historical_cases();
    let mut out: Vec<(String, usize, f64)> = Vec::new();
    for fold in &report.folds {
        let ty = cases
            .iter()
            .find(|c| c.name == fold.name)
            .map(|c| c.ship_type)
            .unwrap_or("unknown");
        let pred = w * fold.reduced_hp + (1.0 - w) * fold.structural_hp;
        let err = ((pred - fold.reference_hp) / fold.reference_hp).abs();
        if let Some(g) = out.iter_mut().find(|g| g.0 == ty) {
            g.1 += 1;
            g.2 += err;
        } else {
            out.push((ty.to_string(), 1, err));
        }
    }
    for g in &mut out {
        g.2 /= g.1 as f64;
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Confidence-weighted leave-one-out cross-validation report.
pub fn run_loocv_weighted() -> LoocvReport {
    let cases = historical_cases();
    let mut folds = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let train: Vec<_> = cases
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, c)| *c)
            .collect();
        let rf = reduced_form_fit_from_cases_weighted(&train);
        let sf = structural_form_fit_from_cases_weighted(&train);
        let reduced_hp = rf.predict(case.displacement_tons, case.speed_kt, case.length_ft);
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let structural_hp = sf.predict_hp(f, w, c);
        folds.push(LoocvFold {
            name: case.name,
            reference_hp: case.reference_hp,
            reduced_hp,
            structural_hp,
        });
    }
    let n = folds.len() as f64;
    let reduced_mape = folds
        .iter()
        .map(|f| ((f.reduced_hp - f.reference_hp) / f.reference_hp).abs())
        .sum::<f64>()
        / n;
    let structural_mape = folds
        .iter()
        .map(|f| ((f.structural_hp - f.reference_hp) / f.reference_hp).abs())
        .sum::<f64>()
        / n;
    let mut best_w = 0.5_f64;
    let mut best_mape = f64::INFINITY;
    for k in 0..=100 {
        let w = k as f64 / 100.0;
        let mape = folds
            .iter()
            .map(|f| {
                let pred = w * f.reduced_hp + (1.0 - w) * f.structural_hp;
                ((pred - f.reference_hp) / f.reference_hp).abs()
            })
            .sum::<f64>()
            / n;
        if mape < best_mape {
            best_mape = mape;
            best_w = w;
        }
    }
    LoocvReport {
        reduced_mape,
        structural_mape,
        ensemble_mape: best_mape,
        optimal_reduced_weight: best_w,
        folds,
    }
}

/// Ensemble fit with weights optimized by LOOCV.
pub fn fit_ensemble_cv() -> EnsembleFit {
    let report = run_loocv();
    EnsembleFit {
        reduced_weight: report.optimal_reduced_weight,
        structural_weight: 1.0 - report.optimal_reduced_weight,
        mean_abs_pct_error: report.ensemble_mape,
        sigma_abs_pct_error: report.structural_mape, // placeholder, replaced below
    }
}

/// Compare QR and SVD least-squares solutions for the simple reduced form.
pub fn compare_qr_svd() -> (ReducedFormFit, ReducedFormFit, f64) {
    let cases = historical_cases();
    let mut a: Vec<Vec<f64>> = Vec::new();
    let mut b: Vec<f64> = Vec::new();
    for case in &cases {
        if case.reference_hp <= 0.0 {
            continue;
        }
        a.push(vec![
            1.0,
            case.displacement_tons.ln(),
            case.speed_kt.ln(),
            case.length_ft.ln(),
        ]);
        b.push(case.reference_hp.ln());
    }
    let qr_coef = solve_qr(&a, &b).unwrap_or_default();
    let svd_coef = solve_svd(&a, &b).unwrap_or_default();
    let max_diff = qr_coef
        .iter()
        .zip(svd_coef.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0_f64, f64::max);
    let fit_qr = ReducedFormFit {
        intercept: qr_coef[0],
        log_displacement: qr_coef[1],
        log_speed: qr_coef[2],
        log_length: qr_coef[3],
        mean_abs_pct_error: 0.0,
        sigma_abs_pct_error: 0.0,
    };
    let fit_svd = ReducedFormFit {
        intercept: svd_coef[0],
        log_displacement: svd_coef[1],
        log_speed: svd_coef[2],
        log_length: svd_coef[3],
        mean_abs_pct_error: 0.0,
        sigma_abs_pct_error: 0.0,
    };
    (fit_qr, fit_svd, max_diff)
}

/// Reduced-form with Froude x ship-type interaction terms.
#[derive(Clone, Debug)]
pub struct ReducedFormInteractionFit {
    pub intercept: f64,
    pub log_displacement: f64,
    pub log_speed: f64,
    pub log_length: f64,
    pub d_carrier: f64,
    pub d_cruiser: f64,
    pub d_destroyer: f64,
    pub fn_carrier: f64,
    pub fn_cruiser: f64,
    pub fn_destroyer: f64,
    pub mean_abs_pct_error: f64,
    pub sigma_abs_pct_error: f64,
}

impl ReducedFormInteractionFit {
    pub fn predict(
        &self,
        displacement: f64,
        speed_kt: f64,
        length_ft: f64,
        ship_type: &str,
    ) -> f64 {
        let fn_ = froude_number(speed_kt, length_ft);
        let d_carrier = if ship_type == "carrier" { 1.0 } else { 0.0 };
        let d_cruiser = if ship_type == "cruiser" { 1.0 } else { 0.0 };
        let d_destroyer = if ship_type == "destroyer" { 1.0 } else { 0.0 };
        (self.intercept
            + self.log_displacement * displacement.ln()
            + self.log_speed * speed_kt.ln()
            + self.log_length * length_ft.ln()
            + self.d_carrier * d_carrier
            + self.d_cruiser * d_cruiser
            + self.d_destroyer * d_destroyer
            + self.fn_carrier * fn_ * d_carrier
            + self.fn_cruiser * fn_ * d_cruiser
            + self.fn_destroyer * fn_ * d_destroyer)
            .exp()
    }
}

pub fn fit_reduced_form_interaction() -> ReducedFormInteractionFit {
    let cases = historical_cases();
    let mut a: Vec<Vec<f64>> = Vec::new();
    let mut b: Vec<f64> = Vec::new();
    for case in cases.iter() {
        if case.reference_hp <= 0.0 {
            continue;
        }
        let fn_ = froude_number(case.speed_kt, case.length_ft);
        let dcar = if case.ship_type == "carrier" {
            1.0
        } else {
            0.0
        };
        let dcru = if case.ship_type == "cruiser" {
            1.0
        } else {
            0.0
        };
        let ddes = if case.ship_type == "destroyer" {
            1.0
        } else {
            0.0
        };
        a.push(vec![
            1.0,
            case.displacement_tons.ln(),
            case.speed_kt.ln(),
            case.length_ft.ln(),
            dcar,
            dcru,
            ddes,
            fn_ * dcar,
            fn_ * dcru,
            fn_ * ddes,
        ]);
        b.push(case.reference_hp.ln());
    }
    let coef = solve_qr(&a, &b).unwrap_or_default();
    let fit = ReducedFormInteractionFit {
        intercept: coef[0],
        log_displacement: coef[1],
        log_speed: coef[2],
        log_length: coef[3],
        d_carrier: coef[4],
        d_cruiser: coef[5],
        d_destroyer: coef[6],
        fn_carrier: coef[7],
        fn_cruiser: coef[8],
        fn_destroyer: coef[9],
        mean_abs_pct_error: 0.0,
        sigma_abs_pct_error: 0.0,
    };
    let mut errs = Vec::new();
    for case in cases.iter() {
        let pred = fit.predict(
            case.displacement_tons,
            case.speed_kt,
            case.length_ft,
            case.ship_type,
        );
        errs.push((pred - case.reference_hp) / case.reference_hp);
    }
    let (mean, sd) = mean_sd(&errs);
    ReducedFormInteractionFit {
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
        ..fit
    }
}

/// Fit ridge-regularized Fn-segmented structural multipliers.
pub fn fit_structural_form_segmented_ridge(lambda: f64) -> Vec<StructuralSegment> {
    fit_structural_form_segmented_regularized(lambda, 0.0)
}

/// Fit lasso-regularized (coordinate descent) Fn-segmented structural multipliers.
pub fn fit_structural_form_segmented_lasso(lambda: f64) -> Vec<StructuralSegment> {
    fit_structural_form_segmented_regularized(0.0, lambda)
}

/// Scale each column to unit RMS before regularisation.
///
/// Returns `(scaled_rows, scale)` such that `raw = beta / scale`.
fn standardize_two_columns(rows: &[Vec<f64>]) -> (Vec<Vec<f64>>, [f64; 2]) {
    let n = rows.len().max(1) as f64;
    let mut scale = [1.0_f64; 2];
    for j in 0..2 {
        let ss: f64 = rows.iter().map(|r| r[j] * r[j]).sum();
        let rms = (ss / n).sqrt();
        if rms > 1e-12 {
            scale[j] = rms;
        }
    }
    let z = rows
        .iter()
        .map(|r| vec![r[0] / scale[0], r[1] / scale[1]])
        .collect();
    (z, scale)
}

/// Fit `a_v * f + a_w * w = y`, optionally with ridge or lasso regularisation.
///
/// Features are scaled to unit RMS first, so the same lambda grid is
/// meaningful whether the components are in kW or hp.  Coefficients are
/// converted back to the original units.
fn fit_two_component_regularized(
    rows: &[Vec<f64>],
    targets: &[f64],
    ridge: f64,
    lasso: f64,
) -> Vec<f64> {
    if rows.is_empty() {
        return vec![1.0, 1.0];
    }
    let (z, scale) = standardize_two_columns(rows);
    // Normalising the target as well keeps the regularisation strength
    // dimensionless: without it, lambda would be swamped by the kW/hp scale of
    // the power components and the grid would be a no-op.
    let y_rms = (targets.iter().map(|y| y * y).sum::<f64>() / targets.len().max(1) as f64).sqrt();
    let y_scale = if y_rms > 1e-12 { y_rms } else { 1.0 };
    let y_norm: Vec<f64> = targets.iter().map(|y| y / y_scale).collect();
    let beta = if lasso > 0.0 {
        lasso_coordinate_descent(&z, &y_norm, lasso)
    } else if ridge > 0.0 {
        let mut ar = z.clone();
        let mut br = y_norm.clone();
        for j in 0..2 {
            let mut row = vec![0.0; 2];
            row[j] = ridge.max(0.0).sqrt();
            ar.push(row);
            br.push(0.0);
        }
        solve_qr(&ar, &br).unwrap_or_else(|| vec![1.0, 1.0])
    } else {
        solve_qr(&z, &y_norm).unwrap_or_else(|| vec![1.0, 1.0])
    };
    vec![beta[0] * y_scale / scale[0], beta[1] * y_scale / scale[1]]
}

fn fit_structural_form_segmented_regularized(ridge: f64, lasso: f64) -> Vec<StructuralSegment> {
    let bands: [(&'static str, f64, f64); 3] = [
        ("low Fn (<=0.18)", f64::NEG_INFINITY, 0.18),
        ("medium Fn (0.18-0.28)", 0.18, 0.28),
        ("high Fn (>0.28)", 0.28, f64::INFINITY),
    ];
    let all_cases = historical_cases();
    bands
        .iter()
        .map(|(name, lo, hi)| {
            let subset: Vec<_> = all_cases
                .iter()
                .filter(|c| {
                    let fn_ = froude_number(c.speed_kt, c.length_ft);
                    fn_ > *lo && fn_ <= *hi
                })
                .collect();
            if subset.is_empty() {
                return StructuralSegment {
                    name,
                    min_froude: *lo,
                    max_froude: *hi,
                    viscous_factor: 1.0,
                    wave_factor: 1.0,
                    correlation_factor: 1.0,
                    mean_abs_pct_error: f64::INFINITY,
                    sigma_abs_pct_error: f64::INFINITY,
                };
            }
            let mut rows = Vec::new();
            let mut targets = Vec::new();
            for case in &subset {
                let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
                let input = input_from_hull(&ship.hull, case.speed_kt);
                let res = holtrop_resistance(&input);
                let (f, w, c) = structural_kw(case, &res);
                rows.push(vec![f, w]);
                targets.push(case.reference_hp * 0.7456999 - c);
            }
            let coef = fit_two_component_regularized(&rows, &targets, ridge, lasso);

            let mut errs = Vec::new();
            for (i, case) in subset.iter().enumerate() {
                let target = case.reference_hp * 0.7456999;
                let pred = coef[0] * rows[i][0] + coef[1] * rows[i][1] + (target - targets[i]);
                if target > 0.0 {
                    errs.push((pred - target) / target);
                }
            }
            let (mean, sd) = mean_sd(&errs);
            StructuralSegment {
                name,
                min_froude: *lo,
                max_froude: *hi,
                viscous_factor: coef[0],
                wave_factor: coef[1],
                correlation_factor: 1.0,
                mean_abs_pct_error: mean.abs(),
                sigma_abs_pct_error: sd,
            }
        })
        .collect()
}

/// Coordinate descent for a two-coefficient lasso problem.
fn lasso_coordinate_descent(rows: &[Vec<f64>], target: &[f64], lambda: f64) -> Vec<f64> {
    let mut x = vec![0.0_f64, 0.0_f64];
    for _ in 0..200 {
        for j in 0..2 {
            let mut rho = 0.0_f64;
            let mut rj = 0.0_f64;
            for (i, row) in rows.iter().enumerate() {
                let pred_other = (0..2)
                    .filter(|&k| k != j)
                    .map(|k| row[k] * x[k])
                    .sum::<f64>();
                rj += row[j] * (target[i] - pred_other);
                rho += row[j] * row[j];
            }
            if rho > 1e-12 {
                let z = rj / rho;
                let threshold = lambda / rho;
                x[j] = if z > threshold {
                    z - threshold
                } else if z < -threshold {
                    z + threshold
                } else {
                    0.0
                };
            }
        }
    }
    x
}

/// --- Lambda selection and bootstrap uncertainty ---

#[derive(Clone, Debug)]
pub struct LambdaSelection {
    pub best_lambda: f64,
    pub best_loocv_mape: f64,
    pub results: Vec<(f64, f64)>,
}

fn structural_form_fit_from_cases_regularized(
    cases: &[crate::accuracy::HistoricalCase],
    ridge: f64,
    lasso: f64,
) -> StructuralFormFit {
    let mut rows = Vec::new();
    let mut targets = Vec::new();
    for case in cases {
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        rows.push(vec![f, w]);
        targets.push(case.reference_hp * 0.7456999 - c);
    }
    let coef = fit_two_component_regularized(&rows, &targets, ridge, lasso);
    // Real in-sample error, not a hardcoded placeholder.
    let mut signed = Vec::new();
    let mut abs_sum = 0.0_f64;
    for (i, case) in cases.iter().enumerate() {
        let target = case.reference_hp * 0.7456999;
        let pred = coef[0] * rows[i][0] + coef[1] * rows[i][1] + (target - targets[i]);
        if target > 0.0 {
            let rel = (pred - target) / target;
            signed.push(rel);
            abs_sum += rel.abs();
        }
    }
    let mape = if signed.is_empty() {
        f64::INFINITY
    } else {
        abs_sum / signed.len() as f64
    };
    let (_mean, sd) = mean_sd(&signed);
    StructuralFormFit {
        viscous_factor: coef[0],
        wave_factor: coef[1],
        correlation_factor: 1.0,
        mean_abs_pct_error: mape,
        sigma_abs_pct_error: sd,
    }
}

fn structural_form_fit_from_cases_ridge(
    cases: &[crate::accuracy::HistoricalCase],
    lambda: f64,
) -> StructuralFormFit {
    structural_form_fit_from_cases_regularized(cases, lambda, 0.0)
}

fn structural_form_fit_from_cases_lasso(
    cases: &[crate::accuracy::HistoricalCase],
    lambda: f64,
) -> StructuralFormFit {
    structural_form_fit_from_cases_regularized(cases, 0.0, lambda)
}

fn loocv_structural_mape(
    cases: &[crate::accuracy::HistoricalCase],
    regularizer: &str,
    lambda: f64,
) -> f64 {
    let mut errors = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let train: Vec<_> = cases
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, c)| *c)
            .collect();
        let fit = if regularizer == "ridge" {
            structural_form_fit_from_cases_ridge(&train, lambda)
        } else if regularizer == "lasso" {
            structural_form_fit_from_cases_lasso(&train, lambda)
        } else {
            structural_form_fit_from_cases(&train)
        };
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let pred = fit.predict_hp(f, w, c);
        errors.push(((pred - case.reference_hp) / case.reference_hp).abs());
    }
    if errors.is_empty() {
        f64::INFINITY
    } else {
        errors.iter().sum::<f64>() / errors.len() as f64
    }
}

/// Lambda grid for regularised structural fits.  Features are standardized to
/// unit RMS, so this range spans "essentially unregularised" to
/// "coefficients driven to zero".
const LAMBDA_CANDIDATES: [f64; 9] = [1e-4, 1e-3, 1e-2, 1e-1, 1.0, 1e1, 1e2, 1e3, 1e4];

/// Select Ridge lambda by structural-form LOOCV.
pub fn select_ridge_lambda() -> LambdaSelection {
    let cases = historical_cases();
    let candidates = LAMBDA_CANDIDATES;
    let mut best = (f64::INFINITY, f64::INFINITY);
    let mut results = Vec::new();
    for lambda in candidates {
        let mape = loocv_structural_mape(&cases, "ridge", lambda);
        results.push((lambda, mape));
        if mape < best.1 {
            best = (lambda, mape);
        }
    }
    LambdaSelection {
        best_lambda: best.0,
        best_loocv_mape: best.1,
        results,
    }
}

/// Select Lasso lambda by structural-form LOOCV.
pub fn select_lasso_lambda() -> LambdaSelection {
    let cases = historical_cases();
    let candidates = LAMBDA_CANDIDATES;
    let mut best = (f64::INFINITY, f64::INFINITY);
    let mut results = Vec::new();
    for lambda in candidates {
        let mape = loocv_structural_mape(&cases, "lasso", lambda);
        results.push((lambda, mape));
        if mape < best.1 {
            best = (lambda, mape);
        }
    }
    LambdaSelection {
        best_lambda: best.0,
        best_loocv_mape: best.1,
        results,
    }
}

/// Simple percentile confidence interval.
#[derive(Clone, Debug)]
pub struct ConfidenceInterval {
    pub lower: f64,
    pub median: f64,
    pub upper: f64,
}

#[derive(Clone, Debug)]
pub struct BootstrapUncertainty {
    pub reduced: ConfidenceInterval,
    pub structural: ConfidenceInterval,
    pub ensemble: ConfidenceInterval,
}

fn percentile(sorted: &mut [f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    // `total_cmp` orders NaN deterministically instead of panicking, which is
    // important because bootstrap samples can occasionally contain non-finite
    // predictions if an input is degenerate.
    sorted.sort_by(f64::total_cmp);
    let idx = (q * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Bootstrap prediction intervals for a given design point.
///
/// Uses the same historical database, resampled with replacement, to estimate
/// 5th/50th/95th percentiles for reduced-form, structural-form and ensemble
/// predictions.
pub fn bootstrap_uncertainty_for_ship(
    ship: &crate::model::ShipModel,
    n_boot: usize,
) -> BootstrapUncertainty {
    let cases = historical_cases();
    let speed = ship.engine.speed_max;
    let disp = ship.hull.displacement;
    let length = ship.hull.length;
    let _ship_type = "battleship"; // generic; can be refined later
    let input = input_from_hull(&ship.hull, speed);
    let res = holtrop_resistance(&input);
    let v = speed * 0.514444;
    let fric_kw = (res.friction_n + res.appendage_n) * v / 1000.0;
    let wave_kw = (res.wave_n + res.bulb_n + res.transom_n) * v / 1000.0;
    let corr_kw = res.correlation_n * v / 1000.0;
    let loocv = run_loocv();
    let w_red = loocv.optimal_reduced_weight;

    // Parallel bootstrap using std threads (no external dependencies).
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8);
    let chunk_size = n_boot.div_ceil(n_threads);
    let mut handles = Vec::new();
    let cases_owned = cases.clone();
    for t in 0..n_threads {
        let start = t * chunk_size;
        let end = ((t + 1) * chunk_size).min(n_boot);
        if start >= end {
            continue;
        }
        let cases_ref = cases_owned.clone();
        handles.push(std::thread::spawn(move || {
            let mut rng = 12345_u64.wrapping_add((t as u64).wrapping_mul(0x9E3779B97F4A7C15));
            let mut reduced = Vec::new();
            let mut structural = Vec::new();
            let mut ensemble = Vec::new();
            for _ in start..end {
                let mut sample = Vec::with_capacity(cases_ref.len());
                for _ in 0..cases_ref.len() {
                    rng = rng
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let idx = (rng >> 33) as usize % cases_ref.len();
                    sample.push(cases_ref[idx]);
                }
                let rf = reduced_form_fit_from_cases(&sample);
                let sf = structural_form_fit_from_cases(&sample);
                let rp = rf.predict(disp, speed, length);
                let sp = sf.predict_hp(fric_kw, wave_kw, corr_kw);
                reduced.push(rp);
                structural.push(sp);
                ensemble.push(w_red * rp + (1.0 - w_red) * sp);
            }
            (reduced, structural, ensemble)
        }));
    }

    let mut reduced_preds = Vec::with_capacity(n_boot);
    let mut structural_preds = Vec::with_capacity(n_boot);
    let mut ensemble_preds = Vec::with_capacity(n_boot);
    for handle in handles {
        let (r, s, e) = handle.join().expect("bootstrap thread panicked");
        reduced_preds.extend(r);
        structural_preds.extend(s);
        ensemble_preds.extend(e);
    }

    let ci = |vals: &mut Vec<f64>| ConfidenceInterval {
        lower: percentile(vals, 0.05),
        median: percentile(vals, 0.50),
        upper: percentile(vals, 0.95),
    };

    BootstrapUncertainty {
        reduced: ci(&mut reduced_preds),
        structural: ci(&mut structural_preds),
        ensemble: ci(&mut ensemble_preds),
    }
}

/// Apply LOOCV-optimized ensemble weights to a ship's saved design.
pub fn apply_ensemble_calibration(ship: &mut ShipModel) {
    let ens = fit_ensemble_cv();
    ship.engine.ensemble_reduced_weight = ens.reduced_weight;
    ship.engine.ensemble_structural_weight = ens.structural_weight;
}

/// Apply Ridge-regularized Fn-segmented structural multipliers, using the
/// LOOCV-selected lambda.
pub fn apply_structural_calibration_ridge(ship: &mut ShipModel) {
    let selection = select_ridge_lambda();
    let segments = fit_structural_form_segmented_ridge(selection.best_lambda);
    let fn_ship = froude_number(ship.engine.speed_max, ship.hull.length);
    let seg = segments
        .iter()
        .find(|s| fn_ship > s.min_froude && fn_ship <= s.max_froude)
        .or_else(|| segments.first());
    if let Some(seg) = seg {
        ship.engine.resistance_model = ResistanceModel::HoltropMennen;
        ship.engine.blend_weight = 1.0;
        ship.engine.holtrop_viscous_factor = seg.viscous_factor;
        ship.engine.holtrop_wave_factor = seg.wave_factor;
        ship.engine.holtrop_correlation_factor = seg.correlation_factor;
    }
}

/// Apply Lasso-regularized Fn-segmented structural multipliers, using the
/// LOOCV-selected lambda.
pub fn apply_structural_calibration_lasso(ship: &mut ShipModel) {
    let selection = select_lasso_lambda();
    let segments = fit_structural_form_segmented_lasso(selection.best_lambda);
    let fn_ship = froude_number(ship.engine.speed_max, ship.hull.length);
    let seg = segments
        .iter()
        .find(|s| fn_ship > s.min_froude && fn_ship <= s.max_froude)
        .or_else(|| segments.first());
    if let Some(seg) = seg {
        ship.engine.resistance_model = ResistanceModel::HoltropMennen;
        ship.engine.blend_weight = 1.0;
        ship.engine.holtrop_viscous_factor = seg.viscous_factor;
        ship.engine.holtrop_wave_factor = seg.wave_factor;
        ship.engine.holtrop_correlation_factor = seg.correlation_factor;
    }
}

/// Confidence-weighted reduced-form fit.
///
/// Samples with higher historical-data confidence receive more weight in the
/// least-squares objective.
pub fn fit_reduced_form_weighted() -> ReducedFormFit {
    let cases = historical_cases();
    let mut a: Vec<Vec<f64>> = Vec::new();
    let mut b: Vec<f64> = Vec::new();
    for case in cases.iter() {
        if case.reference_hp <= 0.0 {
            continue;
        }
        let w = case_confidence(case.name).sqrt();
        a.push(vec![
            w,
            w * case.displacement_tons.ln(),
            w * case.speed_kt.ln(),
            w * case.length_ft.ln(),
        ]);
        b.push(w * case.reference_hp.ln());
    }
    let coef = solve_qr(&a, &b).unwrap_or_default();
    let fit = ReducedFormFit {
        intercept: coef[0],
        log_displacement: coef[1],
        log_speed: coef[2],
        log_length: coef[3],
        mean_abs_pct_error: 0.0,
        sigma_abs_pct_error: 0.0,
    };
    let mut errs = Vec::new();
    for case in cases.iter() {
        let pred = fit.predict(case.displacement_tons, case.speed_kt, case.length_ft);
        errs.push((pred - case.reference_hp) / case.reference_hp);
    }
    let (mean, sd) = mean_sd(&errs);
    ReducedFormFit {
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
        ..fit
    }
}

/// Confidence-weighted structural-form fit.
pub fn fit_structural_form_weighted() -> StructuralFormFit {
    let cases = historical_cases();
    let mut rows = Vec::new();
    let mut targets = Vec::new();
    for case in cases.iter() {
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let weight = case_confidence(case.name).sqrt();
        rows.push(vec![weight * f, weight * w]);
        targets.push(weight * (case.reference_hp * 0.7456999 - c));
    }
    let coef = solve_qr(&rows, &targets).unwrap_or_else(|| vec![1.0, 1.0]);
    let mut errs = Vec::new();
    for case in cases.iter() {
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let pred = coef[0] * f + coef[1] * w + c;
        let target = case.reference_hp * 0.7456999;
        if target > 0.0 {
            errs.push((pred - target) / target);
        }
    }
    let (mean, sd) = mean_sd(&errs);
    StructuralFormFit {
        viscous_factor: coef[0],
        wave_factor: coef[1],
        correlation_factor: 1.0,
        mean_abs_pct_error: mean.abs(),
        sigma_abs_pct_error: sd,
    }
}

/// Confidence-weighted bootstrap prediction intervals.
pub fn bootstrap_uncertainty_for_ship_weighted(
    ship: &crate::model::ShipModel,
    n_boot: usize,
) -> BootstrapUncertainty {
    let cases = historical_cases();
    let speed = ship.engine.speed_max;
    let disp = ship.hull.displacement;
    let length = ship.hull.length;
    let input = input_from_hull(&ship.hull, speed);
    let res = holtrop_resistance(&input);
    let v = speed * 0.514444;
    let fric_kw = (res.friction_n + res.appendage_n) * v / 1000.0;
    let wave_kw = (res.wave_n + res.bulb_n + res.transom_n) * v / 1000.0;
    let corr_kw = res.correlation_n * v / 1000.0;
    let loocv = run_loocv_weighted();
    let w_red = loocv.optimal_reduced_weight;

    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8);
    let chunk_size = n_boot.div_ceil(n_threads);
    let mut handles = Vec::new();
    let cases_owned = cases.clone();
    for t in 0..n_threads {
        let start = t * chunk_size;
        let end = ((t + 1) * chunk_size).min(n_boot);
        if start >= end {
            continue;
        }
        let cases_ref = cases_owned.clone();
        handles.push(std::thread::spawn(move || {
            let mut rng = 12345_u64.wrapping_add((t as u64).wrapping_mul(0x9E3779B97F4A7C15));
            let mut reduced = Vec::new();
            let mut structural = Vec::new();
            let mut ensemble = Vec::new();
            for _ in start..end {
                let mut sample = Vec::with_capacity(cases_ref.len());
                for _ in 0..cases_ref.len() {
                    rng = rng
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let idx = (rng >> 33) as usize % cases_ref.len();
                    sample.push(cases_ref[idx]);
                }
                let rf = reduced_form_fit_from_cases_weighted(&sample);
                let sf = structural_form_fit_from_cases_weighted(&sample);
                let rp = rf.predict(disp, speed, length);
                let sp = sf.predict_hp(fric_kw, wave_kw, corr_kw);
                reduced.push(rp);
                structural.push(sp);
                ensemble.push(w_red * rp + (1.0 - w_red) * sp);
            }
            (reduced, structural, ensemble)
        }));
    }

    let mut reduced_preds = Vec::with_capacity(n_boot);
    let mut structural_preds = Vec::with_capacity(n_boot);
    let mut ensemble_preds = Vec::with_capacity(n_boot);
    for handle in handles {
        let (r, s, e) = handle.join().expect("weighted bootstrap thread panicked");
        reduced_preds.extend(r);
        structural_preds.extend(s);
        ensemble_preds.extend(e);
    }

    let ci = |vals: &mut Vec<f64>| ConfidenceInterval {
        lower: percentile(vals, 0.05),
        median: percentile(vals, 0.50),
        upper: percentile(vals, 0.95),
    };

    BootstrapUncertainty {
        reduced: ci(&mut reduced_preds),
        structural: ci(&mut structural_preds),
        ensemble: ci(&mut ensemble_preds),
    }
}

/// Confidence-weighted Fn-segmented structural multipliers.
pub fn fit_structural_form_segmented_weighted() -> Vec<StructuralSegment> {
    let bands: [(&'static str, f64, f64); 3] = [
        ("low Fn (<=0.18)", f64::NEG_INFINITY, 0.18),
        ("medium Fn (0.18-0.28)", 0.18, 0.28),
        ("high Fn (>0.28)", 0.28, f64::INFINITY),
    ];
    let all_cases = historical_cases();
    bands
        .iter()
        .map(|(name, lo, hi)| {
            let subset: Vec<_> = all_cases
                .iter()
                .filter(|c| {
                    let fn_ = froude_number(c.speed_kt, c.length_ft);
                    fn_ > *lo && fn_ <= *hi
                })
                .collect();
            if subset.is_empty() {
                return StructuralSegment {
                    name,
                    min_froude: *lo,
                    max_froude: *hi,
                    viscous_factor: 1.0,
                    wave_factor: 1.0,
                    correlation_factor: 1.0,
                    mean_abs_pct_error: f64::INFINITY,
                    sigma_abs_pct_error: f64::INFINITY,
                };
            }
            let mut rows = Vec::new();
            let mut targets = Vec::new();
            for case in &subset {
                let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
                let input = input_from_hull(&ship.hull, case.speed_kt);
                let res = holtrop_resistance(&input);
                let (f, w, c) = structural_kw(case, &res);
                let weight = case_confidence(case.name).sqrt();
                rows.push(vec![weight * f, weight * w]);
                targets.push(weight * (case.reference_hp * 0.7456999 - c));
            }
            let coef = solve_qr(&rows, &targets).unwrap_or_else(|| vec![1.0, 1.0]);
            let mut errs = Vec::new();
            for (i, case) in subset.iter().enumerate() {
                let target = case.reference_hp * 0.7456999;
                let pred = coef[0] * rows[i][0] + coef[1] * rows[i][1] + (target - targets[i]);
                if target > 0.0 {
                    errs.push((pred - target) / target);
                }
            }
            let (mean, sd) = mean_sd(&errs);
            StructuralSegment {
                name,
                min_froude: *lo,
                max_froude: *hi,
                viscous_factor: coef[0],
                wave_factor: coef[1],
                correlation_factor: 1.0,
                mean_abs_pct_error: mean.abs(),
                sigma_abs_pct_error: sd,
            }
        })
        .collect()
}

/// Convert an external CSV case into a built-in `HistoricalCase`.
///
/// Note: this intentionally leaks the name/type strings so the calibration
/// functions can use `&'static str`.  It is suitable for one-shot CLI imports
/// and small datasets; a production database should use owned data instead.
fn external_to_historical(case: &ExternalCase) -> crate::accuracy::HistoricalCase {
    fn leak(s: &str) -> &'static str {
        Box::leak(s.to_string().into_boxed_str())
    }
    crate::accuracy::HistoricalCase {
        name: leak(&case.name),
        ship_type: leak(&case.ship_type),
        length_ft: case.length_ft,
        beam_ft: case.beam_ft,
        draught_ft: case.draught_ft,
        displacement_tons: case.displacement_tons,
        block_coefficient: case.block_coefficient,
        speed_kt: case.speed_kt,
        reference_hp: case.reference_hp,
        provenance: crate::accuracy::Provenance {
            // External CSV data is treated as public-reference unless it is
            // later upgraded through explicit provenance metadata.
            source_type: crate::accuracy::SourceType::PublicReference,
            displacement_type: crate::accuracy::DisplacementType::Unspecified,
            confidence: case.confidence,
            source: leak(&case.source),
            source_url: "",
            verifier: "",
            verification_date: "unknown",
        },
    }
}

/// Fit reduced form using built-in data plus user-supplied external cases.
pub fn fit_reduced_form_with_external(external: &[ExternalCase]) -> ReducedFormFit {
    let mut cases = historical_cases();
    cases.extend(external.iter().map(external_to_historical));
    reduced_form_fit_from_cases(&cases)
}

/// Fit structural form using built-in data plus user-supplied external cases.
pub fn fit_structural_form_with_external(external: &[ExternalCase]) -> StructuralFormFit {
    let mut cases = historical_cases();
    cases.extend(external.iter().map(external_to_historical));
    structural_form_fit_from_cases(&cases)
}

/// Return built-in historical cases plus external cases converted to the
/// calibration format.
fn combined_cases(external: &[ExternalCase]) -> Vec<crate::accuracy::HistoricalCase> {
    let mut cases = historical_cases();
    cases.extend(external.iter().map(external_to_historical));
    cases
}

/// Leave-one-out cross-validation using built-in + external data.
pub fn run_loocv_with_external(external: &[ExternalCase]) -> LoocvReport {
    let cases = combined_cases(external);
    let mut folds = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let train: Vec<_> = cases
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, c)| *c)
            .collect();
        let rf = reduced_form_fit_from_cases(&train);
        let sf = structural_form_fit_from_cases(&train);
        let reduced_hp = rf.predict(case.displacement_tons, case.speed_kt, case.length_ft);
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let structural_hp = sf.predict_hp(f, w, c);
        folds.push(LoocvFold {
            name: case.name,
            reference_hp: case.reference_hp,
            reduced_hp,
            structural_hp,
        });
    }
    let n = folds.len() as f64;
    let reduced_mape = folds
        .iter()
        .map(|f| ((f.reduced_hp - f.reference_hp) / f.reference_hp).abs())
        .sum::<f64>()
        / n;
    let structural_mape = folds
        .iter()
        .map(|f| ((f.structural_hp - f.reference_hp) / f.reference_hp).abs())
        .sum::<f64>()
        / n;
    let mut best_w = 0.5_f64;
    let mut best_mape = f64::INFINITY;
    for k in 0..=100 {
        let w = k as f64 / 100.0;
        let mape = folds
            .iter()
            .map(|f| {
                let pred = w * f.reduced_hp + (1.0 - w) * f.structural_hp;
                ((pred - f.reference_hp) / f.reference_hp).abs()
            })
            .sum::<f64>()
            / n;
        if mape < best_mape {
            best_mape = mape;
            best_w = w;
        }
    }
    LoocvReport {
        reduced_mape,
        structural_mape,
        ensemble_mape: best_mape,
        optimal_reduced_weight: best_w,
        folds,
    }
}

/// Bootstrap uncertainty using built-in + external data.
pub fn bootstrap_uncertainty_with_external(
    external: &[ExternalCase],
    ship: &crate::model::ShipModel,
    n_boot: usize,
) -> BootstrapUncertainty {
    let cases = combined_cases(external);
    let speed = ship.engine.speed_max;
    let disp = ship.hull.displacement;
    let length = ship.hull.length;
    let input = input_from_hull(&ship.hull, speed);
    let res = holtrop_resistance(&input);
    let v = speed * 0.514444;
    let fric_kw = (res.friction_n + res.appendage_n) * v / 1000.0;
    let wave_kw = (res.wave_n + res.bulb_n + res.transom_n) * v / 1000.0;
    let corr_kw = res.correlation_n * v / 1000.0;
    let loocv = run_loocv_with_external(external);
    let w_red = loocv.optimal_reduced_weight;

    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8);
    let chunk_size = n_boot.div_ceil(n_threads);
    let mut handles = Vec::new();
    let cases_owned = cases.clone();
    for t in 0..n_threads {
        let start = t * chunk_size;
        let end = ((t + 1) * chunk_size).min(n_boot);
        if start >= end {
            continue;
        }
        let cases_ref = cases_owned.clone();
        handles.push(std::thread::spawn(move || {
            let mut rng = 12345_u64.wrapping_add((t as u64).wrapping_mul(0x9E3779B97F4A7C15));
            let mut reduced = Vec::new();
            let mut structural = Vec::new();
            let mut ensemble = Vec::new();
            for _ in start..end {
                let mut sample = Vec::with_capacity(cases_ref.len());
                for _ in 0..cases_ref.len() {
                    rng = rng
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let idx = (rng >> 33) as usize % cases_ref.len();
                    sample.push(cases_ref[idx]);
                }
                let rf = reduced_form_fit_from_cases(&sample);
                let sf = structural_form_fit_from_cases(&sample);
                let rp = rf.predict(disp, speed, length);
                let sp = sf.predict_hp(fric_kw, wave_kw, corr_kw);
                reduced.push(rp);
                structural.push(sp);
                ensemble.push(w_red * rp + (1.0 - w_red) * sp);
            }
            (reduced, structural, ensemble)
        }));
    }

    let mut reduced_preds = Vec::with_capacity(n_boot);
    let mut structural_preds = Vec::with_capacity(n_boot);
    let mut ensemble_preds = Vec::with_capacity(n_boot);
    for handle in handles {
        let (r, s, e) = handle.join().expect("external bootstrap thread panicked");
        reduced_preds.extend(r);
        structural_preds.extend(s);
        ensemble_preds.extend(e);
    }

    let ci = |vals: &mut Vec<f64>| ConfidenceInterval {
        lower: percentile(vals, 0.05),
        median: percentile(vals, 0.50),
        upper: percentile(vals, 0.95),
    };

    BootstrapUncertainty {
        reduced: ci(&mut reduced_preds),
        structural: ci(&mut structural_preds),
        ensemble: ci(&mut ensemble_preds),
    }
}

/// Confidence-weighted LOOCV using built-in + external data.
pub fn run_loocv_with_external_weighted(external: &[ExternalCase]) -> LoocvReport {
    let cases = combined_cases(external);
    let mut folds = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let train: Vec<_> = cases
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, c)| *c)
            .collect();
        let rf = reduced_form_fit_from_cases_weighted(&train);
        let sf = structural_form_fit_from_cases_weighted(&train);
        let reduced_hp = rf.predict(case.displacement_tons, case.speed_kt, case.length_ft);
        let ship = simulate_case_with_model(case, ResistanceModel::HoltropMennen);
        let input = input_from_hull(&ship.hull, case.speed_kt);
        let res = holtrop_resistance(&input);
        let (f, w, c) = structural_kw(case, &res);
        let structural_hp = sf.predict_hp(f, w, c);
        folds.push(LoocvFold {
            name: case.name,
            reference_hp: case.reference_hp,
            reduced_hp,
            structural_hp,
        });
    }
    let n = folds.len() as f64;
    let reduced_mape = folds
        .iter()
        .map(|f| ((f.reduced_hp - f.reference_hp) / f.reference_hp).abs())
        .sum::<f64>()
        / n;
    let structural_mape = folds
        .iter()
        .map(|f| ((f.structural_hp - f.reference_hp) / f.reference_hp).abs())
        .sum::<f64>()
        / n;
    let mut best_w = 0.5_f64;
    let mut best_mape = f64::INFINITY;
    for k in 0..=100 {
        let w = k as f64 / 100.0;
        let mape = folds
            .iter()
            .map(|f| {
                let pred = w * f.reduced_hp + (1.0 - w) * f.structural_hp;
                ((pred - f.reference_hp) / f.reference_hp).abs()
            })
            .sum::<f64>()
            / n;
        if mape < best_mape {
            best_mape = mape;
            best_w = w;
        }
    }
    LoocvReport {
        reduced_mape,
        structural_mape,
        ensemble_mape: best_mape,
        optimal_reduced_weight: best_w,
        folds,
    }
}

/// Confidence-weighted Bootstrap using built-in + external data.
pub fn bootstrap_uncertainty_with_external_weighted(
    external: &[ExternalCase],
    ship: &crate::model::ShipModel,
    n_boot: usize,
) -> BootstrapUncertainty {
    let cases = combined_cases(external);
    let speed = ship.engine.speed_max;
    let disp = ship.hull.displacement;
    let length = ship.hull.length;
    let input = input_from_hull(&ship.hull, speed);
    let res = holtrop_resistance(&input);
    let v = speed * 0.514444;
    let fric_kw = (res.friction_n + res.appendage_n) * v / 1000.0;
    let wave_kw = (res.wave_n + res.bulb_n + res.transom_n) * v / 1000.0;
    let corr_kw = res.correlation_n * v / 1000.0;
    let loocv = run_loocv_with_external_weighted(external);
    let w_red = loocv.optimal_reduced_weight;

    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8);
    let chunk_size = n_boot.div_ceil(n_threads);
    let mut handles = Vec::new();
    let cases_owned = cases.clone();
    for t in 0..n_threads {
        let start = t * chunk_size;
        let end = ((t + 1) * chunk_size).min(n_boot);
        if start >= end {
            continue;
        }
        let cases_ref = cases_owned.clone();
        handles.push(std::thread::spawn(move || {
            let mut rng = 12345_u64.wrapping_add((t as u64).wrapping_mul(0x9E3779B97F4A7C15));
            let mut reduced = Vec::new();
            let mut structural = Vec::new();
            let mut ensemble = Vec::new();
            for _ in start..end {
                let mut sample = Vec::with_capacity(cases_ref.len());
                for _ in 0..cases_ref.len() {
                    rng = rng
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let idx = (rng >> 33) as usize % cases_ref.len();
                    sample.push(cases_ref[idx]);
                }
                let rf = reduced_form_fit_from_cases_weighted(&sample);
                let sf = structural_form_fit_from_cases_weighted(&sample);
                let rp = rf.predict(disp, speed, length);
                let sp = sf.predict_hp(fric_kw, wave_kw, corr_kw);
                reduced.push(rp);
                structural.push(sp);
                ensemble.push(w_red * rp + (1.0 - w_red) * sp);
            }
            (reduced, structural, ensemble)
        }));
    }

    let mut reduced_preds = Vec::with_capacity(n_boot);
    let mut structural_preds = Vec::with_capacity(n_boot);
    let mut ensemble_preds = Vec::with_capacity(n_boot);
    for handle in handles {
        let (r, s, e) = handle
            .join()
            .expect("external weighted bootstrap thread panicked");
        reduced_preds.extend(r);
        structural_preds.extend(s);
        ensemble_preds.extend(e);
    }

    let ci = |vals: &mut Vec<f64>| ConfidenceInterval {
        lower: percentile(vals, 0.05),
        median: percentile(vals, 0.50),
        upper: percentile(vals, 0.95),
    };

    BootstrapUncertainty {
        reduced: ci(&mut reduced_preds),
        structural: ci(&mut structural_preds),
        ensemble: ci(&mut ensemble_preds),
    }
}

/// Apply external-data LOOCV-optimized ensemble weights to a design.
pub fn apply_ensemble_calibration_with_external(ship: &mut ShipModel, external: &[ExternalCase]) {
    let report = run_loocv_with_external_weighted(external);
    ship.engine.ensemble_reduced_weight = report.optimal_reduced_weight;
    ship.engine.ensemble_structural_weight = 1.0 - report.optimal_reduced_weight;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_form_fit_produces_finite_coefficients() {
        let fit = fit_reduced_form();
        assert!(fit.intercept.is_finite());
        assert!(fit.mean_abs_pct_error.is_finite());
        assert!(fit.sigma_abs_pct_error.is_finite());
    }

    #[test]
    fn small_fast_external_merge_is_usable_for_calibration() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nPT Boat,motor,80,20,4,50,0.42,40,1500,0.65,approx\nCorvette,patrol,205,33,11,925,0.55,16,2750,0.70,approx\n";
        let cases = crate::accuracy::parse_external_cases(csv).unwrap();
        let rf = fit_reduced_form_with_external(&cases);
        assert!(rf.mean_abs_pct_error.is_finite());
        let loocv = run_loocv_with_external_weighted(&cases);
        assert!(loocv.ensemble_mape.is_finite());
    }

    #[test]
    fn external_data_without_external_cases_uses_builtin_only() {
        let rf = fit_reduced_form_with_external(&[]);
        let sf = fit_structural_form_with_external(&[]);
        assert!(rf.mean_abs_pct_error.is_finite());
        assert!(sf.mean_abs_pct_error.is_finite());
    }

    #[test]
    fn structural_form_fit_produces_finite_coefficients() {
        let fit = fit_structural_form();
        assert!(fit.viscous_factor.is_finite());
        assert!(fit.mean_abs_pct_error.is_finite());
        assert!(fit.sigma_abs_pct_error.is_finite());
    }

    #[test]
    fn type_dummy_reduced_form_runs() {
        let fit = fit_reduced_form_by_type();
        assert!(fit.mean_abs_pct_error.is_finite());
    }

    #[test]
    fn external_data_merge_is_finite() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nTest,battleship,600,90,28,23760,0.55,24,50000,0.9,example\n";
        let cases = crate::accuracy::parse_external_cases(csv).unwrap();
        let rf = fit_reduced_form_with_external(&cases);
        let sf = fit_structural_form_with_external(&cases);
        assert!(rf.mean_abs_pct_error.is_finite());
        assert!(sf.mean_abs_pct_error.is_finite());
        let loocv = run_loocv_with_external_weighted(&cases);
        assert!(loocv.ensemble_mape.is_finite());
    }

    #[test]
    fn regularization_lambda_changes_coefficients() {
        let cases = historical_cases();
        let weak = structural_form_fit_from_cases_ridge(&cases, 1e-4);
        let strong = structural_form_fit_from_cases_ridge(&cases, 1e4);
        assert!(weak.mean_abs_pct_error.is_finite());
        assert!(strong.mean_abs_pct_error.is_finite());
        let weak_scale = weak.viscous_factor.abs().max(1.0);
        assert!(
            (weak.viscous_factor - strong.viscous_factor).abs() > 1e-3 * weak_scale,
            "ridge lambda grid is a no-op: weak={} strong={}",
            weak.viscous_factor,
            strong.viscous_factor
        );
        let weak_l = structural_form_fit_from_cases_lasso(&cases, 1e-4);
        let strong_l = structural_form_fit_from_cases_lasso(&cases, 1e4);
        assert!(
            (weak_l.viscous_factor - strong_l.viscous_factor).abs()
                > 1e-3 * weak_l.viscous_factor.abs().max(1.0),
            "lasso lambda grid is a no-op: weak={} strong={}",
            weak_l.viscous_factor,
            strong_l.viscous_factor
        );
        // The selected lambda must not be a tie: the grid has to actually vary.
        let ridge_sel = select_ridge_lambda();
        let distinct: std::collections::BTreeSet<i64> = ridge_sel
            .results
            .iter()
            .map(|(_, m)| (m * 1e6).round() as i64)
            .collect();
        assert!(
            distinct.len() > 1,
            "ridge lambda selection is degenerate: {:?}",
            ridge_sel.results
        );
    }

    #[test]
    fn loocv_by_type_covers_all_types_out_of_sample() {
        let global = run_loocv();
        let by_type = loocv_by_type();
        assert!(!by_type.is_empty());
        let total: usize = by_type.iter().map(|(_, n, _)| *n).sum();
        assert_eq!(total, historical_cases().len());
        for (ty, n, mape) in &by_type {
            assert!(*n >= 2, "{ty} has too few cases");
            assert!(mape.is_finite() && *mape >= 0.0);
        }
        // The ensemble weight used is the global LOOCV optimum.
        assert!(global.optimal_reduced_weight >= 0.0 && global.optimal_reduced_weight <= 1.0);
    }

    #[test]
    fn lambda_selection_and_bootstrap_are_finite() {
        let ridge = select_ridge_lambda();
        assert!(ridge.best_lambda.is_finite());
        assert!(ridge.best_loocv_mape.is_finite());
        let lasso = select_lasso_lambda();
        assert!(lasso.best_lambda.is_finite());
        let mut ship = crate::model::ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.displacement = 23760.0;
        ship.engine.speed_max = 24.0;
        let boot = bootstrap_uncertainty_for_ship(&ship, 30);
        assert!(boot.ensemble.lower.is_finite());
        assert!(boot.ensemble.upper.is_finite());
    }

    #[test]
    fn interaction_and_loocv_and_regularized_are_finite() {
        let inter = fit_reduced_form_interaction();
        assert!(inter.mean_abs_pct_error.is_finite());
        let loocv = run_loocv();
        assert!(loocv.ensemble_mape.is_finite());
        assert!(loocv.optimal_reduced_weight >= 0.0);
        let (_, _, diff) = compare_qr_svd();
        assert!(diff.is_finite() && diff >= 0.0);
        for seg in fit_structural_form_segmented_ridge(0.01) {
            if seg.mean_abs_pct_error.is_finite() {
                assert!(seg.viscous_factor.is_finite());
            }
        }
        for seg in fit_structural_form_segmented_lasso(0.01) {
            if seg.mean_abs_pct_error.is_finite() {
                assert!(seg.viscous_factor.is_finite());
            }
        }
    }

    #[test]
    fn structural_segments_and_ensemble_are_finite() {
        for seg in fit_structural_form_segmented() {
            if seg.mean_abs_pct_error.is_finite() {
                assert!(seg.viscous_factor.is_finite());
                assert!(seg.wave_factor.is_finite());
            }
        }
        let ens = fit_ensemble();
        assert!(ens.mean_abs_pct_error.is_finite());
        assert!(ens.reduced_weight + ens.structural_weight - 1.0 < 1e-9);
    }
}
