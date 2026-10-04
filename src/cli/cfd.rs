//! `cfd` command handlers.

use super::{load_ship, opt_f64_arg, parse_f64_arg, read_file, require_arg, save_ship, CliError};
use springsharp::*;

pub(crate) fn run(cmd: &str, args: &[String]) -> Result<(), CliError> {
    match cmd {
        "cfd-apply" => {
            let csv = require_arg(args, 1, "cfd-apply <csv> <design.ssr> [min_r2] [force]")?;
            let design = require_arg(args, 2, "cfd-apply <csv> <design.ssr> [min_r2] [force]")?;
            let threshold: f64 = opt_f64_arg(args, 3, "min_r2", 0.5)?;
            let force = args.get(4).map(|x| x == "force").unwrap_or(false)
                || args.iter().any(|x| x == "--force");
            let text = read_file(csv)?;
            let samples = parse_cfd_samples(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse CFD samples: {e}")))?;
            let cfd = cfd_status(&samples);
            if !cfd.usable_for_evidence && !force {
                eprintln!(
                    "Refusing to apply CFD proxy: {} measured CFD sample(s) (< {} minimum). \
    literature={}, analytic={}, synthetic={}. Use `force` to override.",
                    cfd.measured_cfd, cfd.min_samples, cfd.literature, cfd.analytic, cfd.synthetic
                );
                std::process::exit(3);
            }
            let selection = select_cfd_ridge_lambda(&samples);
            let cv = cfd_loocv_report(&samples);
            if cv.r2.is_finite() && cv.r2 < threshold && !force {
                eprintln!(
                    "Refusing to apply CFD proxy: LOOCV R2={:.4} < threshold {:.3}. Use `force` to override.",
                    cv.r2, threshold
                );
                std::process::exit(3);
            }
            let cfg = fit_cfd_proxy_ridge(&samples, selection.best_lambda);
            let mut ship = load_ship(design)?;
            ship.engine.cfd_proxy_enabled = cfg.enabled;
            ship.engine.cfd_proxy_viscous = cfg.viscous_correction;
            ship.engine.cfd_proxy_wave = cfg.wave_correction;
            ship.engine.cfd_proxy_fn_ref = cfg.fn_ref;
            save_ship(&ship, design)?;
            println!(
                "Applied CFD proxy to {design}: lambda={:.3} viscous={:.4} wave={:.4} fn_ref={:.4} LOOCV_R2={:.4}",
                selection.best_lambda, cfg.viscous_correction, cfg.wave_correction, cfg.fn_ref, cv.r2
            );
        }
        "cfd-select-lambda" => {
            let path = require_arg(args, 1, "cfd-select-lambda <csv>")?;
            let text = read_file(path)?;
            let samples = parse_cfd_samples(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse CFD samples: {e}")))?;
            let selection = select_cfd_ridge_lambda(&samples);
            for (lambda, rmse) in &selection.results {
                println!("lambda={lambda:<7.3} LOOCV RMSE={rmse:.5}");
            }
            println!(
                "best lambda={:.3} LOOCV RMSE={:.5}",
                selection.best_lambda, selection.best_rmse
            );
        }
        "cfd-fit-ridge" => {
            let path = require_arg(args, 1, "cfd-fit-ridge <csv> <lambda>")?;
            let lambda = require_arg(args, 2, "cfd-fit-ridge <csv> <lambda>")?;
            let text = read_file(path)?;
            let samples = parse_cfd_samples(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse CFD samples: {e}")))?;
            let lambda: f64 = parse_f64_arg(lambda, "lambda")?;
            let cfg = fit_cfd_proxy_ridge(&samples, lambda);
            println!(
                "CFD ridge fit: viscous={:.4} wave={:.4} fn_ref={:.4} lambda={:.3}",
                cfg.viscous_correction, cfg.wave_correction, cfg.fn_ref, lambda
            );
        }
        "cfd-report" => {
            let path = require_arg(args, 1, "cfd-report <csv>")?;
            let text = read_file(path)?;
            let samples = parse_cfd_samples(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse CFD samples: {e}")))?;
            print!("{}", cfd_report_html(&samples));
        }
        "cfd-status" => {
            let path = require_arg(args, 1, "cfd-status <csv>")?;
            let text = read_file(path)?;
            let samples = parse_cfd_samples(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse CFD samples: {e}")))?;
            let status = cfd_status(&samples);
            println!("CFD surrogate status for {path}:");
            println!("  samples: {}", status.samples);
            println!(
                "  provenance: measured_cfd={}, literature={}, analytic={}, synthetic={}",
                status.measured_cfd, status.literature, status.analytic, status.synthetic
            );
            println!(
                "  minimum measured CFD for evidence: {}",
                status.min_samples
            );
            println!("  usable_for_evidence: {}", status.usable_for_evidence);
            println!("  {}", status.message);
        }
        "cfd-fit" => {
            let path = require_arg(args, 1, "cfd-fit <csv>")?;
            let text = read_file(path)?;
            let samples = parse_cfd_samples(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse CFD samples: {e}")))?;
            let report = fit_cfd_proxy_report(&samples);
            println!(
                "CFD proxy fit: enabled={} viscous={:.4} wave={:.4} fn_ref={:.4} R2={:.4} RMSE={:.5} N={}",
                report.config.enabled,
                report.config.viscous_correction,
                report.config.wave_correction,
                report.config.fn_ref,
                report.r2,
                report.rmse,
                report.samples
            );
            let cv = cfd_loocv_report(&samples);
            println!(
                "CFD proxy LOOCV: R2={:.4} RMSE={:.5} N={}",
                cv.r2, cv.rmse, cv.samples
            );
            let holdout = cfd_holdout_report(&samples, 0.3);
            println!(
                "CFD proxy holdout(0.3): R2={:.4} RMSE={:.5} N={}",
                holdout.r2, holdout.rmse, holdout.samples
            );
            print!("CFD holdout stability:");
            for (fraction, r2) in cfd_holdout_stability(&samples) {
                print!(" {:.1}:{:.3}", fraction, r2);
            }
            println!();
        }
        "cfd-proxy" => {
            let hp = require_arg(args, 1, "cfd-proxy <hp> <fn> [visc] [wave] [fn_ref]")?;
            let fn_ = require_arg(args, 2, "cfd-proxy <hp> <fn> [visc] [wave] [fn_ref]")?;
            let hp: f64 = parse_f64_arg(hp, "hp")?;
            let fn_: f64 = parse_f64_arg(fn_, "fn")?;
            let visc: f64 = opt_f64_arg(args, 3, "visc", 0.0)?;
            let wave: f64 = opt_f64_arg(args, 4, "wave", 0.0)?;
            let fn_ref: f64 = opt_f64_arg(args, 5, "fn_ref", 0.25)?;
            let cfg = CfdProxyConfig {
                enabled: true,
                viscous_correction: visc,
                wave_correction: wave,
                fn_ref,
            };
            println!(
                "CFD-corrected hp: {:.0} (factor {:.4})",
                cfg.apply(hp, fn_),
                cfg.correction_factor(fn_)
            );
        }
        _ => unreachable!("cfd::run called with unexpected command {cmd}"),
    }
    Ok(())
}
