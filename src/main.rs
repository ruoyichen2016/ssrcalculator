use std::io::{self, BufRead, Write};

use springsharp::{
    apply_ensemble_calibration, apply_ensemble_calibration_with_external, apply_regime_calibration,
    apply_structural_calibration, apply_structural_calibration_lasso,
    apply_structural_calibration_ridge, apply_structural_calibration_segmented,
    bootstrap_uncertainty_for_ship, bootstrap_uncertainty_for_ship_weighted,
    bootstrap_uncertainty_with_external, bootstrap_uncertainty_with_external_weighted,
    calculate_internal_spaces, calibration_factor, calibration_factor_holtrop, cfd_holdout_report,
    cfd_holdout_stability, cfd_loocv_report, cfd_report_html, cfd_status, compare_qr_svd,
    compartment_placement_html, compartment_placement_json, compute_planing_calibration,
    compute_planing_calibration_from_files, data_quality_report_html, data_quality_score,
    data_quality_summary, data_replacement_log, data_replacement_priority,
    data_replacement_progress, data_source_summary, data_target_matrix_for, data_tier_summary,
    data_version, demo_ship, deserialize, detect_duplicate_external_cases, external_report_html,
    fit_cfd_proxy_report, fit_cfd_proxy_ridge, fit_ensemble, fit_ensemble_cv, fit_reduced_form,
    fit_reduced_form_by_type, fit_reduced_form_interaction, fit_reduced_form_weighted,
    fit_reduced_form_with_external, fit_regime_calibration, fit_structural_form,
    fit_structural_form_segmented, fit_structural_form_segmented_lasso,
    fit_structural_form_segmented_ridge, fit_structural_form_segmented_weighted,
    fit_structural_form_weighted, fit_structural_form_with_external, format_compartment_report,
    format_space_report, froude_number, generate_html_report, generate_report, holdout_validation,
    is_planing_eligible, load_from_file, parse_cfd_samples, parse_external_cases,
    planing_hp_estimate_calibrated, quality_gate_by_type, quality_gate_with_loocv, run_loocv,
    run_loocv_regime, run_loocv_weighted, run_loocv_with_external,
    run_loocv_with_external_weighted, run_server, run_validation, save_to_file, savitsky_planing,
    segmented_calibration, select_cfd_ridge_lambda, select_lasso_lambda, select_ridge_lambda,
    small_fast_error_diagnostic, solve_max_speed, solve_range, suggest_compartment_placement,
    summarize_external_by_source, summarize_external_by_type, type_calibration_factors,
    validate_external_cases, validate_planing_cases, validation_by_type, CfdProxyConfig,
    PlaningInput, ShipModel,
};

