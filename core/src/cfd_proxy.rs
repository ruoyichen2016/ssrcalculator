//! CFD surrogate residual-correction framework.
//!
//! Full CFD is expensive, so this module provides a small, replaceable
//! correction layer.  A future CFD surrogate can predict the residual
//! `(CFD_power / empirical_power) - 1` as a function of Froude number and hull
//! parameters; for now the correction is a simple Froude-number polynomial.

/// Configuration for the optional CFD-surrogate correction.
#[derive(Clone, Copy, Debug, Default)]
pub struct CfdProxyConfig {
    pub enabled: bool,
    /// Linear Froude-number correction coefficient.
    pub viscous_correction: f64,
    /// Quadratic Froude-number correction coefficient.
    pub wave_correction: f64,
    /// Reference Froude number where the correction is zero.
    pub fn_ref: f64,
}

impl CfdProxyConfig {
    /// Compute the multiplicative residual correction factor.
    pub fn correction_factor(&self, fn_: f64) -> f64 {
        if !self.enabled {
            return 1.0;
        }
        let d = fn_ - self.fn_ref;
        1.0 + self.viscous_correction * d + self.wave_correction * d * d
    }

    /// Apply the surrogate correction to a shaft-power estimate.
    pub fn apply(&self, hp: f64, fn_: f64) -> f64 {
        hp * self.correction_factor(fn_)
    }
}

/// Froude number for a waterline length in feet and a speed in knots.
pub fn cfd_froude_number(speed_kt: f64, length_ft: f64) -> f64 {
    let v = speed_kt * 0.514444;
    let l = length_ft * 0.3048;
    v / (9.80665 * l).sqrt()
}

/// Where a CFD surrogate sample came from.  Only measured CFD (or an official
/// model-test surrogate) counts as evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CfdProvenance {
    MeasuredCfd,
    Literature,
    Analytic,
    #[default]
    Synthetic,
}

impl CfdProvenance {
    pub const fn label(self) -> &'static str {
        match self {
            CfdProvenance::MeasuredCfd => "measured_cfd",
            CfdProvenance::Literature => "literature",
            CfdProvenance::Analytic => "analytic",
            CfdProvenance::Synthetic => "synthetic",
        }
    }

    pub const fn is_measured_cfd(self) -> bool {
        matches!(self, CfdProvenance::MeasuredCfd)
    }

    pub fn parse(token: &str) -> Result<Self, String> {
        match token.trim().to_ascii_lowercase().as_str() {
            "measured_cfd" | "cfd" | "measured" => Ok(CfdProvenance::MeasuredCfd),
            "literature" | "model_test" | "modeltest" => Ok(CfdProvenance::Literature),
            "analytic" | "analysis" => Ok(CfdProvenance::Analytic),
            "synthetic" | "placeholder" | "synthetic_placeholder" => Ok(CfdProvenance::Synthetic),
            other => Err(format!("unknown CFD provenance '{other}'")),
        }
    }
}

/// One CFD/empirical comparison sample.
#[derive(Clone, Copy, Debug)]
pub struct CfdSample {
    pub fn_: f64,
    pub empirical_hp: f64,
    pub cfd_hp: f64,
    pub provenance: CfdProvenance,
}

