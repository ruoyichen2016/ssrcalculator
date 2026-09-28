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
springsharp/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs        # library entry point
    ├── main.rs       # CLI + interactive terminal session
    ├── model.rs      # all calculation structures and formulas
    ├── io.rs         # save/load support
    ├── features.rs   # report generation and iterative solvers
    ├── accuracy.rs   # historical validation / segmented power calibration
    ├── resistance.rs # Holtrop-Mennen style resistance/power prediction
    ├── spaces.rs     # internal space + compartment prototype
    ├── cfd_proxy.rs  # optional CFD-surrogate correction framework
    ├── calibration.rs # reduced-form and structural-form statistical calibration
    ├── web.rs        # zero-dependency local web console
    └── bin/
        └── gui.rs    # optional native egui GUI
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

# Native egui GUI (optional feature; needs a display/GL environment)
cargo run --features gui --bin springsharp-gui

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

`accuracy.rs` includes 52 historical speed/power cases from battleships, fleet
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
regime (Holtrop + per-type factor) : 8.92% case-weighted, 8.39% type-averaged
original + per-type factor         : 11.53%   <- comparison baseline
Holtrop uncalibrated               : 22.24%
```

Apply it to a design with:

```bash
cargo run -- calibrate-regime example.ssr
```

This sets `engine.resistance_model=holtrop` and `engine.power_factor` for the
design's ship type.  The original SpringSharp model is retained as a comparison
baseline.  In-sample fits are diagnostic only; use `validate-regime` for the
out-of-sample number.

Because all 52 cases are also used by the `calibrate-*` fits, the in-sample
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

The `calibrate-stats` command also reports:

- QR vs SVD cross-check
- reduced-form regression with ship-type dummy variables
- reduced-form with Froude x type interactions
- Fn-segmented structural-form multipliers
- Ridge / Lasso regularized Fn-segmented structural multipliers
- LOOCV report with per-sample residual table
- Ridge / Lasso lambda selection by LOOCV
- inverse-error weighted ensemble prediction
- ensemble weights optimized by LOOCV
- bootstrap 5%-50%-95% uncertainty intervals (parallel using std threads)
- write LOOCV-optimized ensemble weights into `.ssr` via `apply-ensemble`
- data-quality/confidence metadata for the historical database
- confidence-weighted reduced/structural fits
- confidence-weighted LOOCV and bootstrap intervals
- confidence-weighted Fn-segmented structural multipliers
- `calibrate-ridge` / `calibrate-lasso` commands using LOOCV-selected lambdas
- `import-data` command for user-supplied CSV ship cases
- `calibrate-external` command to fit models on built-in + external data
- external-data LOOCV and bootstrap uncertainty in `calibrate-external`
- confidence-weighted external-data LOOCV and bootstrap in `calibrate-external`
- external-data source grouping statistics in `calibrate-external`
- external-data ship-type grouping statistics in `calibrate-external`
- external CSV boundary tests (malformed lines, non-numeric fields, comments/header)
- external duplicate ship-name detection in `import-data`
- Web console external CSV validation endpoint
- Web console external-data calibration endpoint
- Web console `/api/planing` and `/api/planing-calibrate` endpoints
- Web console planing calibrate button auto-fills factor
- `planing.calibration_factor` stored in `.ssr` / Web form
- `planing-calibrate` can write factor to a design file
- engine auto-uses planing model for small fast craft when Holtrop selected
- planing auto-switch thresholds configurable (`.ssr` / Web form)
- planing threshold preservation test
- save/load preserves planing thresholds
- planing calibration empty fallback test
- additional `more_planing.csv` dataset included in example parse test
- `planing-calibrate-all` command learns combined factor from multiple example files
- optional CFD-surrogate correction framework (`cfd_proxy`)
- Propulsion efficiency breakdown in summaries/reports
- CFD sample CSV fitting (`cfd-fit`) with R²/RMSE, LOOCV, holdout and holdout stability
- HTML CFD surrogate report (`cfd-report`)
- Ridge-regularized CFD proxy fit (`cfd-fit-ridge`)
- CFD Ridge lambda selection by LOOCV (`cfd-select-lambda`)
- Apply fitted CFD proxy into `.ssr` (`cfd-apply`) with LOOCV R² threshold check
- `cfd-status` command and a 20-sample minimum guard: the bundled `cfd_samples.csv` is synthetic (9 rows) and `cfd-apply` refuses it unless `force` is given
- multi-deck genetic-algorithm general-arrangement optimizer with an access network (`compartments` / `optimize-compartments`)
- `compartments` includes magazine/machinery longitudinal hints
- `compartments --json` machine-readable output and `compartments-report` general-arrangement HTML report
- `optimize-compartments` prints constraint check results (placement, void, continuity)
- constraint-satisfaction unit test for the compartment optimizer
- GA optimizes volumes, vertical centres and longitudinal positions with feasibility repair
  - optimizes both compartment volumes and vertical centers
  - enforces magazine/accommodation/machinery vertical placement bounds
  - adds access/void and fuel/machinery placement penalties
  - couples magazine longitudinal position to main-battery layout, deck lengths, and G1/G2 mount counts
  - scales magazine volume by calibre, rounds, and mount count
  - adds structural-continuity penalty across nine adjacent compartment pairs
  - enforces a minimum void/maintenance volume (2% hull volume)
- default planing calibration factor updated from combined datasets (0.738)
- Web console planing estimator form
- Web console external-data HTML report
- web route unit tests for health/calculate/external/calibrate endpoints
- `quality-report` command for built-in data provenance HTML report
- `data-priority` command listing low-confidence cases first
- `data-sources` command summarizing built-in data by source
- `data-score` command showing overall data-quality score
- `data-progress` command tracking official-trial/model-test vs public-reference counts and traceability
- `data-target` command checking a composite gate (official/model-test + traceability + LOOCV accuracy) with per-ship-type breakdown
- `data-tiers` command summarizing high/medium/low confidence tiers
- `data-replace-log` command listing per-case replacement actions
- `data-version` command showing built-in database version tag
- `data-targets` command checking multiple quality targets at once
- `validate-types` command grouping validation error by ship type
- `calibrate-type` command applying per-ship-type power calibration
- `data/README.md` documenting provenance and replacement workflow
- `data/CHANGELOG.md` tracking database versions
- built-in-v1.7: 52 cases, all `SourceType::PublicReference` (0/52 independently
  verified, 0/52 traceable); `validate`, `data-score`, `data-target` report this honestly
- 15 additional carrier cases (literature): carrier nested LOOCV improved 13.13% -> 10.68%
- Fn-segmented per-type factors and per-type Fn slopes tested and rejected (overfit: 9.06% -> 11.7%+); a 15% trimmed-mean factor is used instead
- explicit `Provenance { source_type, displacement_type, confidence, source, source_url, verifier, verification_date }` on every case
- hardcoded `DATA_UPGRADES` / `case_source_confidence` removed; confidence is explicit provenance data
- composite quality gate: `0.4*official + 0.2*traceable + 0.4*nested-regime-LOOCV accuracy`, with per-ship-type output
- deterministic holdout split (`validate-holdout`): calibration fit on 28 train, scored on 9 held-out cases
- `tests/accuracy_regression.rs` freezes overall/per-type LOOCV and holdout baselines with regression thresholds
- Ridge/Lasso structural fits standardize features and target, with a scale-aware lambda grid (`1e-4` .. `1e4`)
- QPC shown only for the Holtrop model; original-model output states that no QPC applies
- Savitsky (1964) planing solver and `planing-validate`: on 8 public planing cases the
  calibrated Savitsky and empirical models tie at 23.3% MAPE, so empirical remains the default
- provenance-aware CFD ingestion (`measured_cfd` / `literature` / `analytic` / `synthetic`);
  `cfd-apply` requires >= 20 measured CFD samples. `data/examples/cfd_reference.csv` has 30
  analytic rows (Savitsky vs empirical) to exercise the pipeline but is not evidence
- compartment optimizer upgraded to a deterministic **multi-deck** GA: 6 volumes, deck index +
  within-deck offset, and longitudinal position per compartment, plus a structural access-network
  connectivity constraint; JSON/HTML report decks, access edges and objective
- regime-aware modern model: Holtrop-Mennen + per-ship-type 15% trimmed-mean factor (nested LOOCV 8.92% case-weighted / 8.39% type-averaged over 52 cases); original retained as baseline
- `validate-regime`, `regime-table`, `calibrate-regime` and `accuracy-target` commands
- grouped CLI aliases (`calibrate stats`, `validate holdout`, `data score`, `regime show`, `accuracy target`, `cfd fit`, ...) with flat names preserved
- quality-report HTML test
- `external-report` command for HTML external data reports
- `apply-ensemble-external` command writes external-data ensemble weights into `.ssr`
- sample external datasets in `data/examples/sample_ships.csv`, `more_ships.csv`, `fast_ships.csv`, `escort_ships.csv`, `small_ships.csv`, `planing_ships.csv`, and `more_planing.csv`
- automated test parses all example CSV datasets
- small/fast external validation finite test
- small/fast external merge usable for calibration test
- small/fast external route test
- external-data empty merge test
- small/fast diagnostic helper
- `small-fast` CLI command for small/fast external error diagnostic
- small/fast diagnostic unit test
- planing-regime warning in `small-fast`
- experimental `planing-hp` CLI and module
- `small-fast` suggests `planing-hp` when large error detected
- `small-fast` dynamically calibrates planing-hp from the input dataset
- `compute_planing_calibration` reusable function and unit test
- planing eligibility unit test
- `planing-calibrate` CLI command
- `planing-hp` optional calibration factor

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
- Native GUI uses eframe/egui behind the optional `gui` feature:
  `cargo run --features gui --bin springsharp-gui`. The GUI builds headless but
  needs a display/GL environment at runtime.

## Design notes

- The C# code uses many mutually recursive updates through UI event handlers.
  The Rust model resolves the same dependency graph by running the
  armour/machinery weight cycle to a fixed point.
- Areas where the decompiled C# has obvious typo-like behaviour are replicated
  as faithfully as practical.
- The graphical WinForms code is left out; the terminal CLI and web console are the
  current front-ends. The web console now uses structured form inputs instead of
  a raw `.ssr` textarea, while an optional raw editor could be added later.
