//! `data` command handlers.

use super::{load_ship, opt_f64_arg, read_file, require_arg, CliError};
use springsharp::*;

pub(crate) fn run(cmd: &str, args: &[String]) -> Result<(), CliError> {
    match cmd {
        "data-target" => {
            let target: f64 = opt_f64_arg(args, 1, "target", 0.8)?;
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
                return Err(CliError::gate(format!(
                    "composite quality target not met: {:.3} < {:.3}. \
    Add official trial/model-test data with citation + URL + verifier.",
                    gate.composite, target
                )));
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
        "data-consistency" => {
            let mut rows = data_consistency_audit();
            rows.sort_by(|a, b| {
                (b.ratio - 1.0)
                    .abs()
                    .partial_cmp(&(a.ratio - 1.0).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let bad = rows.iter().filter(|r| !r.plausible).count();
            println!("Physical consistency (modeled = L*B*T*Cb/35 vs documented displacement):");
            println!(
                "  {} / {} cases outside the full-load band [0.95, 1.60]",
                bad,
                rows.len()
            );
            for r in rows.iter().filter(|r| !r.plausible) {
                println!(
                    "  {:<30} {:<11} documented={:>7.0}  modeled={:>7.0}  ratio={:.2}",
                    r.name, r.ship_type, r.documented_tons, r.modeled_tons, r.ratio
                );
            }
            println!("  Note: ratios >1 are expected (documented standard vs modeled full load);");
            println!(
                "  values above 1.6 usually mean overall length / flight-deck beam / full-load"
            );
            println!("  draught were mixed with a standard displacement.");
        }
        "data-outliers" => {
            let rows = data_outlier_audit();
            let flagged: Vec<_> = rows.iter().filter(|r| r.flagged).collect();
            println!(
                "Outlier screen: {} / {} cases flagged (>2.5 sigma from the curated power law)",
                flagged.len(),
                rows.len()
            );
            for r in &flagged {
                println!(
                    "  {:<30} {:<11} conf={:.2}  z={:.2}",
                    r.name, r.ship_type, r.confidence, r.z_score
                );
            }
            println!("  Note: flags are data-error candidates (design vs trial values, wrong");
            println!("  dimensions/displacement), not proof of error.");
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
            let path = require_arg(args, 1, "external-report <file>")?;
            let text = read_file(path)?;
            let cases = parse_external_cases(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse external cases: {e}")))?;
            print!("{}", external_report_html(&cases));
        }
        "import-data" => {
            let path = require_arg(args, 1, "import-data <file>")?;
            let text = read_file(path)?;
            let cases = parse_external_cases(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse external cases: {e}")))?;
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
        "small-fast" => {
            let path = require_arg(args, 1, "small-fast <file>")?;
            let text = read_file(path)?;
            let cases = parse_external_cases(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse external cases: {e}")))?;
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
        "validation-audit" => {
            let a = historical_validation_audit();
            println!("Historical data coverage by model family:");
            println!("  cases: {}", a.case_count);
            println!("  speed/power references: {}", a.speed_power_cases);
            println!(
                "  structural-weight references: {}",
                a.structural_weight_cases
            );
            println!("  internal-space references: {}", a.space_cases);
            println!("  general-arrangement references: {}", a.arrangement_cases);
            println!(
                "  non-physical hull weight in simulated cases: {}/{}",
                a.nonphysical_hull_weight_cases, a.case_count
            );
            println!(
                "  zero magazine volume in simulated cases: {}/{}",
                a.zero_magazine_cases, a.case_count
            );
            for note in &a.notes {
                println!("  - {note}");
            }
        }
        "historical-compare" => {
            let path = require_arg(args, 1, "historical-compare <file> [n]")?;
            let max: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
            let mut ship = load_ship(path)?;
            ship.calculate();
            println!(
                "Design: {} ({}, {:.0} t, {:.1} kt, {:.0} ft)",
                ship.name,
                ship.ship_type,
                ship.hull.displacement,
                ship.engine.speed_max,
                ship.hull.length
            );
            println!(
                "{:<30} {:<11} {:>8} {:>7} {:>9} {:>7}",
                "Historical case", "Type", "Disp(t)", "Speed", "Ref HP", "Dist"
            );
            for c in nearest_historical_cases(&ship, max) {
                println!(
                    "{:<30} {:<11} {:>8.0} {:>6.1} {:>9.0} {:>7.2}",
                    c.name,
                    c.ship_type,
                    c.displacement_tons,
                    c.speed_kt,
                    c.reference_hp,
                    c.distance
                );
            }
            println!(
                "Note: public historical references (approximate), for context only - not validation."
            );
        }
        _ => unreachable!("data::run called with unexpected command {cmd}"),
    }
    Ok(())
}
