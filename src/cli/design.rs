//! `design` command handlers.

use super::{load_ship, parse_f64_arg, require_arg, save_ship, CliError};
use springsharp::*;

pub(crate) fn run(cmd: &str, args: &[String]) -> Result<(), CliError> {
    match cmd {
        "demo" => {
            let mut ship = demo_ship();
            ship.calculate();
            print!("{}", ship.summary());
        }
        "save" => {
            let path = require_arg(args, 1, "save <file>")?;
            let mut ship = demo_ship();
            ship.calculate();
            save_ship(&ship, path)?;
            println!("Saved design to {path}");
        }
        "load" => {
            let path = require_arg(args, 1, "load <file>")?;
            let mut ship = load_ship(path)?;
            ship.calculate();
            print!("{}", ship.summary());
        }
        "report" | "html" => {
            let path = require_arg(args, 1, &format!("{cmd} <file>"))?;
            let mut ship = load_ship(path)?;
            ship.calculate();
            if cmd == "report" {
                print!("{}", generate_report(&ship));
            } else {
                print!("{}", generate_html_report(&ship));
            }
        }
        "solve-speed" => {
            let path = require_arg(args, 1, "solve-speed <file> <target_hp>")?;
            let target = require_arg(args, 2, "solve-speed <file> <target_hp>")?;
            let target: f64 = parse_f64_arg(target, "target HP")?;
            let mut ship = load_ship(path)?;
            let speed = solve_max_speed(&mut ship, target);
            println!(
                "Solved max speed: {speed:.3} kt at {:.0} hp",
                ship.engine.hp_max
            );
            print!("{}", ship.summary());
        }
        "solve-range" => {
            let path = require_arg(args, 1, "solve-range <file> <target_bunker_tons>")?;
            let target = require_arg(args, 2, "solve-range <file> <target_bunker_tons>")?;
            let target: f64 = parse_f64_arg(target, "target bunker")?;
            let mut ship = load_ship(path)?;
            let range = solve_range(&mut ship, target);
            println!(
                "Solved range: {range:.0} nm at {:.0} tons normal bunker",
                ship.engine.bunker
            );
            print!("{}", ship.summary());
        }
        _ => unreachable!("design::run called with unexpected command {cmd}"),
    }
    Ok(())
}