/// Parse a simple CSV of CFD samples.
///
/// Columns: `fn,empirical_hp,cfd_hp[,provenance]`.  A 3-column row is treated
/// as `synthetic` (conservative).  Provenance tokens: `measured_cfd`,
/// `literature`, `analytic`, `synthetic`.
pub fn parse_cfd_samples(text: &str) -> Result<Vec<CfdSample>, String> {
    let mut out = Vec::new();
    for (line_no, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("fn,") {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(str::trim).collect();
        if cols.len() != 3 && cols.len() != 4 {
            return Err(format!(
                "line {}: expected 3 or 4 columns, got {}",
                line_no + 1,
                cols.len()
            ));
        }
        let parse = |i: usize, label: &str| -> Result<f64, String> {
            cols[i]
                .parse::<f64>()
                .map_err(|e| format!("line {}: {label} invalid: {e}", line_no + 1))
        };
        let provenance = if cols.len() == 4 {
            CfdProvenance::parse(cols[3]).map_err(|e| format!("line {}: {e}", line_no + 1))?
        } else {
            CfdProvenance::Synthetic
        };
        out.push(CfdSample {
            fn_: parse(0, "fn")?,
            empirical_hp: parse(1, "empirical_hp")?,
            cfd_hp: parse(2, "cfd_hp")?,
            provenance,
        });
    }
    Ok(out)
}

/// Fit the residual correction `ratio - 1 = b*d + c*d^2`, where
/// `d = Fn - mean(Fn)`.
pub fn fit_cfd_proxy(samples: &[CfdSample]) -> CfdProxyConfig {
    let valid: Vec<CfdSample> = samples
        .iter()
        .copied()
        .filter(|s| s.fn_.is_finite() && s.empirical_hp > 0.0 && s.cfd_hp > 0.0)
        .collect();
    if valid.len() < 2 {
        return CfdProxyConfig::default();
    }
    let mean_fn: f64 = valid.iter().map(|s| s.fn_).sum::<f64>() / valid.len() as f64;
    let mut sxx = 0.0_f64;
    let mut sxxx = 0.0_f64;
    let mut sxxxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    let mut sxxy = 0.0_f64;
    for s in &valid {
        let d = s.fn_ - mean_fn;
        let y = s.cfd_hp / s.empirical_hp - 1.0;
        sxx += d * d;
        sxxx += d * d * d;
        sxxxx += d * d * d * d;
        sxy += d * y;
        sxxy += d * d * y;
    }
    let det = sxx * sxxxx - sxxx * sxxx;
    if det.abs() < 1e-18 {
        return CfdProxyConfig {
            enabled: true,
            viscous_correction: 0.0,
            wave_correction: 0.0,
            fn_ref: mean_fn,
        };
    }
    let b = (sxy * sxxxx - sxxy * sxxx) / det;
    let c = (sxx * sxxy - sxxx * sxy) / det;
    CfdProxyConfig {
        enabled: true,
        viscous_correction: b,
        wave_correction: c,
        fn_ref: mean_fn,
    }
}

/// Minimum number of independent CFD samples before a fitted surrogate is
/// treated as usable evidence.  The bundled example dataset has far fewer and
/// is synthetic, so the surrogate is a pipeline placeholder.
pub const CFD_MIN_USABLE_SAMPLES: usize = 20;

#[derive(Clone, Debug)]
pub struct CfdStatus {
    pub samples: usize,
    pub measured_cfd: usize,
    pub literature: usize,
    pub analytic: usize,
    pub synthetic: usize,
    pub min_samples: usize,
    /// True only when there are enough *measured* CFD samples.
    pub usable_for_evidence: bool,
    pub message: String,
}

pub fn cfd_status(samples: &[CfdSample]) -> CfdStatus {
    let mut measured = 0;
    let mut literature = 0;
    let mut analytic = 0;
    let mut synthetic = 0;
    for s in samples {
        match s.provenance {
            CfdProvenance::MeasuredCfd => measured += 1,
            CfdProvenance::Literature => literature += 1,
            CfdProvenance::Analytic => analytic += 1,
            CfdProvenance::Synthetic => synthetic += 1,
        }
    }
    let usable = measured >= CFD_MIN_USABLE_SAMPLES;
    let message = if usable {
        format!(
            "{measured} measured CFD samples (>= {} minimum): surrogate may be used as evidence.",
            CFD_MIN_USABLE_SAMPLES
        )
    } else {
        format!(
            "{measured} measured CFD sample(s) (< {} minimum): NOT evidence. \
Other rows: literature={literature}, analytic={analytic}, synthetic={synthetic}. \
Add real CFD data with provenance `measured_cfd` to enable `cfd-apply`.",
            CFD_MIN_USABLE_SAMPLES
        )
    };
    CfdStatus {
        samples: samples.len(),
        measured_cfd: measured,
        literature,
        analytic,
        synthetic,
        min_samples: CFD_MIN_USABLE_SAMPLES,
        usable_for_evidence: usable,
        message,
    }
}

/// Fit-quality report for the CFD proxy.
#[derive(Clone, Copy, Debug)]
pub struct CfdFitReport {
    pub config: CfdProxyConfig,
    pub r2: f64,
    pub rmse: f64,
    pub samples: usize,
}

/// Fit the proxy and return R² / RMSE statistics.
pub fn fit_cfd_proxy_report(samples: &[CfdSample]) -> CfdFitReport {
    let config = fit_cfd_proxy(samples);
    let valid: Vec<CfdSample> = samples
        .iter()
        .copied()
        .filter(|s| s.fn_.is_finite() && s.empirical_hp > 0.0 && s.cfd_hp > 0.0)
        .collect();
    if valid.is_empty() {
        return CfdFitReport {
            config,
            r2: f64::NAN,
            rmse: f64::NAN,
            samples: 0,
        };
    }
    let ys: Vec<f64> = valid
        .iter()
        .map(|s| s.cfd_hp / s.empirical_hp - 1.0)
        .collect();
    let mean_y = ys.iter().sum::<f64>() / ys.len() as f64;
    let ss_tot: f64 = ys.iter().map(|y| (y - mean_y).powi(2)).sum();
    let mut ss_res = 0.0_f64;
    for s in &valid {
        let y = s.cfd_hp / s.empirical_hp - 1.0;
        let pred = config.correction_factor(s.fn_) - 1.0;
        ss_res += (y - pred).powi(2);
    }
    let r2 = if ss_tot > 1e-18 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };
    let rmse = (ss_res / valid.len() as f64).sqrt();
    CfdFitReport {
        config,
        r2,
        rmse,
        samples: valid.len(),
    }
}

