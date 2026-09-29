# SpringSharp in Rust

This repository contains a Rust reimplementation of the calculation engine from
the decompiled C# WinForms project in `raw/SpringSharpMono`.

The original program is a naval design tool ("SpringSharp").  It lets a user
enter hull dimensions, machinery, guns, armour and secondary weapons and
instantly calculates displacement, weights, strength, stability, seakeeping,
cost, flotation and damage resistance.

## Ported modules

| C# source | Rust equivalent |
|---|---|
| `HullControl.cs` | `HullValues`, `ShipModel::prepare_hull_freeboard` |
| `FreeboardControl.cs` | `FreeboardValues` |
| `GunControl.cs` / `GunsControl.cs` | `GunBattery`, `GunsAggregate` |
| `WeaponControl.cs` / `WeaponsControl.cs` | `WeaponMount`, `WeaponAggregate` |
| `ArmourControl.cs` | `ArmourValues` |
| `EngineControl.cs` | `EngineValues` |
| `PerformanceControl.cs` | `ShipModel::calculate_performance` |
| `MainForm.cs` save/load | `io` module (Rust-native `.ssr` format + legacy SpringSharp 3.0 read) |
| `ReportControl.cs` | `features::generate_report` / `generate_html_report` / `ship_classification` / `gun_description` |
| `EngineControl.SpeedBtn_Click` | `features::solve_max_speed` / `solve_range` |

The WinForms UI itself is not ported, but a command-line/terminal interface and
zero-dependency local web console are provided as working front-ends.

## Project layout

```
springsharp/            # core crate: zero dependencies
├── Cargo.toml          # core workspace (members = ["."], excludes gui/)
├── src/
│   ├── lib.rs
│   ├── main.rs         # CLI + interactive terminal session
│   ├── model.rs        # calculation structures and formulas
│   ├── io.rs           # save/load support
│   ├── features.rs     # reports and iterative solvers
│   ├── accuracy.rs     # historical cases, validation, provenance audits
│   ├── resistance.rs   # Holtrop-Mennen style resistance/power
│   ├── regime.rs       # modern regime model (Holtrop + per-type factor)
│   ├── calibration.rs  # reduced/structural-form calibration
│   ├── spaces.rs       # internal spaces + general-arrangement GA
│   ├── scantling.rs    # rule-style plate/stiffener checks
│   ├── planing.rs      # experimental planing model
│   ├── savitsky.rs     # Savitsky planing solver
│   ├── cfd_proxy.rs    # CFD-surrogate framework
│   └── web.rs          # zero-dependency local web console
├── docs/               # ALGORITHM / VALIDATION / ROADMAP / CHANGELOG
└── gui/                # separate workspace crate (springsharp-gui)
    ├── Cargo.toml      # depends on `springsharp` + eframe
    └── src/{main.rs,app.rs}
```

## Usage

Run the built-in example:

```bash
cargo run -- demo
```

CLI commands:

```bash
# Save/load a design
cargo run -- save example.ssr
cargo run -- load example.ssr

# Reports
cargo run -- report example.ssr
cargo run -- html example.ssr > example.html

# Native egui GUI (optional feature; needs a display/GL environment).
# scripts/build-gui.sh builds it into a separate target-gui/ directory so the
# eframe dependency tree never competes with the core target/.
scripts/build-gui.sh
CARGO_TARGET_DIR=target-gui cargo run --manifest-path gui/Cargo.toml

# Web console (browser GUI with form inputs)
cargo run -- web 8080

# Solvers
cargo run -- solve-speed example.ssr 50000
cargo run -- solve-range example.ssr 1500

# CFD surrogate correction and compartment prototype
cargo run -- cfd-status data/examples/cfd_samples.csv
cargo run -- cfd-status data/examples/cfd_reference.csv
cargo run -- planing-validate
cargo run -- savitsky-hp 80 20 112000 40 12 33.6 0.55
cargo run -- cfd-proxy 49132 0.3 0.1 0.2 0.25
cargo run -- cfd-fit data/examples/cfd_samples.csv
cargo run -- cfd-report data/examples/cfd_samples.csv
cargo run -- cfd-fit-ridge data/examples/cfd_samples.csv 1.0
cargo run -- cfd-select-lambda data/examples/cfd_samples.csv
cargo run -- cfd-apply data/examples/cfd_samples.csv example.ssr 0.8
cargo run -- compartments example.ssr
cargo run -- compartments example.ssr --json
cargo run -- compartments-report example.ssr > ga.html
cargo run -- optimize-compartments example.ssr 0.45 --json

# Accuracy validation / calibration
cargo run -- validate
cargo run -- validate-types
cargo run -- calibrate example.ssr
cargo run -- calibrate-type example.ssr
cargo run -- calibrate-stats
cargo run -- calibrate-structural example.ssr
cargo run -- calibrate-structural-segmented example.ssr
cargo run -- spaces example.ssr

# Interactive terminal session
cargo run -- interact example.ssr
```