fn print_help() {
    println!(
        "SpringSharp Rust CLI

USAGE:
  springsharp [COMMAND]

COMMANDS:
  demo                 Build and show the built-in example ship
  save <file>          Save the example ship to a Rust SpringSharp file
  load <file>          Load a saved design and print its summary
  report <file>        Generate a text report for a saved design
  html <file>          Generate an HTML report for a saved design
  solve-speed <file> <hp>       Solve max speed for a target HP
  solve-range <file> <bunker>   Solve range for a target normal bunker
  web [port]           Start a local web console (default 127.0.0.1:8080)
  validate             Run historical speed/power validation
  validate-types       Show validation errors grouped by ship type
  validate-holdout     Out-of-sample check (calibration fit on training split only)
  validate-regime      Nested-LOOCV comparison of regime model vs original baseline
  accuracy-target [m]  Check nested regime LOOCV against a MAPE target (default 0.10)
  regime-table         Show production regime table (Holtrop + per-ship-type factor)
  calibrate-regime <file>  Apply Holtrop + per-ship-type factor (modern default)
  calibrate-type <file>  Apply per-ship-type power calibration
  calibrate <file>     Apply historical power calibration to a design
  spaces <file>        Show improved internal-space volume model
  compartments <file> [--json]  Show multi-deck GA allocation + access network
  optimize-compartments <file> [target_cg] [--json]  Run multi-deck GA + access-network optimizer
  compartments-report <file> [target_cg]  Print HTML general-arrangement report
  calibrate-stats      Show reduced-form and structural-form calibration fits
  import-data <file>   Validate a user-supplied CSV of ship cases
  small-fast <file>    Show small/fast external data error diagnostic
  planing-hp <len> <beam> <disp_lbs> <speed>  Estimate planing hull power (experimental)
  cfd-proxy <hp> <fn> [visc] [wave] [fn_ref]  Apply CFD surrogate correction
  cfd-status <csv>     Report whether a CFD sample set is usable evidence
  cfd-fit <csv>        Fit CFD surrogate correction from samples
  cfd-report <csv>     Generate HTML CFD surrogate report
  cfd-fit-ridge <csv> <lambda>  Ridge-regularized CFD proxy fit
  cfd-select-lambda <csv>  Select CFD Ridge lambda by LOOCV
  cfd-apply <csv> <design.ssr> [min_r2] [force]  Fit CFD proxy and write it to a design

  planing-calibrate <file>  Compute planing-hp calibration factor from external cases
  planing-validate [csv...]  Validate Savitsky vs empirical on planing cases
  savitsky-hp <L> <B> <lbs> <kt> [deadrise] [lcg] [eta]  Savitsky planing estimate
  planing-calibrate-all  Compute combined calibration from example files
  calibrate-external <file>  Fit models on built-in + external data
  external-report <file>  Generate HTML report for external CSV data
  quality-report       Generate HTML report for built-in data quality
  data-priority        List built-in cases by replacement priority
  data-sources         Summarize built-in data by source
  data-score           Show overall built-in data quality score
  data-progress        Show verified/public-reference counts and traceability
  data-tiers           Show built-in data quality tier counts
  data-replace-log     Show suggested replacement actions per case
  data-version         Show built-in data version tag
  data-targets         Check multiple composite-quality targets
  data-target [frac]   Check composite quality gate + per-type (default 0.8)
  apply-ensemble-external <file> <csv>  Apply external-data ensemble weights to a design
  apply-ensemble <file>  Write LOOCV-optimized ensemble weights into a design
  calibrate-ridge <file>  Apply LOOCV-selected Ridge regularized structural fit
  calibrate-lasso <file>  Apply LOOCV-selected Lasso regularized structural fit
  calibrate-structural <file>  Apply structural Holtrop multipliers to a design
  calibrate-structural-segmented <file>  Apply Fn-segmented structural multipliers
  interact [file]      Start an interactive terminal session

Grouped aliases (flat names still work):
  calibrate <stats|regime|type|ridge|lasso|structural|external> ...
  validate  <types|holdout|regime>
  data      <score|progress|target|targets|tiers|priority|sources|replace-log|version>
  regime    <show|apply|validate>
  accuracy  target [mape]
  cfd       <status|proxy|fit|report|fit-ridge|select-lambda|apply>
  planing   <hp|savitsky|validate|calibrate|calibrate-all>

Run without a command to enter interactive mode.
"
    );
}

/// Map grouped invocations (`calibrate stats`, `data score`, ...) onto the
/// canonical flat command names.  Flat names remain supported as aliases.
fn normalize_grouped_command(mut args: Vec<String>) -> Vec<String> {
    if args.len() < 2 {
        return args;
    }
    let canonical: Option<String> = match (args[0].as_str(), args[1].as_str()) {
        (
            "calibrate",
            s @ ("stats"
            | "regime"
            | "type"
            | "ridge"
            | "lasso"
            | "structural"
            | "structural-segmented"
            | "external"),
        ) => Some(format!("calibrate-{s}")),
        (
            "data",
            s @ ("score" | "progress" | "target" | "targets" | "tiers" | "priority" | "sources"
            | "replace-log" | "version"),
        ) => Some(format!("data-{s}")),
        ("validate", s @ ("types" | "holdout")) => Some(format!("validate-{s}")),
        ("accuracy", "target") => Some("accuracy-target".to_string()),
        ("regime", "show") => Some("regime-table".to_string()),
        ("regime", "apply") => Some("calibrate-regime".to_string()),
        ("regime", "validate") => Some("validate-regime".to_string()),
        (
            "cfd",
            s @ ("status" | "proxy" | "fit" | "report" | "fit-ridge" | "select-lambda" | "apply"),
        ) => Some(format!("cfd-{s}")),
        ("planing", "hp") => Some("planing-hp".to_string()),
        ("planing", "calibrate") => Some("planing-calibrate".to_string()),
        ("planing", "calibrate-all") => Some("planing-calibrate-all".to_string()),
        ("planing", "validate") => Some("planing-validate".to_string()),
        ("planing", "savitsky") => Some("savitsky-hp".to_string()),
        _ => None,
    };
    if let Some(c) = canonical {
        let rest = args.split_off(2);
        let mut out = vec![c];
        out.extend(rest);
        args = out;
    }
    args
}

