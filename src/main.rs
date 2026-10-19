mod cli;
use cli::CliError;
use springsharp::ShipSummary;

use std::io::{self, BufRead, Write};

use springsharp::{
    demo_ship, deserialize, generate_report, load_from_file, save_to_file, solve_max_speed,
    solve_range, ShipModel,
};

fn main() -> std::process::ExitCode {
    let args = cli::normalize_grouped_command(std::env::args().skip(1).collect());
    match run_cli(&args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::ExitCode::from(e.code as u8)
        }
    }
}

fn run_cli(args: &[String]) -> Result<(), cli::CliError> {
    let cmd = args.first().map(String::as_str).unwrap_or("interact");

    match cmd {
        "help" | "--help" | "-h" => cli::print_help(),
        "demo" => crate::cli::design::run(cmd, args)?,
        "save" => crate::cli::design::run(cmd, args)?,
        "load" => crate::cli::design::run(cmd, args)?,
        "report" => crate::cli::design::run(cmd, args)?,
        "html" => crate::cli::design::run(cmd, args)?,
        "solve-speed" => crate::cli::design::run(cmd, args)?,
        "solve-range" => crate::cli::design::run(cmd, args)?,
        "calibrate" => crate::cli::calibration::run(cmd, args)?,
        "calibrate-structural-segmented" => crate::cli::calibration::run(cmd, args)?,
        "calibrate-structural" => crate::cli::calibration::run(cmd, args)?,
        "calibrate-ridge" => crate::cli::calibration::run(cmd, args)?,
        "calibrate-lasso" => crate::cli::calibration::run(cmd, args)?,
        "apply-ensemble" => crate::cli::calibration::run(cmd, args)?,
        "apply-ensemble-external" => crate::cli::calibration::run(cmd, args)?,
        "data-target" => crate::cli::data::run(cmd, args)?,
        "data-targets" => crate::cli::data::run(cmd, args)?,
        "data-version" => crate::cli::data::run(cmd, args)?,
        "data-replace-log" => crate::cli::data::run(cmd, args)?,
        "data-consistency" => crate::cli::data::run(cmd, args)?,
        "data-outliers" => crate::cli::data::run(cmd, args)?,
        "data-tiers" => crate::cli::data::run(cmd, args)?,
        "data-progress" => crate::cli::data::run(cmd, args)?,
        "data-score" => crate::cli::data::run(cmd, args)?,
        "data-sources" => crate::cli::data::run(cmd, args)?,
        "data-priority" => crate::cli::data::run(cmd, args)?,
        "quality-report" => crate::cli::data::run(cmd, args)?,
        "external-report" => crate::cli::data::run(cmd, args)?,
        "calibrate-external" => crate::cli::calibration::run(cmd, args)?,
        "planing-calibrate-all" => crate::cli::planing::run(cmd, args)?,
        "planing-calibrate" => crate::cli::planing::run(cmd, args)?,
        "cfd-apply" => crate::cli::cfd::run(cmd, args)?,
        "cfd-select-lambda" => crate::cli::cfd::run(cmd, args)?,
        "cfd-fit-ridge" => crate::cli::cfd::run(cmd, args)?,
        "cfd-report" => crate::cli::cfd::run(cmd, args)?,
        "cfd-status" => crate::cli::cfd::run(cmd, args)?,
        "cfd-fit" => crate::cli::cfd::run(cmd, args)?,
        "cfd-proxy" => crate::cli::cfd::run(cmd, args)?,
        "planing-hp" => crate::cli::planing::run(cmd, args)?,
        "small-fast" => crate::cli::data::run(cmd, args)?,
        "import-data" => crate::cli::data::run(cmd, args)?,
        "calibrate-stats" => crate::cli::calibration::run(cmd, args)?,
        "optimize-compartments" => crate::cli::spaces::run(cmd, args)?,
        "compartments-report" => crate::cli::spaces::run(cmd, args)?,
        "compartments" => crate::cli::spaces::run(cmd, args)?,
        "spaces" => crate::cli::spaces::run(cmd, args)?,
        "calibrate-type" => crate::cli::calibration::run(cmd, args)?,
        "regime-table" => crate::cli::validation::run(cmd, args)?,
        "calibrate-regime" => crate::cli::calibration::run(cmd, args)?,
        "validate-regime" => crate::cli::validation::run(cmd, args)?,
        "accuracy-target" => crate::cli::validation::run(cmd, args)?,
        "validate-types" => crate::cli::validation::run(cmd, args)?,
        "validate-holdout" => crate::cli::validation::run(cmd, args)?,
        "validate-weights" => crate::cli::validation::run(cmd, args)?,
        "validate" => crate::cli::validation::run(cmd, args)?,
        "web" => crate::cli::web::run(cmd, args)?,
        "savitsky-hp" => crate::cli::planing::run(cmd, args)?,
        "planing-validate" => crate::cli::planing::run(cmd, args)?,
        "blend-table" => crate::cli::calibration::run(cmd, args)?,
        "calibrate-blend" => crate::cli::calibration::run(cmd, args)?,
        "validate-blend" => crate::cli::validation::run(cmd, args)?,
        "regime-compare" => crate::cli::validation::run(cmd, args)?,
        "regime-residuals" => crate::cli::validation::run(cmd, args)?,
        "validation-audit" => crate::cli::data::run(cmd, args)?,
        "historical-compare" => crate::cli::data::run(cmd, args)?,
        "interact" => {
            // `load_ship` returns the populated model; discarding it left the
            // session on an empty `ShipModel::new()` even when a file was given.
            let mut ship = match args.get(1) {
                Some(path) => cli::load_ship(path)?,
                None => demo_ship(),
            };
            ship.calculate();
            interactive(&mut ship)
                .map_err(|e| CliError::runtime(format!("interactive session failed: {e}")))?;
        }
        "version" | "--version" | "-V" => {
            println!("springsharp {}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Unknown command: {cmd}");
            cli::print_help();
            return Err(CliError::usage(format!("unknown command '{cmd}'")));
        }
    }
    Ok(())
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
