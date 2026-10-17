//! `validation` command handlers.

use super::{opt_f64_arg, CliError};
use springsharp::*;

pub(crate) fn run(cmd: &str, args: &[String]) -> Result<(), CliError> {
    match cmd {
        "validate-regime" => {
            let r = run_loocv_regime();
            println!("Regime model: Holtrop-Mennen + per-ship-type calibration factor");
            println!("Nested LOOCV (out-of-sample):");
            println!(
                "  regime (Holtrop + factor) : {:.2}% case-weighted, {:.2}% type-averaged",
                r.overall_mape * 100.0,
                r.macro_mape * 100.0
            );
            println!(
                "  original (uncalibrated)   : {:.2}%  (original SpringSharp baseline)",
                r.original_uncalibrated_mape * 100.0
            );
            println!(
                "  original + per-type factor: {:.2}%",
                r.original_baseline_mape * 100.0
            );
            println!(
                "  Holtrop uncalibrated       : {:.2}%",
                r.holtrop_uncalibrated_mape * 100.0
            );
            println!(
                "  curated core ({} cases, conf>=0.65): {:.2}%  (approximate bulk excluded)",
                r.curated_cases,
                r.curated_mape * 100.0
            );
            println!("\nPer ship type (regime, nested):");
            for (ty, n, m) in &r.per_type {
                println!("  {:<12} n={:<2} {:.2}%", ty, n, m * 100.0);
            }
        }
        "accuracy-target" => {
            let target: f64 = opt_f64_arg(args, 1, "target", 0.10)?;
            let r = run_loocv_regime();
            println!(
                "Nested-LOOCV regime MAPE {:.2}% case-weighted / {:.2}% type-averaged vs target {:.1}%",
                r.overall_mape * 100.0,
                r.macro_mape * 100.0,
                target * 100.0
            );
            for (ty, n, m) in &r.per_type {
                println!("  {:<12} n={:<2} {:.2}%", ty, n, m * 100.0);
            }
            if !r.overall_mape.is_finite() || r.overall_mape > target {
                return Err(CliError::gate(format!(
                    "accuracy target not met: regime nested LOOCV {:.2}% > {:.2}%",
                    r.overall_mape * 100.0,
                    target * 100.0
                )));
            }
        }
        "validate-types" => {
            println!(
                "{:<14} {:>5} {:>12} {:>14}",
                "Ship type", "Count", "Orig MAPE", "Holtrop MAPE"
            );
            for g in validation_by_type() {
                println!(
                    "{:<14} {:>5} {:>11.1}% {:>13.1}%",
                    g.ship_type,
                    g.count,
                    g.mean_orig_error * 100.0,
                    g.mean_holtrop_error * 100.0
                );
            }
            println!(
                "Note: these are in-sample comparisons. For out-of-sample error use `validate-holdout` or the LOOCV block in `calibrate-stats`."
            );
        }
        "validate-holdout" => {
            let r = holdout_validation();
            println!("Holdout validation (calibration fitted on the training split only):");
            println!(
                "  training cases: {}   holdout cases: {}",
                r.train_cases, r.holdout_cases
            );
            println!(
                "  original : calibration={:.3}  holdout MAPE={:.1}%",
                r.original_factor,
                r.original_mape * 100.0
            );
            println!(
                "  Holtrop  : calibration={:.3}  holdout MAPE={:.1}%",
                r.holtrop_factor,
                r.holtrop_mape * 100.0
            );
            println!("  (LOOCV on all cases is reported by `calibrate-stats`.)");
        }
        "validate" => {
            let results = run_validation();
            println!(
                "{:<28} {:>12} {:>12} {:>12} {:>10}",
                "Ship", "Orig HP", "Holtrop HP", "Ref HP", "Error %"
            );
            for r in &results {
                println!(
                    "{:<28} {:>12.0} {:>12.0} {:>12.0} {:>9.1}%",
                    r.name, r.simulated_hp, r.holtrop_hp, r.reference_hp, r.error_percent
                );
            }
            let factor = calibration_factor(&results);
            let h_factor = calibration_factor_holtrop(&results);
            println!("\nSuggested calibration factor (ref/sim): {factor:.3}");
            println!("Holtrop suggested calibration factor: {h_factor:.3}");
            println!("\nSpeed-band calibration factors:");
            for seg in segmented_calibration(&results) {
                println!(
                    "  {:<22} orig={:.3} holtrop={:.3}",
                    seg.name, seg.factor_original, seg.factor_holtrop
                );
            }
            println!("\nData provenance summary:");
            let quality = data_quality_summary();
            let verified = quality
                .iter()
                .filter(|q| q.source_type.is_verified())
                .count();
            let mean_conf =
                quality.iter().map(|q| q.confidence).sum::<f64>() / quality.len().max(1) as f64;
            println!("  total cases: {}", quality.len());
            println!(
                "  independently verified (official trial / model test): {}",
                verified
            );
            println!(
                "  public reference / approximate: {}",
                quality.len() - verified
            );
            println!("  mean declared confidence: {mean_conf:.2}");
            println!(
                "  source: {}",
                quality.first().map(|q| q.source).unwrap_or("unknown")
            );
            println!(
                "  NOTE: built-in cases are public-reference approximations, NOT independent verification."
            );
            println!(
                "  Run `validate-holdout` for the out-of-sample estimate (calibration fit on training split only)."
            );
            println!("\nRelevant literature:");
            for r in springsharp::REFERENCES {
                println!("- {r}");
            }
        }
        "validate-blend" => {
            let b = run_loocv_blend();
            let r = run_loocv_regime();
            println!("Blended regime: Holtrop and original SpringSharp, per-type weights");
            println!("Nested LOOCV (out-of-sample):");
            println!(
                "  blended                  : {:.2}% case-weighted, {:.2}% type-averaged",
                b.overall_mape * 100.0,
                b.macro_mape * 100.0
            );
            println!(
                "  regime only (Holtrop)    : {:.2}%",
                r.overall_mape * 100.0
            );
            println!(
                "  original + per-type factor: {:.2}%",
                r.original_baseline_mape * 100.0
            );
            println!(
                "  curated core             : {:.2}%",
                b.curated_mape * 100.0
            );
            println!("\nPer ship type:");
            for (ty, n, m) in &b.per_type {
                println!("  {ty:<12} n={n:<3} {:.2}%", m * 100.0);
            }
        }
        "regime-compare" => {
            let c = compare_regime_vs_original();
            println!("Baseline comparison (nested, {} cases):", c.n);
            println!("  {:<26} {:>8} {:>12}", "Model", "MAPE", "median |err|");
            for (name, mape, median) in [
                ("regime (Holtrop + factor)", c.regime_mape, c.regime_median),
                (
                    "original (uncalibrated)",
                    c.original_mape,
                    c.original_median,
                ),
                (
                    "original + per-type factor",
                    c.original_calibrated_mape,
                    f64::NAN,
                ),
                ("Holtrop (uncalibrated)", c.holtrop_mape, f64::NAN),
            ] {
                if median.is_finite() {
                    println!(
                        "  {name:<26} {:>7.2}% {:>11.2}%",
                        mape * 100.0,
                        median * 100.0
                    );
                } else {
                    println!("  {name:<26} {:>7.2}% {:>12}", mape * 100.0, "-");
                }
            }
            println!(
                "\nPaired (regime vs original): {} wins / {} losses / {} ties",
                c.regime_wins, c.original_wins, c.ties
            );
            println!("  two-sided sign-test p = {:.2e}", c.sign_test_p);
            println!("\nPer ship type (regime MAPE vs original MAPE, wins/n):");
            for (ty, n, rm, om, w) in &c.per_type {
                println!(
                    "  {:<12} n={:<3} {:>6.2}% vs {:>6.2}%   wins {}/{}",
                    ty,
                    n,
                    rm * 100.0,
                    om * 100.0,
                    w,
                    n
                );
            }
        }
        "regime-residuals" => {
            let r = run_loocv_regime();
            let mut rows = r.residuals.clone();
            rows.sort_by(|a, b| {
                b.error_pct
                    .abs()
                    .partial_cmp(&a.error_pct.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            println!(
                "{:<28} {:>10} {:>10} {:>9} {:>6}",
                "Ship", "Ref HP", "Regime HP", "Error %", "Conf"
            );
            for row in &rows {
                println!(
                    "{:<28} {:>10.0} {:>10.0} {:>8.1}% {:>6.2}",
                    row.name, row.reference_hp, row.predicted_hp, row.error_pct, row.confidence
                );
            }
            println!(
                "\nOverall nested LOOCV: {:.2}% case-weighted / {:.2}% type-averaged / {:.2}% curated core ({} cases)",
                r.overall_mape * 100.0,
                r.macro_mape * 100.0,
                r.curated_mape * 100.0,
                r.curated_cases
            );
            println!(
                "Residuals are against public historical references (approximate, not trial data)."
            );
        }
        "regime-table" => {
            println!(
                "{:<12} {:>4} {:>18} {:>9} {:>11}",
                "Type", "N", "Model", "Factor", "Train MAPE"
            );
            for r in fit_regime_calibration() {
                println!(
                    "{:<12} {:>4} {:>18} {:>9.3} {:>10.1}%",
                    r.ship_type,
                    r.count,
                    r.model_label,
                    r.factor,
                    r.train_mape * 100.0
                );
            }
            println!(
                "\nNote: Train MAPE is in-sample/diagnostic. Use `validate-regime` for nested LOOCV."
            );
        }
        "validate-weights" => {
            let report = validate_preset_weights();
            println!(
                "Subsystem Weight & Stability Validation across {} Full-Design Historical Presets",
                report.rows.len()
            );
            println!(
                "{:<16} {:>13} {:>15} {:>15} {:>15} {:>15} {:>13}",
                "Ship",
                "Disp (s/r)",
                "Hull (s/r %)",
                "Armour (s/r %)",
                "Mach (s/r %)",
                "Arm+Am (s/r %)",
                "GM ft (s/r %)"
            );
            for r in &report.rows {
                let b = &r.benchmark;
                let armour_cell = match r.armour_err_pct {
                    Some(pct) => {
                        format!("{:.0}/{:.0} {:+.0}%", r.sim_armour_t, b.ref_armour_t, pct)
                    }
                    None => format!("{:.0}/{:.0} (n/a)", r.sim_armour_t, b.ref_armour_t),
                };
                println!(
                    "{:<16} {:>13} {:>15} {:>15} {:>15} {:>15} {:>13}",
                    b.name,
                    format!("{:.0}/{:.0}", r.sim_displacement_t, b.ref_displacement_t),
                    format!(
                        "{:.0}/{:.0} {:+.0}%",
                        r.sim_hull_t, b.ref_hull_t, r.hull_err_pct
                    ),
                    armour_cell,
                    format!(
                        "{:.0}/{:.0} {:+.0}%",
                        r.sim_machinery_t, b.ref_machinery_t, r.machinery_err_pct
                    ),
                    format!(
                        "{:.0}/{:.0} {:+.0}%",
                        r.sim_armament_t, b.ref_armament_t, r.armament_err_pct
                    ),
                    format!(
                        "{:.1}/{:.1} {:+.0}%",
                        r.sim_gm_ft, b.ref_gm_ft, r.gm_err_pct
                    ),
                );
            }
            println!("\nSummary MAPE & Physical Feasibility:");
            println!(
                "  Normal displacement MAPE (all 8)         : {:.2}%",
                report.disp_mape * 100.0
            );
            println!(
                "  Armour weight MAPE (6 armoured warships) : {:.2}%",
                report.armour_mape * 100.0
            );
            println!(
                "  Metacentric height GM MAPE (all 8)       : {:.2}%",
                report.gm_mape * 100.0
            );
            println!(
                "  Hull weight MAPE (6 armoured / all 8)    : {:.2}% / {:.2}%",
                report.hull_armoured_mape * 100.0,
                report.hull_mape * 100.0
            );
            println!(
                "  Armament+Ammo MAPE (6 armoured / all 8)  : {:.2}% / {:.2}%",
                report.armament_armoured_mape * 100.0,
                report.armament_mape * 100.0
            );
            println!(
                "  Machinery weight MAPE (6 armoured / all 8): {:.2}% / {:.2}%",
                report.machinery_armoured_mape * 100.0,
                report.machinery_mape * 100.0
            );
            println!(
                "  Physical feasibility                     : {}/{} positive hull, {}/{} positive magazine, {}/{} converged",
                report.physical_hull_count,
                report.rows.len(),
                report.positive_magazine_count,
                report.rows.len(),
                report.converged_count,
                report.rows.len()
            );
            println!(
                "\nNote: SpringSharp 3.0 computes hull weight as the residual (disp - armour - machinery - armament - load).\n      Phase 32 introduced `machinery_weight_factor` which calibrates lightweight/high-pressure destroyer steam\n      plants (Fletcher +3%, Shimakaze +5%), bringing overall 8-preset machinery MAPE from 60.63% to 29.69%."
            );
        }
        _ => unreachable!("validation::run called with unexpected command {cmd}"),
    }
    Ok(())
}