fn main() {
    let args = normalize_grouped_command(std::env::args().skip(1).collect());
    let cmd = args.first().map(String::as_str).unwrap_or("interact");

    match cmd {
        "help" | "--help" | "-h" => print_help(),
        "demo" => {
            let mut ship = demo_ship();
            ship.calculate();
            print!("{}", ship.summary());
        }
        "save" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp save <file>");
                std::process::exit(2);
            };
            let mut ship = demo_ship();
            ship.calculate();
            save_to_file(&ship, path).expect("failed to save design");
            println!("Saved design to {path}");
        }
        "load" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp load <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            ship.calculate();
            print!("{}", ship.summary());
        }
        "report" | "html" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp {cmd} <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            ship.calculate();
            if cmd == "report" {
                print!("{}", generate_report(&ship));
            } else {
                print!("{}", generate_html_report(&ship));
            }
        }
        "solve-speed" => {
            let (Some(path), Some(target)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: springsharp solve-speed <file> <target_hp>");
                std::process::exit(2);
            };
            let target: f64 = target.parse().expect("target HP must be a number");
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            let speed = solve_max_speed(&mut ship, target);
            println!(
                "Solved max speed: {speed:.3} kt at {:.0} hp",
                ship.engine.hp_max
            );
            print!("{}", ship.summary());
        }
        "solve-range" => {
            let (Some(path), Some(target)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: springsharp solve-range <file> <target_bunker_tons>");
                std::process::exit(2);
            };
            let target: f64 = target.parse().expect("target bunker must be a number");
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            let range = solve_range(&mut ship, target);
            println!(
                "Solved range: {range:.0} nm at {:.0} tons normal bunker",
                ship.engine.bunker
            );
            print!("{}", ship.summary());
        }
        "calibrate" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp calibrate <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
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
            ship.engine.power_factor = factor;
            ship.calculate();
            println!("Applied historical calibration factor {factor:.3} to {path}");
            print!("{}", ship.summary());
        }
        "calibrate-structural-segmented" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp calibrate-structural-segmented <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
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
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp calibrate-structural <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
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
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp calibrate-ridge <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            apply_structural_calibration_ridge(&mut ship);
            ship.calculate();
            save_to_file(&ship, path).expect("failed to save ridge-calibrated design");
            println!("Applied Ridge-regularized structural calibration to {path}");
            print!("{}", ship.summary());
        }
        "calibrate-lasso" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp calibrate-lasso <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            apply_structural_calibration_lasso(&mut ship);
            ship.calculate();
            save_to_file(&ship, path).expect("failed to save lasso-calibrated design");
            println!("Applied Lasso-regularized structural calibration to {path}");
            print!("{}", ship.summary());
        }
        "apply-ensemble" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp apply-ensemble <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            apply_ensemble_calibration(&mut ship);
            ship.calculate();
            save_to_file(&ship, path).expect("failed to save ensemble weights");
            println!(
                "Applied LOOCV ensemble weights to {path}: reduced={:.3}, structural={:.3}",
                ship.engine.ensemble_reduced_weight, ship.engine.ensemble_structural_weight
            );
            print!("{}", ship.summary());
        }
        "apply-ensemble-external" => {
            let (Some(path), Some(csv_path)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: springsharp apply-ensemble-external <file> <csv>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(csv_path).expect("failed to read external csv");
            let cases = parse_external_cases(&text).expect("failed to parse external cases");
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            apply_ensemble_calibration_with_external(&mut ship, &cases);
            ship.calculate();
            save_to_file(&ship, path).expect("failed to save design");
            println!(
                "Applied external-data ensemble weights to {path}: reduced={:.3}, structural={:.3}",
                ship.engine.ensemble_reduced_weight, ship.engine.ensemble_structural_weight
            );
            print!("{}", ship.summary());
        }
        "data-target" => {
            let target: f64 = args
                .get(1)
                .map(|x| x.parse().expect("target must be a number"))
                .unwrap_or(0.8);
            let regime = run_loocv_regime();
            let gate = quality_gate_with_loocv(regime.overall_mape);
            println!("Composite data-quality gate (target {target:.2}):");
            println!("  composite             : {:.3}", gate.composite);
            println!(
                "  official/model-test   : {:.1}%",
                gate.official_model_fraction * 100.0
            );
            println!(
                "  provenance traceable  : {:.1}%",
                gate.traceability_fraction * 100.0
            );
            println!(
                "  nested LOOCV MAPE     : {:.1}%  -> accuracy score {:.3}",
                gate.loocv_mape * 100.0,
                gate.loocv_accuracy_score
            );
            println!("  (composite = 0.4*official + 0.2*traceable + 0.4*accuracy)");
            println!("\nPer ship type:");
            println!(
                "{:<12} {:>4} {:>10} {:>12} {:>9} {:>10}",
                "Type", "N", "Official%", "Traceable%", "LOOCV%", "Composite"
            );
            for g in quality_gate_by_type(&regime.per_type) {
                println!(
                    "{:<12} {:>4} {:>9.1}% {:>11.1}% {:>8.1}% {:>10.3}",
                    g.ship_type,
                    g.count,
                    g.official_model_fraction * 100.0,
                    g.traceability_fraction * 100.0,
                    g.loocv_mape * 100.0,
                    g.composite
                );
            }
            if gate.composite < target {
                eprintln!(
                    "Composite quality target not met: {:.3} < {:.3}. Add official trial/model-test data with citation + URL + verifier.",
                    gate.composite, target
                );
                std::process::exit(4);
            }
        }
        "data-targets" => {
            let regime = run_loocv_regime();
            let gate = quality_gate_with_loocv(regime.overall_mape);
            for (target, met) in data_target_matrix_for(gate.composite) {
                println!(
                    "target {:.1}: {}",
                    target,
                    if met { "met" } else { "not met" }
                );
            }
            println!(
                "Note: composite = 0.4*official-model + 0.2*traceable + 0.4*LOOCV-accuracy; current composite={:.3}.",
                gate.composite
            );
        }
        "data-version" => {
            println!("{}", data_version());
        }
        "data-replace-log" => {
            println!("{:>5}  {:<28} Action", "Conf", "Ship");
            for (name, confidence, action) in data_replacement_log() {
                println!("{confidence:>5.2}  {name:<28} {action}");
            }
        }
        "data-tiers" => {
            let tiers = data_tier_summary();
            println!(
                "Data tiers: high={}, medium={}, low={}",
                tiers.high, tiers.medium, tiers.low
            );
        }
        "data-progress" => {
            let p = data_replacement_progress();
            println!(
                "Built-in data: {}/{} independently verified (official trial or model test, {:.1}%), {} public-reference/approximate",
                p.verified,
                p.total,
                p.progress * 100.0,
                p.approximate
            );
            println!(
                "  traceable (citation + URL + verifier): {}/{} ({:.1}%); provenance coverage {:.1}%",
                p.traceable,
                p.total,
                p.traceability * 100.0,
                p.provenance_coverage * 100.0
            );
        }
        "data-score" => {
            let score = data_quality_score();
            println!(
                "Built-in data quality: mean declared confidence={:.3}, mean evidence={:.3}, verified={}/{} ({:.1}%)",
                score.mean_confidence,
                score.mean_evidence,
                score.verified_cases,
                score.total_cases,
                score.verified_fraction * 100.0
            );
            let regime = run_loocv_regime();
            let gate = quality_gate_with_loocv(regime.overall_mape);
            println!(
                "Composite quality gate: {:.3} (official {:.1}%, traceable {:.1}%, nested LOOCV {:.1}% -> accuracy {:.3})",
                gate.composite,
                gate.official_model_fraction * 100.0,
                gate.traceability_fraction * 100.0,
                gate.loocv_mape * 100.0,
                gate.loocv_accuracy_score
            );
        }
        "data-sources" => {
            println!("{:<60} {:>5} {:>6}", "Source", "Count", "Conf");
            for (source, count, conf) in data_source_summary() {
                println!("{source:<60} {count:>5} {conf:>6.2}");
            }
        }
        "data-priority" => {
            println!("{:>5}  {:<28} Source", "Conf", "Ship");
            for entry in data_replacement_priority() {
                println!(
                    "{:>5.2}  {:<28} {}",
                    entry.confidence, entry.name, entry.source
                );
            }
        }
        "quality-report" => {
            print!("{}", data_quality_report_html());
        }
        "external-report" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp external-report <file>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let cases = parse_external_cases(&text).expect("failed to parse external cases");
            print!("{}", external_report_html(&cases));
        }
        "calibrate-external" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp calibrate-external <file>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let cases = parse_external_cases(&text).expect("failed to parse external cases");
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
        "planing-calibrate-all" => {
            let paths = [
                "data/examples/planing_ships.csv",
                "data/examples/more_planing.csv",
                "data/examples/small_ships.csv",
            ];
            let factor = compute_planing_calibration_from_files(&paths);
            println!("Combined planing calibration factor: {factor:.3}");
        }
        "planing-calibrate" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp planing-calibrate <csv> [design.ssr]");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let cases = parse_external_cases(&text).expect("failed to parse external cases");
            let factor = compute_planing_calibration(&cases);
            let eligible = cases
                .iter()
                .filter(|c| is_planing_eligible(c.speed_kt, &c.ship_type))
                .count();
            println!("Planing calibration factor (ref/planing): {factor:.3} based on {eligible} eligible cases");
            if let Some(design) = args.get(2) {
                let mut ship = ShipModel::new();
                load_from_file(&mut ship, design).expect("failed to load design");
                ship.planing_calibration_factor = factor;
                save_to_file(&ship, design).expect("failed to save design");
                println!("Wrote planing.calibration_factor to {design}");
            }
        }
        "cfd-apply" => {
            let (Some(csv), Some(design)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: springsharp cfd-apply <csv> <design.ssr> [min_r2] [force]");
                std::process::exit(2);
            };
            let threshold: f64 = args
                .get(3)
                .map(|x| x.parse().expect("min_r2 must be a number"))
                .unwrap_or(0.5);
            let force = args.get(4).map(|x| x == "force").unwrap_or(false)
                || args.iter().any(|x| x == "--force");
            let text = std::fs::read_to_string(csv).expect("failed to read CSV");
            let samples = parse_cfd_samples(&text).expect("failed to parse CFD samples");
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
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, design).expect("failed to load design");
            ship.engine.cfd_proxy_enabled = cfg.enabled;
            ship.engine.cfd_proxy_viscous = cfg.viscous_correction;
            ship.engine.cfd_proxy_wave = cfg.wave_correction;
            ship.engine.cfd_proxy_fn_ref = cfg.fn_ref;
            save_to_file(&ship, design).expect("failed to save design");
            println!(
                "Applied CFD proxy to {design}: lambda={:.3} viscous={:.4} wave={:.4} fn_ref={:.4} LOOCV_R2={:.4}",
                selection.best_lambda, cfg.viscous_correction, cfg.wave_correction, cfg.fn_ref, cv.r2
            );
        }
        "cfd-select-lambda" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp cfd-select-lambda <csv>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let samples = parse_cfd_samples(&text).expect("failed to parse CFD samples");
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
            let (Some(path), Some(lambda)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: springsharp cfd-fit-ridge <csv> <lambda>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let samples = parse_cfd_samples(&text).expect("failed to parse CFD samples");
            let lambda: f64 = lambda.parse().expect("lambda must be a number");
            let cfg = fit_cfd_proxy_ridge(&samples, lambda);
            println!(
                "CFD ridge fit: viscous={:.4} wave={:.4} fn_ref={:.4} lambda={:.3}",
                cfg.viscous_correction, cfg.wave_correction, cfg.fn_ref, lambda
            );
        }
        "cfd-report" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp cfd-report <csv>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let samples = parse_cfd_samples(&text).expect("failed to parse CFD samples");
            print!("{}", cfd_report_html(&samples));
        }
        "cfd-status" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp cfd-status <csv>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let samples = parse_cfd_samples(&text).expect("failed to parse CFD samples");
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
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp cfd-fit <csv>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let samples = parse_cfd_samples(&text).expect("failed to parse CFD samples");
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
            let (Some(hp), Some(fn_)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: springsharp cfd-proxy <hp> <fn> [visc] [wave] [fn_ref]");
                std::process::exit(2);
            };
            let hp: f64 = hp.parse().expect("hp must be a number");
            let fn_: f64 = fn_.parse().expect("fn must be a number");
            let visc: f64 = args
                .get(3)
                .map(|x| x.parse().expect("visc must be a number"))
                .unwrap_or(0.0);
            let wave: f64 = args
                .get(4)
                .map(|x| x.parse().expect("wave must be a number"))
                .unwrap_or(0.0);
            let fn_ref: f64 = args
                .get(5)
                .map(|x| x.parse().expect("fn_ref must be a number"))
                .unwrap_or(0.25);
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
        "planing-hp" => {
            let (Some(l), Some(b), Some(d), Some(s)) =
                (args.get(1), args.get(2), args.get(3), args.get(4))
            else {
                eprintln!("usage: springsharp planing-hp <length_ft> <beam_ft> <displacement_lbs> <speed_kn> [factor]");
                std::process::exit(2);
            };
            let l: f64 = l.parse().expect("length must be a number");
            let b: f64 = b.parse().expect("beam must be a number");
            let d: f64 = d.parse().expect("displacement must be a number");
            let s: f64 = s.parse().expect("speed must be a number");
            let factor: f64 = args
                .get(5)
                .map(|x| x.parse().expect("factor must be a number"))
                .unwrap_or(1.0);
            let hp = planing_hp_estimate_calibrated(l, b, d, s, factor);
            println!("Experimental planing hull power estimate: {hp:.0} hp (factor={factor:.3})");
        }
        "small-fast" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp small-fast <file>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let cases = parse_external_cases(&text).expect("failed to parse external cases");
            let (count, mape) = small_fast_error_diagnostic(&cases, 0.0);
            println!(
                "Small/fast external cases: count={count}, mean abs pct error={:.1}%",
                mape * 100.0
            );
            if mape > 0.5 {
                println!("Warning: large error indicates planing/semi-planing regime; current Holtrop-style model is not applicable.");
                println!("Try `cargo run -- planing-hp <len> <beam> <disp_lbs> <speed>` for an experimental planing estimate.");
            }
            let eligible_cases: Vec<_> = cases
                .iter()
                .filter(|c| is_planing_eligible(c.speed_kt, &c.ship_type))
                .collect();
            let dyn_factor = compute_planing_calibration(&cases);
            println!("\nExperimental planing-hp estimates for eligible cases (dynamic factor={dyn_factor:.3}):");
            for case in &eligible_cases {
                let hp = planing_hp_estimate_calibrated(
                    case.length_ft,
                    case.beam_ft,
                    case.displacement_tons * 2240.0,
                    case.speed_kt,
                    dyn_factor,
                );
                println!(
                    "  {}: planing(calibrated)={:.0} hp, ref={:.0} hp",
                    case.name, hp, case.reference_hp
                );
            }
        }
        "import-data" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp import-data <file>");
                std::process::exit(2);
            };
            let text = std::fs::read_to_string(path).expect("failed to read file");
            let cases = parse_external_cases(&text).expect("failed to parse external cases");
            let results = validate_external_cases(&cases);
            println!(
                "{:<24} {:>10} {:>10} {:>10}",
                "Ship", "Sim HP", "Ref HP", "Conf"
            );
            for (name, sim, reference, confidence) in results {
                println!(
                    "{:<24} {:>10.0} {:>10.0} {:>9.2}",
                    name, sim, reference, confidence
                );
            }
            println!("Imported {} cases from {path}", cases.len());
            let duplicates = detect_duplicate_external_cases(&cases);
            if duplicates.is_empty() {
                println!("No duplicate ship names detected.");
            } else {
                println!("Warning: duplicate ship names: {}", duplicates.join(", "));
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

        "optimize-compartments" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp optimize-compartments <file> [target_cg] [--json]");
                std::process::exit(2);
            };
            let json = args.iter().any(|a| a == "--json");
            let target_cg: f64 = args
                .iter()
                .skip(1)
                .find_map(|a| a.parse::<f64>().ok())
                .unwrap_or(0.45);
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            ship.calculate();
            let placement = suggest_compartment_placement(&ship, target_cg);
            if json {
                print!("{}", compartment_placement_json(&placement));
            } else {
                println!("{}", format_compartment_report(&placement.layout));
                println!(
                    "Achieved CG ratio: {:.3} (target {target_cg:.3})",
                    placement.cg_vertical
                );
                println!(
                    "Magazine longitudinal position: {:.3} L (from armament layout)",
                    placement.magazine_longitudinal
                );
                println!(
                    "Machinery longitudinal position: {:.3} L (amidships)",
                    placement.machinery_longitudinal
                );
                println!(
                    "Vertical CG: {:.3}  Longitudinal CG: {:.3}  Objective: {:.5}",
                    placement.cg_vertical, placement.cg_longitudinal, placement.objective
                );
                println!(
                    "GA: {} generations, {} evaluations",
                    placement.generations, placement.evaluations
                );
                println!(
                    "Decks: {:?}  Access network: {} edges, connected={}",
                    placement.decks, placement.access_edges, placement.access_connected
                );
                for (name, ok) in &placement.checks {
                    println!("  [{}] {name}", if *ok { "ok" } else { "FAIL" });
                }
            }
        }
        "compartments-report" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp compartments-report <file> [target_cg]");
                std::process::exit(2);
            };
            let target_cg: f64 = args
                .iter()
                .skip(1)
                .find_map(|a| a.parse::<f64>().ok())
                .unwrap_or(0.45);
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            ship.calculate();
            let placement = suggest_compartment_placement(&ship, target_cg);
            print!("{}", compartment_placement_html(&ship, &placement));
        }
        "compartments" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp compartments <file> [--json]");
                std::process::exit(2);
            };
            let json = args.iter().any(|a| a == "--json");
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            ship.calculate();
            let placement = suggest_compartment_placement(&ship, 0.45);
            if json {
                print!("{}", compartment_placement_json(&placement));
                return;
            }
            println!("{}", format_compartment_report(&placement.layout));
            println!(
                "Magazine longitudinal position: {:.3} L (from armament layout)",
                placement.magazine_longitudinal
            );
            println!(
                "Machinery longitudinal position: {:.3} L (amidships)",
                placement.machinery_longitudinal
            );
            println!(
                "Vertical CG: {:.3}  Longitudinal CG: {:.3}  Objective: {:.5}  GA: {} gen / {} eval",
                placement.cg_vertical,
                placement.cg_longitudinal,
                placement.objective,
                placement.generations,
                placement.evaluations
            );
            println!(
                "Decks: {:?}  Access network: {} edges, connected={}",
                placement.decks, placement.access_edges, placement.access_connected
            );
            for (name, ok) in &placement.checks {
                println!("  [{}] {name}", if *ok { "ok" } else { "FAIL" });
            }
        }
        "spaces" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp spaces <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            ship.calculate();
            let report = calculate_internal_spaces(&ship);
            println!("{}", format_space_report(&report));
        }
        "calibrate-type" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp calibrate-type <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
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
            ship.engine.power_factor = factor;
            ship.calculate();
            save_to_file(&ship, path).expect("failed to save design");
            println!(
                "Applied {} calibration factor {:.3} to {path}",
                entry.ship_type, factor
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
        "calibrate-regime" => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: springsharp calibrate-regime <file>");
                std::process::exit(2);
            };
            let mut ship = ShipModel::new();
            load_from_file(&mut ship, path).expect("failed to load design");
            ship.calculate();
            match apply_regime_calibration(&mut ship) {
                Some(entry) => {
                    save_to_file(&ship, path).expect("failed to save design");
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
                "  original + per-type factor: {:.2}%  (comparison baseline)",
                r.original_baseline_mape * 100.0
            );
            println!(
                "  Holtrop uncalibrated       : {:.2}%",
                r.holtrop_uncalibrated_mape * 100.0
            );
            println!("\nPer ship type (regime, nested):");
            for (ty, n, m) in &r.per_type {
                println!("  {:<12} n={:<2} {:.2}%", ty, n, m * 100.0);
            }
        }
        "accuracy-target" => {
            let target: f64 = args
                .get(1)
                .map(|x| x.parse().expect("target must be a number"))
                .unwrap_or(0.10);
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
                eprintln!(
                    "Accuracy target not met: regime nested LOOCV {:.2}% > {:.2}%",
                    r.overall_mape * 100.0,
                    target * 100.0
                );
                std::process::exit(4);
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
        "web" => {
            let port: u16 = args
                .get(1)
                .map(|s| s.parse().expect("port must be a number"))
                .unwrap_or(8080);
            run_server(port).expect("web server failed");
        }
        "savitsky-hp" => {
            let nums: Vec<f64> = args
                .iter()
                .skip(1)
                .filter_map(|a| a.parse::<f64>().ok())
                .collect();
            if nums.len() < 4 {
                eprintln!(
                    "usage: springsharp savitsky-hp <length_ft> <beam_ft> <weight_lbs> <speed_kt> [deadrise_deg] [lcg_ft] [eta]"
                );
                std::process::exit(2);
            }
            let input = PlaningInput {
                length_ft: nums[0],
                beam_ft: nums[1],
                weight_lb: nums[2],
                speed_kt: nums[3],
                deadrise_deg: nums.get(4).copied().unwrap_or(12.0),
                lcg_ft: nums.get(5).copied().unwrap_or(0.42 * nums[0]),
                propulsive_coefficient: nums.get(6).copied().unwrap_or(0.55),
            };
            match savitsky_planing(&input) {
                Some(r) => {
                    println!("Savitsky planing estimate:");
                    println!("  trim: {:.2} deg", r.trim_deg);
                    println!(
                        "  wetted length: {:.1} ft (lambda={:.2})",
                        r.wetted_length_ft, r.wetted_length_ratio
                    );
                    println!("  drag: {:.0} lbf", r.drag_lb);
                    println!("  shaft power: {:.0} hp", r.shaft_hp);
                }
                None => {
                    eprintln!("No planing equilibrium for the given inputs");
                    std::process::exit(3);
                }
            }
        }
        "planing-validate" => {
            let paths: Vec<String> = if args.len() > 1 {
                args[1..].to_vec()
            } else {
                vec![
                    "data/examples/planing_ships.csv".to_string(),
                    "data/examples/more_planing.csv".to_string(),
                ]
            };
            let mut cases = Vec::new();
            for path in &paths {
                let text = std::fs::read_to_string(path).expect("failed to read CSV");
                cases.extend(parse_external_cases(&text).expect("failed to parse CSV"));
            }
            let v = validate_planing_cases(&cases);
            println!(
                "{:<22} {:>10} {:>12} {:>12}",
                "Case", "Ref HP", "Savitsky", "Empirical"
            );
            for r in &v.rows {
                println!(
                    "{:<22} {:>10.0} {:>12.0} {:>12.0}",
                    r.name, r.reference_hp, r.savitsky_hp, r.empirical_hp
                );
            }
            println!("\nEligible planing cases: {}", v.cases);
            println!(
                "Savitsky : raw MAPE={:.1}%  factor={:.3}  calibrated MAPE={:.1}%",
                v.savitsky_raw_mape * 100.0,
                v.savitsky_factor,
                v.savitsky_calibrated_mape * 100.0
            );
            println!(
                "Empirical: raw MAPE={:.1}%  factor={:.3}  calibrated MAPE={:.1}%",
                v.empirical_raw_mape * 100.0,
                v.empirical_factor,
                v.empirical_calibrated_mape * 100.0
            );
            println!(
                "Note: public historical/approximate cases (not basin data); assumptions: deadrise 12-14 deg, LCG=0.42L, eta=0.55."
            );
        }
        "interact" => {
            let mut ship = ShipModel::new();
            if let Some(path) = args.get(1) {
                load_from_file(&mut ship, path).expect("failed to load design");
            } else {
                ship = demo_ship();
            }
            ship.calculate();
            interactive(&mut ship).expect("interactive session failed");
        }
        _ => {
            eprintln!("Unknown command: {cmd}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn interactive(ship: &mut ShipModel) -> io::Result<()> {
    let stdin = io::stdin();
    let lines = stdin.lock().lines();
    println!("SpringSharp Rust interactive session. Type 'help' for commands.");
    print!("> ");
    io::stdout().flush()?;
    for line in lines {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            print!("> ");
            io::stdout().flush()?;
            continue;
        }
        let mut parts = line.split_whitespace();
        match parts.next().unwrap_or_default() {
            "quit" | "exit" => break,
            "help" => {
                println!(
                    "Commands:\n  summary        show design summary\n  report         show full report\n  save <file>    save design\n  load <file>    load design\n  set <key> <value>   set one input field, e.g. hull.length 650\n  solve-speed <hp>    iterate to max speed for target HP\n  solve-range <t>     iterate range for target bunker tons\n  quit"
                );
            }
            "summary" => {
                print!("{}", ship.summary());
            }
            "report" => {
                print!("{}", generate_report(ship));
            }
            "save" => {
                let Some(path) = parts.next() else {
                    println!("usage: save <file>");
                    print!("> ");
                    io::stdout().flush()?;
                    continue;
                };
                save_to_file(ship, path)?;
                println!("Saved to {path}");
            }
            "load" => {
                let Some(path) = parts.next() else {
                    println!("usage: load <file>");
                    print!("> ");
                    io::stdout().flush()?;
                    continue;
                };
                load_from_file(ship, path)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                ship.calculate();
                println!("Loaded {path}");
            }
            "set" => {
                let key = parts.next();
                let value = parts.next();
                let (Some(key), Some(value)) = (key, value) else {
                    println!("usage: set <key> <value>");
                    print!("> ");
                    io::stdout().flush()?;
                    continue;
                };
                let line = format!("{key}={value}\n");
                if let Err(e) = deserialize(ship, &line) {
                    println!("set failed: {e}");
                } else {
                    ship.calculate();
                    println!("Updated {key} = {value}");
                }
            }
            "solve-speed" => {
                let Some(target) = parts.next() else {
                    println!("usage: solve-speed <hp>");
                    print!("> ");
                    io::stdout().flush()?;
                    continue;
                };
                let target: f64 = match target.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("target must be a number");
                        print!("> ");
                        io::stdout().flush()?;
                        continue;
                    }
                };
                let speed = solve_max_speed(ship, target);
                println!(
                    "Solved max speed: {speed:.3} kt at {:.0} hp",
                    ship.engine.hp_max
                );
            }
            "solve-range" => {
                let Some(target) = parts.next() else {
                    println!("usage: solve-range <bunker_tons>");
                    print!("> ");
                    io::stdout().flush()?;
                    continue;
                };
                let target: f64 = match target.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("target must be a number");
                        print!("> ");
                        io::stdout().flush()?;
                        continue;
                    }
                };
                let range = solve_range(ship, target);
                println!(
                    "Solved range: {range:.0} nm at {:.0} tons normal bunker",
                    ship.engine.bunker
                );
            }
            _ => println!("Unknown command. Type 'help'."),
        }
        print!("> ");
        io::stdout().flush()?;
    }
    Ok(())
}