/// Leave-one-out cross-validation statistics for the CFD proxy.
#[derive(Clone, Copy, Debug)]
pub struct CfdCvReport {
    pub r2: f64,
    pub rmse: f64,
    pub samples: usize,
}

/// Leave-one-out cross-validation of the CFD surrogate correction.
pub fn cfd_loocv_report(samples: &[CfdSample]) -> CfdCvReport {
    let valid: Vec<CfdSample> = samples
        .iter()
        .copied()
        .filter(|s| s.fn_.is_finite() && s.empirical_hp > 0.0 && s.cfd_hp > 0.0)
        .collect();
    if valid.len() < 3 {
        return CfdCvReport {
            r2: f64::NAN,
            rmse: f64::NAN,
            samples: valid.len(),
        };
    }
    let mut preds = Vec::with_capacity(valid.len());
    let mut ys = Vec::with_capacity(valid.len());
    for (i, test) in valid.iter().enumerate() {
        let train: Vec<CfdSample> = valid
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, s)| *s)
            .collect();
        let config = fit_cfd_proxy(&train);
        preds.push(config.correction_factor(test.fn_) - 1.0);
        ys.push(test.cfd_hp / test.empirical_hp - 1.0);
    }
    let mean_y = ys.iter().sum::<f64>() / ys.len() as f64;
    let ss_tot: f64 = ys.iter().map(|y| (y - mean_y).powi(2)).sum();
    let ss_res: f64 = ys
        .iter()
        .zip(preds.iter())
        .map(|(y, p)| (y - p).powi(2))
        .sum();
    let r2 = if ss_tot > 1e-18 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };
    CfdCvReport {
        r2,
        rmse: (ss_res / valid.len() as f64).sqrt(),
        samples: valid.len(),
    }
}

/// Deterministic holdout validation for the CFD proxy.
///
/// `holdout_fraction` is the approximate fraction of samples placed in the
/// test set.  The split is deterministic so results are reproducible.
pub fn cfd_holdout_report(samples: &[CfdSample], holdout_fraction: f64) -> CfdCvReport {
    let valid: Vec<CfdSample> = samples
        .iter()
        .copied()
        .filter(|s| s.fn_.is_finite() && s.empirical_hp > 0.0 && s.cfd_hp > 0.0)
        .collect();
    if valid.len() < 4 || holdout_fraction <= 0.0 {
        return CfdCvReport {
            r2: f64::NAN,
            rmse: f64::NAN,
            samples: 0,
        };
    }
    let threshold = (holdout_fraction * 1000.0) as u64;
    let is_test = |i: usize| -> bool {
        let h = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        (h >> 33) % 1000 < threshold
    };
    let train: Vec<CfdSample> = valid
        .iter()
        .enumerate()
        .filter(|(i, _)| !is_test(*i))
        .map(|(_, s)| *s)
        .collect();
    let test: Vec<CfdSample> = valid
        .iter()
        .enumerate()
        .filter(|(i, _)| is_test(*i))
        .map(|(_, s)| *s)
        .collect();
    if train.len() < 2 || test.is_empty() {
        return CfdCvReport {
            r2: f64::NAN,
            rmse: f64::NAN,
            samples: test.len(),
        };
    }
    let config = fit_cfd_proxy(&train);
    let ys: Vec<f64> = test
        .iter()
        .map(|s| s.cfd_hp / s.empirical_hp - 1.0)
        .collect();
    let preds: Vec<f64> = test
        .iter()
        .map(|s| config.correction_factor(s.fn_) - 1.0)
        .collect();
    let mean_y = ys.iter().sum::<f64>() / ys.len() as f64;
    let ss_tot: f64 = ys.iter().map(|y| (y - mean_y).powi(2)).sum();
    let ss_res: f64 = ys
        .iter()
        .zip(preds.iter())
        .map(|(y, p)| (y - p).powi(2))
        .sum();
    let r2 = if ss_tot > 1e-18 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };
    CfdCvReport {
        r2,
        rmse: (ss_res / test.len() as f64).sqrt(),
        samples: test.len(),
    }
}

