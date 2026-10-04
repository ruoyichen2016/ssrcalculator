//! `calibration` command handlers.

use super::{load_ship, read_file, require_arg, save_ship, CliError};
use springsharp::*;

pub(crate) fn run(cmd: &str, args: &[String]) -> Result<(), CliError> {
    match cmd {
        "calibrate" => {
            let path = require_arg(args, 1, "calibrate <file>")?;
            let mut ship = load_ship(path)?;
            let results = run_validation();
            let segments = segmented_calibration(&results);
            let speed = ship.engine.speed_max;
            let length_ft = ship.hull.length;
            let fn_ship = froude_number(speed, length_ft);
            let factor = segments
                .iter()
                .find(|s| fn_ship > s.min_froude && fn_ship <= s.max_froude)
                .map(|s| {
                    if ship.engine.resistance_model == springsharp::ResistanceModel::HoltropMennen {
                        s.factor_holtrop
                    } else {
                        s.factor_original
                    }
                })
                .unwrap_or_else(|| {
                    if ship.engine.resistance_model == springsharp::ResistanceModel::HoltropMennen {
                        calibration_factor_holtrop(&results)
                    } else {
                        calibration_factor(&results)
                    }
                });
            if ship.engine.resistance_model == springsharp::ResistanceModel::HoltropMennen {
                ship.engine.power_factor = factor;
            } else {
                ship.engine.original_power_factor = factor;
            }
            ship.engine.blend_weight = 1.0;
            ship.calculate();
            println!("Applied historical calibration factor {factor:.3} to {path}");
            print!("{}", ship.summary());
        }
        "calibrate-structural-segmented" => {
            let path = require_arg(args, 1, "calibrate-structural-segmented <file>")?;
            let mut ship = load_ship(path)?;
            apply_structural_calibration_segmented(&mut ship);
            ship.calculate();
            println!(
                "Applied Fn-segmented structural Holtrop calibration to {path}: viscous={:.3}, wave={:.3}, correlation={:.3}",
                ship.engine.holtrop_viscous_factor,
                ship.engine.holtrop_wave_factor,
                ship.engine.holtrop_correlation_factor
            );
            print!("{}", ship.summary());
        }
        "calibrate-structural" => {
            let path = require_arg(args, 1, "calibrate-structural <file>")?;
            let mut ship = load_ship(path)?;
            apply_structural_calibration(&mut ship);
            ship.calculate();
            println!(
                "Applied structural Holtrop calibration to {path}: viscous={:.3}, wave={:.3}, correlation={:.3}",
                ship.engine.holtrop_viscous_factor,
                ship.engine.holtrop_wave_factor,
                ship.engine.holtrop_correlation_factor
            );
            print!("{}", ship.summary());
        }
        "calibrate-ridge" => {
            let path = require_arg(args, 1, "calibrate-ridge <file>")?;
            let mut ship = load_ship(path)?;
            apply_structural_calibration_ridge(&mut ship);
            ship.calculate();
            save_ship(&ship, path)?;
            println!("Applied Ridge-regularized structural calibration to {path}");
            print!("{}", ship.summary());
        }
        "calibrate-lasso" => {
            let path = require_arg(args, 1, "calibrate-lasso <file>")?;
            let mut ship = load_ship(path)?;
            apply_structural_calibration_lasso(&mut ship);
            ship.calculate();
            save_ship(&ship, path)?;
            println!("Applied Lasso-regularized structural calibration to {path}");
            print!("{}", ship.summary());
        }
        "apply-ensemble" => {
            let path = require_arg(args, 1, "apply-ensemble <file>")?;
            let mut ship = load_ship(path)?;
            apply_ensemble_calibration(&mut ship);
            ship.calculate();
            save_ship(&ship, path)?;
            println!(
                "Applied LOOCV ensemble weights to {path}: reduced={:.3}, structural={:.3}",
                ship.engine.ensemble_reduced_weight, ship.engine.ensemble_structural_weight
            );
            print!("{}", ship.summary());
        }
        "apply-ensemble-external" => {
            let path = require_arg(args, 1, "apply-ensemble-external <file> <csv>")?;
            let csv_path = require_arg(args, 2, "apply-ensemble-external <file> <csv>")?;
            let text = read_file(csv_path)?;
            let cases = parse_external_cases(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse external cases: {e}")))?;
            let mut ship = load_ship(path)?;
            apply_ensemble_calibration_with_external(&mut ship, &cases);
            ship.calculate();
            save_ship(&ship, path)?;
            println!(
                "Applied external-data ensemble weights to {path}: reduced={:.3}, structural={:.3}",
                ship.engine.ensemble_reduced_weight, ship.engine.ensemble_structural_weight
            );
            print!("{}", ship.summary());
        }
        "calibrate-external" => {
            let path = require_arg(args, 1, "calibrate-external <file>")?;
            let text = read_file(path)?;
            let cases = parse_external_cases(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse external cases: {e}")))?;
            let rf = fit_reduced_form_with_external(&cases);
            let sf = fit_structural_form_with_external(&cases);
            println!("Combined built-in + {} external cases", cases.len());
            println!("Reduced form: MAPE={:.1}%", rf.mean_abs_pct_error * 100.0);
            println!(
                "Structural form: viscous={:.3} wave={:.3} MAPE={:.1}%",
                sf.viscous_factor,
                sf.wave_factor,
                sf.mean_abs_pct_error * 100.0
            );
            let loocv_ext = run_loocv_with_external(&cases);
            println!(
                "External LOOCV: reduced={:.1}% structural={:.1}% ensemble={:.1}%",
                loocv_ext.reduced_mape * 100.0,
                loocv_ext.structural_mape * 100.0,
                loocv_ext.ensemble_mape * 100.0
            );
            let mut boot_ship = demo_ship();
            boot_ship.calculate();
            let boot_ext = bootstrap_uncertainty_with_external(&cases, &boot_ship, 30);
            println!(
                "External bootstrap ensemble interval: {:.0} / {:.0} / {:.0} hp",
                boot_ext.ensemble.lower, boot_ext.ensemble.median, boot_ext.ensemble.upper
            );
            let loocv_ext_w = run_loocv_with_external_weighted(&cases);
            println!(
                "External weighted LOOCV: reduced={:.1}% structural={:.1}% ensemble={:.1}%",
                loocv_ext_w.reduced_mape * 100.0,
                loocv_ext_w.structural_mape * 100.0,
                loocv_ext_w.ensemble_mape * 100.0
            );
            let boot_ext_w = bootstrap_uncertainty_with_external_weighted(&cases, &boot_ship, 30);
            println!(
                "External weighted bootstrap ensemble interval: {:.0} / {:.0} / {:.0} hp",
                boot_ext_w.ensemble.lower, boot_ext_w.ensemble.median, boot_ext_w.ensemble.upper
            );
            println!("\nExternal data by source:");
            for (source, count, conf) in summarize_external_by_source(&cases) {
                println!("  {source}: count={count}, mean confidence={conf:.2}");
            }
            println!("\nExternal data by ship type:");
            for (ship_type, count, conf) in summarize_external_by_type(&cases) {
                println!("  {ship_type}: count={count}, mean confidence={conf:.2}");
            }
        }
        "calibrate-stats" => {
            let rf = fit_reduced_form();
            let rf_type = fit_reduced_form_by_type();
            let sf = fit_structural_form();
            let segments = fit_structural_form_segmented();
            let ens = fit_ensemble();

            // LOOCV is the primary accuracy metric: it is computed on held-out
            // folds, unlike the in-sample fits printed below.
            let loocv = run_loocv();
            let ens_cv = fit_ensemble_cv();
            println!("=== OUT-OF-SAMPLE SUMMARY (primary metric) ===");
            println!(
                "LOOCV MAPE: reduced={:.1}%  structural={:.1}%  ensemble={:.1}%",
                loocv.reduced_mape * 100.0,
                loocv.structural_mape * 100.0,
                loocv.ensemble_mape * 100.0
            );
            println!(
                "LOOCV-optimal ensemble weights: reduced={:.3} structural={:.3}",
                ens_cv.reduced_weight, ens_cv.structural_weight
            );
            println!(
                "\n=== IN-SAMPLE FITS (diagnostic only; optimistic, do NOT use as accuracy evidence) ==="
            );
            println!(
                "These reuse all 37 cases and can look far better than true out-of-sample error.\n"
            );
            println!("Reduced-form power regression (log P = b0 + b1 ln Displacement + b2 ln Speed + b3 ln Length):");
            println!(
                "  b0={:.4}  b1={:.4}  b2={:.4}  b3={:.4}  MAPE={:.1}%  sigma={:.1}%",
                rf.intercept,
                rf.log_displacement,
                rf.log_speed,
                rf.log_length,
                rf.mean_abs_pct_error * 100.0,
                rf.sigma_abs_pct_error * 100.0
            );
            println!("\nReduced-form with ship-type dummies (reference: battleship):");
            println!(
                "  b0={:.4}  b1={:.4}  b2={:.4}  b3={:.4}  carrier={:.3}  cruiser={:.3}  destroyer={:.3}  MAPE={:.1}%  sigma={:.1}%",
                rf_type.intercept,
                rf_type.log_displacement,
                rf_type.log_speed,
                rf_type.log_length,
                rf_type.d_carrier,
                rf_type.d_cruiser,
                rf_type.d_destroyer,
                rf_type.mean_abs_pct_error * 100.0,
                rf_type.sigma_abs_pct_error * 100.0
            );
            println!("\nStructural-form Holtrop component multipliers (global):");
            println!(
                "  viscous={:.3}  wave={:.3}  correlation={:.3}  MAPE={:.1}%  sigma={:.1}%",
                sf.viscous_factor,
                sf.wave_factor,
                sf.correlation_factor,
                sf.mean_abs_pct_error * 100.0,
                sf.sigma_abs_pct_error * 100.0
            );
            println!("\nFn-segmented structural multipliers:");
            for seg in &segments {
                if !seg.mean_abs_pct_error.is_finite() {
                    continue;
                }
                println!(
                    "  {:<20} viscous={:.3} wave={:.3} MAPE={:.1}% sigma={:.1}%",
                    seg.name,
                    seg.viscous_factor,
                    seg.wave_factor,
                    seg.mean_abs_pct_error * 100.0,
                    seg.sigma_abs_pct_error * 100.0
                );
            }
            println!("\nConfidence-weighted Fn-segmented structural multipliers:");
            for seg in fit_structural_form_segmented_weighted() {
                if !seg.mean_abs_pct_error.is_finite() {
                    continue;
                }
                println!(
                    "  {:<20} viscous={:.3} wave={:.3} MAPE={:.1}%",
                    seg.name,
                    seg.viscous_factor,
                    seg.wave_factor,
                    seg.mean_abs_pct_error * 100.0
                );
            }

            let rf_w = fit_reduced_form_weighted();
            let sf_w = fit_structural_form_weighted();
            println!("\nConfidence-weighted reduced form:");
            println!(
                "  b0={:.4} b1={:.4} b2={:.4} b3={:.4} MAPE={:.1}%",
                rf_w.intercept,
                rf_w.log_displacement,
                rf_w.log_speed,
                rf_w.log_length,
                rf_w.mean_abs_pct_error * 100.0
            );
            println!("Confidence-weighted structural form:");
            println!(
                "  viscous={:.3} wave={:.3} MAPE={:.1}%",
                sf_w.viscous_factor,
                sf_w.wave_factor,
                sf_w.mean_abs_pct_error * 100.0
            );

            println!("\nEnsemble (inverse-error weighted):");
            println!(
                "  reduced={:.3} structural={:.3} MAPE={:.1}% sigma={:.1}%",
                ens.reduced_weight,
                ens.structural_weight,
                ens.mean_abs_pct_error * 100.0,
                ens.sigma_abs_pct_error * 100.0
            );

            let (_, _, qr_svd_diff) = compare_qr_svd();
            println!("\nQR vs SVD reduced-form max coefficient difference: {qr_svd_diff:.3e}");

            let inter = fit_reduced_form_interaction();
            println!("\nReduced-form with Froude x type interactions:");
            println!(
                "  MAPE={:.1}% sigma={:.1}%",
                inter.mean_abs_pct_error * 100.0,
                inter.sigma_abs_pct_error * 100.0
            );

            println!("\nRidge-regularized Fn-segmented structural multipliers:");
            for seg in fit_structural_form_segmented_ridge(0.01) {
                if !seg.mean_abs_pct_error.is_finite() {
                    continue;
                }
                println!(
                    "  {:<20} viscous={:.3} wave={:.3} MAPE={:.1}%",
                    seg.name,
                    seg.viscous_factor,
                    seg.wave_factor,
                    seg.mean_abs_pct_error * 100.0
                );
            }

            println!("\nLasso-regularized Fn-segmented structural multipliers:");
            for seg in fit_structural_form_segmented_lasso(0.01) {
                if !seg.mean_abs_pct_error.is_finite() {
                    continue;
                }
                println!(
                    "  {:<20} viscous={:.3} wave={:.3} MAPE={:.1}%",
                    seg.name,
                    seg.viscous_factor,
                    seg.wave_factor,
                    seg.mean_abs_pct_error * 100.0
                );
            }

            println!("\nLOOCV per-sample residuals (out-of-sample):");
            println!(
                "{:<28} {:>10} {:>10} {:>10} {:>10}",
                "Ship", "Ref HP", "Reduced", "Structural", "Ensemble"
            );
            for fold in &loocv.folds {
                let ens_pred = ens_cv.reduced_weight * fold.reduced_hp
                    + ens_cv.structural_weight * fold.structural_hp;
                println!(
                    "{:<28} {:>10.0} {:>10.0} {:>10.0} {:>10.0}",
                    fold.name, fold.reference_hp, fold.reduced_hp, fold.structural_hp, ens_pred
                );
            }

            let ridge_sel = select_ridge_lambda();
            println!("\nRidge lambda selection (LOOCV):");
            for (lambda, mape) in &ridge_sel.results {
                println!("  lambda={lambda:<8} MAPE={:.1}%", mape * 100.0);
            }
            println!(
                "  best lambda={:.4} MAPE={:.1}%",
                ridge_sel.best_lambda,
                ridge_sel.best_loocv_mape * 100.0
            );

            let lasso_sel = select_lasso_lambda();
            println!("\nLasso lambda selection (LOOCV):");
            for (lambda, mape) in &lasso_sel.results {
                println!("  lambda={lambda:<8} MAPE={:.1}%", mape * 100.0);
            }
            println!(
                "  best lambda={:.4} MAPE={:.1}%",
                lasso_sel.best_lambda,
                lasso_sel.best_loocv_mape * 100.0
            );

            let mut boot_ship = demo_ship();
            boot_ship.calculate();
            let boot = bootstrap_uncertainty_for_ship(&boot_ship, 60);
            println!("\nBootstrap 5%-50%-95% power intervals (demo ship, 60 resamples):");
            println!(
                "  reduced:     {:.0} / {:.0} / {:.0} hp",
                boot.reduced.lower, boot.reduced.median, boot.reduced.upper
            );
            println!(
                "  structural:  {:.0} / {:.0} / {:.0} hp",
                boot.structural.lower, boot.structural.median, boot.structural.upper
            );
            println!(
                "  ensemble:    {:.0} / {:.0} / {:.0} hp",
                boot.ensemble.lower, boot.ensemble.median, boot.ensemble.upper
            );

            let loocv_w = run_loocv_weighted();
            println!("\nConfidence-weighted LOOCV:");
            println!(
                "  reduced MAPE={:.1}% structural MAPE={:.1}% ensemble MAPE={:.1}% weights reduced={:.2}",
                loocv_w.reduced_mape * 100.0,
                loocv_w.structural_mape * 100.0,
                loocv_w.ensemble_mape * 100.0,
                loocv_w.optimal_reduced_weight
            );
            let boot_w = bootstrap_uncertainty_for_ship_weighted(&boot_ship, 60);
            println!("Confidence-weighted bootstrap ensemble interval:");
            println!(
                "  {:.0} / {:.0} / {:.0} hp",
                boot_w.ensemble.lower, boot_w.ensemble.median, boot_w.ensemble.upper
            );
        }
        "calibrate-type" => {
            let path = require_arg(args, 1, "calibrate-type <file>")?;
            let mut ship = load_ship(path)?;
            let ty = ship.ship_type.to_ascii_lowercase();
            let factors = type_calibration_factors();
            let Some(entry) = factors.iter().find(|f| ty.contains(f.ship_type)) else {
                eprintln!("No calibration entry for ship type '{ty}'");
                std::process::exit(3);
            };
            let factor =
                if ship.engine.resistance_model == springsharp::ResistanceModel::HoltropMennen {
                    entry.factor_holtrop
                } else {
                    entry.factor_original
                };
            if ship.engine.resistance_model == springsharp::ResistanceModel::HoltropMennen {
                ship.engine.power_factor = factor;
            } else {
                ship.engine.original_power_factor = factor;
            }
            ship.engine.blend_weight = 1.0;
            ship.calculate();
            save_ship(&ship, path)?;
            println!(
                "Applied {} calibration factor {:.3} to {path}",
                entry.ship_type, factor
            );
        }
        "calibrate-regime" => {
            let path = require_arg(args, 1, "calibrate-regime <file>")?;
            let mut ship = load_ship(path)?;
            ship.calculate();
            match apply_regime_calibration(&mut ship) {
                Some(entry) => {
                    save_ship(&ship, path)?;
                    println!(
                        "Applied regime to {path}: type={} model={} factor={:.3} (in-sample MAPE {:.1}%)",
                        entry.ship_type,
                        entry.model_label,
                        entry.factor,
                        entry.train_mape * 100.0
                    );
                    println!("NOTE: use `validate-regime` for the out-of-sample error.");
                }
                None => {
                    eprintln!("No regime entry for ship type '{}'", ship.ship_type);
                    std::process::exit(3);
                }
            }
        }
        "calibrate-blend" => {
            let path = require_arg(args, 1, "calibrate-blend <file>")?;
            let mut ship = load_ship(path)?;
            ship.calculate();
            match apply_blend_calibration(&mut ship) {
                Some(b) => {
                    save_ship(&ship, path)?;
                    println!(
                        "Applied blended regime to {path}: type={} Holtrop w={:.2} (Holtrop factor {:.3}, original factor {:.3})",
                        b.ship_type, b.weight, b.holtrop_factor, b.original_factor
                    );
                    println!("NOTE: use `validate-blend` for the out-of-sample error.");
                }
                None => {
                    eprintln!("No blend entry for ship type '{}'", ship.ship_type);
                    std::process::exit(3);
                }
            }
        }
        "blend-table" => {
            println!(
                "{:<12} {:>4} {:>8} {:>10} {:>11} {:>10}",
                "Type", "N", "Holtrop w", "Holt factor", "Orig factor", "Train MAPE"
            );
            for b in fit_blend_calibration() {
                println!(
                    "{:<12} {:>4} {:>8.2} {:>10.3} {:>11.3} {:>9.1}%",
                    b.ship_type,
                    b.count,
                    b.weight,
                    b.holtrop_factor,
                    b.original_factor,
                    b.train_mape * 100.0
                );
            }
            println!("Note: Train MAPE is in-sample/diagnostic; use `validate-blend`.");
        }
        _ => unreachable!("calibration::run called with unexpected command {cmd}"),
    }
    Ok(())
}