In the interactive session you can use:

```text
help
summary
report
save <file>
load <file>
set <key> <value>       e.g. set hull.length 650
solve-speed <hp>
solve-range <bunker tons>
quit
```

Run tests:

```bash
cargo test
```

Run lint:

```bash
cargo clippy -- -D warnings
```

## Save file format

### Rust-native format

`io.rs` writes a simple line-oriented format:

```text
SpringSharp Rust Save 1.0
meta.name=HMS Example
hull.length=600
hull.beam=90
...
```

It stores all input fields from the model.  Load it back with `ShipModel` +
`load_from_file()` before calling `calculate()`.

### Legacy SpringSharp 3.0 format

`load_legacy_springsharp()` can also read the original WinForms 280-line save
format.  `load_from_file()` automatically detects files beginning with
`SpringSharp Version 3.0` and uses the legacy parser.

## Accuracy and calibration

`accuracy.rs` includes 123 historical speed/power cases from battleships, fleet
carriers, cruisers and destroyers (Dreadnought, Bismarck, Iowa, Fletcher,
Yamato, Kongo, Essex, Atlanta, Tribal, South Dakota, KGV, Baltimore, Cleveland,
Yorktown, Illustrious, North Carolina, Vanguard, Shokaku, Hood, Zara, Mogami,
Kagero, Brooklyn, Dido, Somers, Benson, Independence, Bogue, Alaska, Gneisenau,
Porter, Sims, Mahan, New Orleans, Kuma, Sendai, Admiral Hipper, plus 15 additional
carriers (Lexington, Saratoga, Ranger, Wasp, Akagi, Kaga, Soryu, Hiryu, Ark Royal,
Victorious, Graf Zeppelin, Taiho, Unryu, Casablanca, Commencement Bay).

Every case carries explicit provenance (`Provenance { source_type,
displacement_type, confidence, source, source_url, verifier,
verification_date }`).  The current built-in database is **public reference
only** (`SourceType::PublicReference`) and is **not independently verified**:
only official trial or model-basin test data counts as verified.  Official
trial/model-basin data is not available to this project, so the database is
explicitly approximate.

```bash
cargo run -- validate-regime   # primary: nested-LOOCV regime vs baselines
cargo run -- regime-residuals  # per-case historical speed/power residuals
cargo run -- validation-audit  # which model families have historical references
cargo run -- accuracy-target   # check nested regime LOOCV (default <= 10%)
cargo run -- regime-table      # production factors per ship type
cargo run -- validate-holdout  # calibration fit on 28 train / scored on 9 held-out cases
cargo run -- validate          # per-case comparison (in-sample, diagnostic)
cargo run -- validate-types    # grouped by ship type (in-sample, diagnostic)
```

### Modern regime model (default)

For modern accuracy prediction use the **Holtrop-Mennen model with a
per-ship-type calibration factor** fitted to the public historical dataset.
Nested leave-one-out cross-validation (the factor is re-fitted on every training
split) gives:

```text
regime (Holtrop + per-type factor) : 10.40% case-weighted, 10.45% type-averaged
curated core (37 cases)            : 8.09%
original (uncalibrated)            : 21.05%   <- original SpringSharp baseline
original + per-type factor         : 11.16%
Holtrop uncalibrated               : 19.31%
```

Apply it to a design with:

```bash
cargo run -- calibrate-regime example.ssr
```

This sets `engine.resistance_model=holtrop` and `engine.power_factor` for the
design's ship type.  The original SpringSharp model is retained as a comparison
baseline.  In-sample fits are diagnostic only; use `validate-regime` for the
out-of-sample number.

Because all 123 cases are also used by the `calibrate-*` fits, the in-sample
numbers printed by `calibrate-stats` are optimistic.  `calibrate-stats` prints
the LOOCV summary first and labels the in-sample fits as diagnostic only.

The current SpringSharp empirical model tends to over-predict power for these
cases.  The reported calibration factor can be applied to a design with:

```bash
cargo run -- calibrate example.ssr
cargo run -- calibrate-type example.ssr
```

This sets `engine.power_factor` to a Froude-number-band calibration factor and
recalculates the design.  The value is also exposed in the web form and saved in
`.ssr` files.

`resistance::holtrop_power_hp()` provides a Holtrop-Mennen style alternative
with an explicit propulsion-efficiency breakdown (wake fraction, thrust
deduction, hull/open-water/rotative/shaft efficiencies) and a derived QPC.

The **original** SpringSharp model is an empirical shaft-power fit and has no
explicit propulsive coefficient.  Summaries and reports therefore show the QPC
(and state that it is used in the power calculation) only for the Holtrop model;
for the original model they state that no QPC applies.  Earlier versions
incorrectly displayed a Holtrop-style QPC next to original-model results.

Switch a design to Holtrop by setting:

```text
engine.resistance_model=holtrop
```

The `validate` command now prints both the original SpringSharp estimate and the
Holtrop-Mennen estimate, plus Froude-number-band calibration factors.

Additional hull parameters used by the improved resistance model can be set in
the `.ssr` file and web form:

```text
hull.lcb_percent=-0.75
hull.midship_coefficient=0.95
hull.bulb_area_sqft=0
hull.bulb_centre_height_ft=0
hull.transom_area_sqft=0
hull.appendage_area_sqft=0
hull.stern_shape=0
```

If these are left at zero, the engine derives neutral/default values:
- LCB default: `-0.75% L`
- Cm default: `0.90 + 0.10 * Cb`
- Bulb default: `2%` of midship area for ships laid down after 1910
- Transom default: `transom_width * beam * draught` for transom sterns
- Appendage default: `4%` of wetted surface
- Cstern default: `-10` for transom stern, otherwise `0`

## Reduced-form and structural-form calibration

`calibrate-stats` fits two statistical models to the historical dataset:

```bash
cargo run -- calibrate-stats
```

- **Reduced form**: log-space power-law regression:
  `P = exp(b0) * Displacement^b1 * Speed^b2 * Length^b3`
- **Structural form**: Holtrop-Mennen components are decomposed into viscous,
  wave, and correlation terms; viscous/wave multipliers are estimated by least
  squares while correlation is kept at its physical value.

Example output:

```text
Reduced-form power regression ...
  b0=-7.0018  b1=0.5013  b2=3.5395  b3=0.2263  MAPE=9.0%

Structural-form Holtrop component multipliers:
  viscous=2.327  wave=1.288  correlation=1.000  MAPE=13.6%
```

Use `calibrate-structural` to apply the global structural multipliers directly to
a design, or `calibrate-structural-segmented` to apply Fn-segmented multipliers:

```bash
cargo run -- calibrate-structural example.ssr
cargo run -- calibrate-structural-segmented example.ssr
```

`spaces` command shows an improved internal-space volume model parameterized by
ship type, era, and powerplant type:

```bash
cargo run -- spaces example.ssr
```

Example:

```text
Machinery volume/kW: 0.025 m3
Crew volume/person: 12.0 m3
Stores volume/ton: 0.015 m3
```

The volume coefficients can be overridden in `.ssr` / Web form:

```text
spaces.crew_volume_per_person=10
spaces.machinery_volume_per_kw=0.02
spaces.stores_volume_per_ton=0.02
```

Relevant literature pointers are listed in `accuracy::REFERENCES` and
`spaces` module documentation.

The full cumulative implementation log (calibration variants, CLI commands,
CFD/planing work, dataset expansions) lives in `docs/CHANGELOG.md`.

> Note: the built-in database is approximate public historical data, not
> independent verification (official trial / model-basin data is unavailable).
> `validate-regime` and `accuracy-target` are the primary accuracy evidence;
> `data-target` is a separate provenance gate (official/model-test coverage +
> traceability + accuracy) and correctly fails while the data is public-reference
> only.  See `docs/VALIDATION.md` for the frozen baselines.

## Algorithmic improvements (CLRS-inspired)

- Least-squares fitting now uses Householder QR decomposition instead of normal
  equations, improving numerical stability for the dummy-variable and segmented
  structural regressions.
- Speed/range solvers use tolerance-terminated binary search instead of a fixed
  loop count:
  - max speed: stop when interval width `< 1e-4 kt`
  - range: stop when interval width `< 1e-3 nm`
- Structural-form calibration is directly integrated into the engine through
  `engine.holtrop_viscous_factor`, `engine.holtrop_wave_factor`, and
  `engine.holtrop_correlation_factor`.

## Output annotation

- When `engine.resistance_model=original`, summaries and reports explicitly state:
  `power uses the original SpringSharp empirical model (reconstructed, unimproved)`.
- When `engine.resistance_model=holtrop`, summaries and reports state:
  `power uses the improved Holtrop-Mennen-style model`.

This prevents the reconstructed original estimate from being mistaken for an
improved, calibrated result.

## Units

As in the original program, the engine works in Imperial units internally:

- length: feet
- gun calibre: inches
- armour: inches
- weight: long tons (2240 lb)
- displacement: long tons

Metric conversions (`Conversion` / `metric_calc`) are provided for reports.

## Documentation

- `docs/ALGORITHM.md` describes the current algorithm chain.
- `docs/VALIDATION.md` defines provenance, LOOCV/holdout baselines and the composite quality gate.
- `docs/ROADMAP.md` lists completed work and remaining items.
- `docs/CHANGELOG.md` is the cumulative implementation log.
- Native GUI uses eframe/egui behind the optional `gui` feature:
  `scripts/build-gui.sh` (or `cargo run --manifest-path gui/Cargo.toml`).
  It builds headless but needs a display/GL environment at runtime.  Because the
  GUI is a **separate workspace** (`gui/` is excluded from the core workspace),
  the core `cargo build`/`test` never resolves eframe at all — verified by
  building with an empty cargo cache.  `scripts/build-gui.sh` puts the GUI in
  `target-gui/`, and `[profile.dev.package."*"] debug = false` (in `gui/Cargo.toml`)
  keeps those artifacts smaller.  Measured: core `target/` ~250 MB (or ~60 MB
  after `cargo clean -p springsharp`) with a ~2-15 s rebuild; GUI `target-gui/`
  ~740 MB and removable with `rm -rf target-gui`.

## Design notes

CI runs `fmt`, `clippy -D warnings`, `test` and the optional GUI build (see
`.github/workflows/ci.yml`).

- The C# code uses many mutually recursive updates through UI event handlers.
  The Rust model resolves the same dependency graph by running the
  armour/machinery weight cycle to a fixed point.
- Areas where the decompiled C# has obvious typo-like behaviour are replicated
  as faithfully as practical.
- The graphical WinForms code is left out; the terminal CLI and web console are the
  current front-ends. The web console now uses structured form inputs instead of
  a raw `.ssr` textarea, while an optional raw editor could be added later.