/// Ridge-regularized CFD proxy fit.
pub fn fit_cfd_proxy_ridge(samples: &[CfdSample], lambda: f64) -> CfdProxyConfig {
    let valid: Vec<CfdSample> = samples
        .iter()
        .copied()
        .filter(|s| s.fn_.is_finite() && s.empirical_hp > 0.0 && s.cfd_hp > 0.0)
        .collect();
    if valid.len() < 2 {
        return CfdProxyConfig::default();
    }
    let mean_fn: f64 = valid.iter().map(|s| s.fn_).sum::<f64>() / valid.len() as f64;
    let (mut sxx, mut sxxx, mut sxxxx, mut sxy, mut sxxy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for s in &valid {
        let d = s.fn_ - mean_fn;
        let y = s.cfd_hp / s.empirical_hp - 1.0;
        sxx += d * d;
        sxxx += d * d * d;
        sxxxx += d * d * d * d;
        sxy += d * y;
        sxxy += d * d * y;
    }
    let lam = lambda.max(0.0);
    let a11 = sxx + lam;
    let a22 = sxxxx + lam;
    let det = a11 * a22 - sxxx * sxxx;
    if det.abs() < 1e-18 {
        return CfdProxyConfig {
            enabled: true,
            viscous_correction: 0.0,
            wave_correction: 0.0,
            fn_ref: mean_fn,
        };
    }
    let b = (sxy * a22 - sxxy * sxxx) / det;
    let c = (a11 * sxxy - sxxx * sxy) / det;
    CfdProxyConfig {
        enabled: true,
        viscous_correction: b,
        wave_correction: c,
        fn_ref: mean_fn,
    }
}

/// Selected Ridge lambda and its LOOCV error.
#[derive(Clone, Debug)]
pub struct CfdLambdaSelection {
    pub best_lambda: f64,
    pub best_rmse: f64,
    pub results: Vec<(f64, f64)>,
}

/// Select the Ridge lambda for the CFD proxy by LOOCV.
pub fn select_cfd_ridge_lambda(samples: &[CfdSample]) -> CfdLambdaSelection {
    let candidates = [0.0, 0.01, 0.1, 1.0, 10.0, 100.0];
    let mut results = Vec::new();
    let mut best = (0.0_f64, f64::INFINITY);
    for lambda in candidates {
        let rmse = cfd_ridge_loocv_rmse(samples, lambda);
        results.push((lambda, rmse));
        if rmse < best.1 {
            best = (lambda, rmse);
        }
    }
    CfdLambdaSelection {
        best_lambda: best.0,
        best_rmse: best.1,
        results,
    }
}

fn cfd_ridge_loocv_rmse(samples: &[CfdSample], lambda: f64) -> f64 {
    let valid: Vec<CfdSample> = samples
        .iter()
        .copied()
        .filter(|s| s.fn_.is_finite() && s.empirical_hp > 0.0 && s.cfd_hp > 0.0)
        .collect();
    if valid.len() < 3 {
        return f64::INFINITY;
    }
    let mut ss = 0.0_f64;
    let mut n = 0_u32;
    for (i, test) in valid.iter().enumerate() {
        let train: Vec<CfdSample> = valid
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, s)| *s)
            .collect();
        let config = fit_cfd_proxy_ridge(&train, lambda);
        let y = test.cfd_hp / test.empirical_hp - 1.0;
        let p = config.correction_factor(test.fn_) - 1.0;
        ss += (y - p).powi(2);
        n += 1;
    }
    (ss / n as f64).sqrt()
}

