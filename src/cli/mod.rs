//! Command-line interface: argument helpers, help text and the dispatcher.
//!
//! The command bodies are grouped by theme into submodules so `main.rs` stays a
//! thin entry point.  Each submodule exposes `run(cmd, args)` and dispatches on
//! the canonical (already group-normalised) command name.

pub(crate) mod calibration;
pub(crate) mod cfd;
pub(crate) mod data;
pub(crate) mod design;
pub(crate) mod planing;
pub(crate) mod spaces;
pub(crate) mod validation;
pub(crate) mod web;

use springsharp::{load_from_file, save_to_file, ShipModel};

pub(crate) struct CliError {
    pub(crate) message: String,
    pub(crate) code: i32,
}

impl CliError {
    pub(crate) fn usage(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            code: 2,
        }
    }

    pub(crate) fn runtime(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            code: 1,
        }
    }

    /// A quality/accuracy gate that ran successfully but was not met.
    pub(crate) fn gate(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            code: 4,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Parse a required positional argument or return a usage error.
pub(crate) fn require_arg<'a>(
    args: &'a [String],
    idx: usize,
    usage: &str,
) -> Result<&'a str, CliError> {
    args.get(idx)
        .map(String::as_str)
        .ok_or_else(|| CliError::usage(format!("usage: springsharp {usage}")))
}

/// Parse a floating-point argument with a field name for the error message.
pub(crate) fn parse_f64_arg(value: &str, what: &str) -> Result<f64, CliError> {
    let parsed: f64 = value
        .parse()
        .map_err(|_| CliError::usage(format!("{what} must be a number, got '{value}'")))?;
    if !parsed.is_finite() {
        return Err(CliError::usage(format!(
            "{what} must be finite, got '{value}'"
        )));
    }
    Ok(parsed)
}

/// Parse an optional positional float argument, falling back to `default`.
///
/// Unlike `.map(...).unwrap_or(default)` this distinguishes "absent" from
/// "present but malformed" instead of panicking on the latter.
pub(crate) fn opt_f64_arg(
    args: &[String],
    idx: usize,
    what: &str,
    default: f64,
) -> Result<f64, CliError> {
    match args.get(idx) {
        None => Ok(default),
        Some(v) => parse_f64_arg(v, what),
    }
}

pub(crate) fn opt_u16_arg(
    args: &[String],
    idx: usize,
    what: &str,
    default: u16,
) -> Result<u16, CliError> {
    match args.get(idx) {
        None => Ok(default),
        Some(v) => v
            .parse()
            .map_err(|_| CliError::usage(format!("{what} must be a number, got '{v}'"))),
    }
}

pub(crate) fn load_ship(path: &str) -> Result<ShipModel, CliError> {
    let mut ship = ShipModel::new();
    load_from_file(&mut ship, path)
        .map_err(|e| CliError::runtime(format!("failed to load '{path}': {e}")))?;
    Ok(ship)
}

pub(crate) fn save_ship(ship: &ShipModel, path: &str) -> Result<(), CliError> {
    save_to_file(ship, path).map_err(|e| CliError::runtime(format!("failed to save '{path}': {e}")))
}

pub(crate) fn read_file(path: &str) -> Result<String, CliError> {
    std::fs::read_to_string(path)
        .map_err(|e| CliError::runtime(format!("failed to read '{path}': {e}")))
}

pub(crate) fn print_help() {
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
  validate-weights     Compare subsystem weights & GM on the 10 full-ship presets
  regime-residuals     Per-case nested-LOOCV residuals (historical speed/power)
  regime-compare       Paired comparison: regime vs original SpringSharp baseline
  validation-audit     Historical data coverage by model family
  historical-compare <file> [n]  Show historical cases closest to a design
  accuracy-target [m]  Check nested regime LOOCV against a MAPE target (default 0.10)
  regime-table         Show production regime table (Holtrop + per-ship-type factor)
  calibrate-regime <file>  Apply Holtrop + per-ship-type factor (modern default)
  calibrate-blend <file>   Apply blended Holtrop/original model (per-type weights)
  blend-table              Show per-type blend weights and factors
  validate-blend           Nested-LOOCV error of the blended model
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
  data-consistency     Check displacement/dimension consistency
  data-outliers        Screen approximate cases against the curated power law
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
  validate  <types|holdout|regime|weights|audit>
  data      <score|progress|target|targets|tiers|priority|sources|replace-log|version|consistency|outliers>
  regime    <show|apply|validate|residuals|compare>
  accuracy  target [mape]
  cfd       <status|proxy|fit|report|fit-ridge|select-lambda|apply>
  planing   <hp|savitsky|validate|calibrate|calibrate-all>

Run without a command to enter interactive mode.
"
    );
}

/// Map grouped invocations (`calibrate stats`, `data score`, ...) onto the
/// canonical flat command names.  Flat names remain supported as aliases.
pub(crate) fn normalize_grouped_command(mut args: Vec<String>) -> Vec<String> {
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
        ("validate", s @ ("types" | "holdout" | "weights")) => Some(format!("validate-{s}")),
        ("validate", "audit") => Some("validation-audit".to_string()),
        ("accuracy", "target") => Some("accuracy-target".to_string()),
        ("regime", "show") => Some("regime-table".to_string()),
        ("regime", "apply") => Some("calibrate-regime".to_string()),
        ("regime", "validate") => Some("validate-regime".to_string()),
        ("regime", "residuals") => Some("regime-residuals".to_string()),
        ("regime", "compare") => Some("regime-compare".to_string()),
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
