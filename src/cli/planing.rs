//! `planing` command handlers.

use super::{load_ship, opt_f64_arg, parse_f64_arg, read_file, require_arg, save_ship, CliError};
use springsharp::*;

pub(crate) fn run(cmd: &str, args: &[String]) -> Result<(), CliError> {
    match cmd {
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
            let path = require_arg(args, 1, "planing-calibrate <csv> [design.ssr]")?;
            let text = read_file(path)?;
            let cases = parse_external_cases(&text)
                .map_err(|e| CliError::runtime(format!("failed to parse external cases: {e}")))?;
            let factor = compute_planing_calibration(&cases);
            let eligible = cases
                .iter()
                .filter(|c| is_planing_eligible(c.speed_kt, &c.ship_type))
                .count();
            println!("Planing calibration factor (ref/planing): {factor:.3} based on {eligible} eligible cases");
            if let Some(design) = args.get(2) {
                let mut ship = load_ship(design)?;
                ship.planing_calibration_factor = factor;
                save_ship(&ship, design)?;
                println!("Wrote planing.calibration_factor to {design}");
            }
        }
        "planing-hp" => {
            let l = require_arg(
                args,
                1,
                "planing-hp <length_ft> <beam_ft> <displacement_lbs> <speed_kn> [factor]",
            )?;
            let b = require_arg(
                args,
                2,
                "planing-hp <length_ft> <beam_ft> <displacement_lbs> <speed_kn> [factor]",
            )?;
            let d = require_arg(
                args,
                3,
                "planing-hp <length_ft> <beam_ft> <displacement_lbs> <speed_kn> [factor]",
            )?;
            let s = require_arg(
                args,
                4,
                "planing-hp <length_ft> <beam_ft> <displacement_lbs> <speed_kn> [factor]",
            )?;
            let l: f64 = parse_f64_arg(l, "length")?;
            let b: f64 = parse_f64_arg(b, "beam")?;
            let d: f64 = parse_f64_arg(d, "displacement")?;
            let s: f64 = parse_f64_arg(s, "speed")?;
            let factor: f64 = opt_f64_arg(args, 5, "factor", 1.0)?;
            let hp = planing_hp_estimate_calibrated(l, b, d, s, factor);
            println!("Experimental planing hull power estimate: {hp:.0} hp (factor={factor:.3})");
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
                let text = read_file(path)?;
                cases.extend(
                    parse_external_cases(&text)
                        .map_err(|e| CliError::runtime(format!("failed to parse CSV: {e}")))?,
                );
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
        _ => unreachable!("planing::run called with unexpected command {cmd}"),
    }
    Ok(())
}