/// Holdout stability across several holdout fractions.
pub fn cfd_holdout_stability(samples: &[CfdSample]) -> Vec<(f64, f64)> {
    [0.2, 0.3, 0.4, 0.5]
        .iter()
        .map(|f| (*f, cfd_holdout_report(samples, *f).r2))
        .collect()
}

/// Generate a simple HTML report for CFD proxy fitting.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lambda_selection_returns_valid_candidate() {
        let samples: Vec<CfdSample> = (0..9)
            .map(|i| {
                let fn_ = 0.20 + i as f64 * 0.02;
                let d = fn_ - 0.28;
                let noise = if i % 2 == 0 { 0.02 } else { -0.02 };
                let ratio = 1.0 + 0.10 * d + 0.05 * d * d + noise;
                CfdSample {
                    fn_,
                    empirical_hp: 1000.0,
                    cfd_hp: 1000.0 * ratio,
                    provenance: CfdProvenance::Synthetic,
                }
            })
            .collect();
        let selection = select_cfd_ridge_lambda(&samples);
        assert!(selection.best_lambda >= 0.0);
        assert!(selection.best_rmse.is_finite());
        assert_eq!(selection.results.len(), 6);
    }

    #[test]
    fn ridge_shrinks_coefficients() {
        let samples: Vec<CfdSample> = (0..9)
            .map(|i| {
                let fn_ = 0.20 + i as f64 * 0.02;
                let d = fn_ - 0.28;
                let noise = if i % 2 == 0 { 0.02 } else { -0.02 };
                let ratio = 1.0 + 0.10 * d + 0.05 * d * d + noise;
                CfdSample {
                    fn_,
                    empirical_hp: 1000.0,
                    cfd_hp: 1000.0 * ratio,
                    provenance: CfdProvenance::Synthetic,
                }
            })
            .collect();
        let plain = fit_cfd_proxy(&samples);
        let ridge = fit_cfd_proxy_ridge(&samples, 1.0);
        assert!(ridge.wave_correction.abs() <= plain.wave_correction.abs() + 1e-9);
    }

    #[test]
    fn holdout_report_is_finite() {
        let samples: Vec<CfdSample> = (0..12)
            .map(|i| {
                let fn_ = 0.20 + i as f64 * 0.015;
                let d = fn_ - 0.28;
                let ratio = 1.0 + 0.10 * d + 0.05 * d * d;
                CfdSample {
                    fn_,
                    empirical_hp: 1000.0,
                    cfd_hp: 1000.0 * ratio,
                    provenance: CfdProvenance::Synthetic,
                }
            })
            .collect();
        let report = cfd_holdout_report(&samples, 0.3);
        assert!(report.r2.is_finite());
        assert!(report.rmse.is_finite());
        assert!(report.samples > 0);
    }

    #[test]
    fn loocv_report_is_finite() {
        let samples: Vec<CfdSample> = (0..9)
            .map(|i| {
                let fn_ = 0.20 + i as f64 * 0.02;
                let d = fn_ - 0.28;
                let ratio = 1.0 + 0.10 * d + 0.05 * d * d;
                CfdSample {
                    fn_,
                    empirical_hp: 1000.0,
                    cfd_hp: 1000.0 * ratio,
                    provenance: CfdProvenance::Synthetic,
                }
            })
            .collect();
        let cv = cfd_loocv_report(&samples);
        assert!(cv.r2.is_finite());
        assert!(cv.rmse.is_finite());
        assert_eq!(cv.samples, 9);
    }

    #[test]
    fn fit_report_has_good_r2_for_synthetic_data() {
        let samples: Vec<CfdSample> = (0..9)
            .map(|i| {
                let fn_ = 0.20 + i as f64 * 0.02;
                let d = fn_ - 0.28;
                let ratio = 1.0 + 0.10 * d + 0.05 * d * d;
                CfdSample {
                    fn_,
                    empirical_hp: 1000.0,
                    cfd_hp: 1000.0 * ratio,
                    provenance: CfdProvenance::Synthetic,
                }
            })
            .collect();
        let report = fit_cfd_proxy_report(&samples);
        assert!(report.r2 > 0.99);
        assert!(report.rmse < 1e-3);
        assert_eq!(report.samples, 9);
    }

    #[test]
    fn fit_recovers_synthetic_polynomial() {
        let samples: Vec<CfdSample> = (0..9)
            .map(|i| {
                let fn_ = 0.20 + i as f64 * 0.02;
                let d = fn_ - 0.28;
                let ratio = 1.0 + 0.10 * d + 0.05 * d * d;
                CfdSample {
                    fn_,
                    empirical_hp: 1000.0,
                    cfd_hp: 1000.0 * ratio,
                    provenance: CfdProvenance::Synthetic,
                }
            })
            .collect();
        let cfg = fit_cfd_proxy(&samples);
        assert!(cfg.enabled);
        let check = cfg.correction_factor(0.28 + 0.05);
        let expected = 1.0 + 0.10 * 0.05 + 0.05 * 0.05 * 0.05;
        assert!((check - expected).abs() < 1e-6);
    }

    #[test]
    fn parse_cfd_samples_works() {
        let text = "fn,empirical_hp,cfd_hp\n0.25,1000,1100\n0.30,1000,1150\n";
        let samples = parse_cfd_samples(text).unwrap();
        assert_eq!(samples.len(), 2);
    }

    #[test]
    fn holdout_stability_is_finite() {
        let samples: Vec<CfdSample> = (0..12)
            .map(|i| {
                let fn_ = 0.20 + i as f64 * 0.015;
                let d = fn_ - 0.28;
                let ratio = 1.0 + 0.10 * d + 0.05 * d * d;
                CfdSample {
                    fn_,
                    empirical_hp: 1000.0,
                    cfd_hp: 1000.0 * ratio,
                    provenance: CfdProvenance::Synthetic,
                }
            })
            .collect();
        for (fraction, r2) in cfd_holdout_stability(&samples) {
            assert!(fraction > 0.0 && fraction < 1.0);
            assert!(r2.is_finite());
        }
    }

    #[test]
    fn disabled_correction_is_identity() {
        let c = CfdProxyConfig::default();
        assert!((c.correction_factor(0.3) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn correction_is_applied() {
        let c = CfdProxyConfig {
            enabled: true,
            viscous_correction: 0.1,
            wave_correction: 0.2,
            fn_ref: 0.25,
        };
        let f = c.correction_factor(0.35);
        assert!(f > 1.0);
        assert!(c.apply(1000.0, 0.35) > 1000.0);
    }
    #[test]
    fn provenance_and_status_are_honest() {
        let csv = "# fn,empirical_hp,cfd_hp,provenance\n\
0.5,1000,900,synthetic\n\
0.6,1000,950,analytic\n\
0.7,1000,1100,literature\n\
0.8,1000,1050,measured_cfd\n";
        let s = parse_cfd_samples(csv).unwrap();
        assert_eq!(s.len(), 4);
        let st = cfd_status(&s);
        assert_eq!(st.measured_cfd, 1);
        assert_eq!(st.synthetic, 1);
        assert_eq!(st.analytic, 1);
        assert_eq!(st.literature, 1);
        assert!(!st.usable_for_evidence);

        // 3-column rows default to synthetic (conservative).
        let s2 = parse_cfd_samples("0.5,1000,900\n").unwrap();
        assert_eq!(s2[0].provenance, CfdProvenance::Synthetic);
        assert!(CfdProvenance::parse("measured").unwrap().is_measured_cfd());
        assert!(CfdProvenance::parse("bogus").is_err());
    }

    #[test]
    fn analytic_reference_dataset_is_not_evidence() {
        let text = std::fs::read_to_string("data/examples/cfd_reference.csv").unwrap();
        let samples = parse_cfd_samples(&text).unwrap();
        assert!(samples.len() >= 20);
        assert!(samples
            .iter()
            .all(|s| s.provenance == CfdProvenance::Analytic));
        let st = cfd_status(&samples);
        assert_eq!(st.measured_cfd, 0);
        assert!(!st.usable_for_evidence);
    }
}
