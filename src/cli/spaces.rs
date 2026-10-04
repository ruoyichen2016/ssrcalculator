//! `spaces` command handlers.

use super::{load_ship, require_arg, CliError};
use springsharp::*;

pub(crate) fn run(cmd: &str, args: &[String]) -> Result<(), CliError> {
    match cmd {
        "optimize-compartments" => {
            let path = require_arg(args, 1, "optimize-compartments <file> [target_cg] [--json]")?;
            let json = args.iter().any(|a| a == "--json");
            let target_cg: f64 = args
                .iter()
                .skip(1)
                .find_map(|a| a.parse::<f64>().ok())
                .unwrap_or(0.45);
            let mut ship = load_ship(path)?;
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
                println!(
                    "Escape: max {} hops / {:.1} m   Structural-weight estimate: {:.0} t",
                    placement.escape_max_hops,
                    placement.escape_max_path,
                    placement.structure_weight_tons
                );
                println!(
                    "IMO evacuation: network {:.0} persons / {} s, congestion {:.0} s, assembly at {:.2} L, density {:.2} p/m2",
                    placement.network_evacuation_capacity,
                    MAX_EVAC_TIME_S as u32,
                    placement.congestion_time_s,
                    placement.assembly_x,
                    placement.max_edge_density
                );
                println!(
                    "Rule scantlings: max utilization {:.2} (<= 1 ok)",
                    placement.scantling_utilization
                );
                for (name, ok) in &placement.checks {
                    println!("  [{}] {name}", if *ok { "ok" } else { "FAIL" });
                }
            }
        }
        "compartments-report" => {
            let path = require_arg(args, 1, "compartments-report <file> [target_cg]")?;
            let target_cg: f64 = args
                .iter()
                .skip(1)
                .find_map(|a| a.parse::<f64>().ok())
                .unwrap_or(0.45);
            let mut ship = load_ship(path)?;
            ship.calculate();
            let placement = suggest_compartment_placement(&ship, target_cg);
            print!("{}", compartment_placement_html(&ship, &placement));
        }
        "compartments" => {
            let path = require_arg(args, 1, "compartments <file> [--json]")?;
            let json = args.iter().any(|a| a == "--json");
            let mut ship = load_ship(path)?;
            ship.calculate();
            let placement = suggest_compartment_placement(&ship, 0.45);
            if json {
                print!("{}", compartment_placement_json(&placement));
                return Ok(());
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
            println!(
                "Escape: max {} hops / {:.1} m   Structural-weight estimate: {:.0} t",
                placement.escape_max_hops,
                placement.escape_max_path,
                placement.structure_weight_tons
            );
            println!(
                "IMO evacuation: network {:.0} persons / {} s, congestion {:.0} s, assembly at {:.2} L, density {:.2} p/m2",
                placement.network_evacuation_capacity,
                MAX_EVAC_TIME_S as u32,
                placement.congestion_time_s,
                placement.assembly_x,
                placement.max_edge_density
            );
            println!(
                "Rule scantlings: max utilization {:.2} (<= 1 ok)",
                placement.scantling_utilization
            );
            for (name, ok) in &placement.checks {
                println!("  [{}] {name}", if *ok { "ok" } else { "FAIL" });
            }
        }
        "spaces" => {
            let path = require_arg(args, 1, "spaces <file>")?;
            let mut ship = load_ship(path)?;
            ship.calculate();
            let report = calculate_internal_spaces(&ship);
            println!("{}", format_space_report(&report));
        }
        _ => unreachable!("spaces::run called with unexpected command {cmd}"),
    }
    Ok(())
}
