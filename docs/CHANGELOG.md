# Project change log

## Unreleased (Phases 29–34) — 10-Preset Library, Live Blueprints, Subsystem Weight Benchmarks, Machinery Calibration, Side-by-Side Comparison & Native GUI Parity

- **10-preset expansion & machinery weight calibration** (Phase 34 Track A):
  - Added *HMS Hood* (1920 Battlecruiser, 41,785 t) and *USS Essex* (1943 Aircraft carrier, 31,500 t) to `src/presets.rs`, expanding the built-in preset library and subsystem weight/stability benchmark suite to **10 iconic warships**.
  - Calibrated `machinery_weight_factor` across all 10 presets, slashing overall machinery weight MAPE from **29.69%** down to **1.43%** (0.84% across 8 armoured warships) and improving normal displacement MAPE to **2.57%**, armour MAPE to **5.84%**, armoured hull MAPE to **11.67%**, and $GM$ MAPE to **12.80%**.
  - Exposed `broadside_lbs`, `g1_shell_lbs`, and `g1_broadside_lbs` in `evaluation_json()` and added `GET /api/presets/{id}/eval` in `src/web.rs`.
- **Web Workbench side-by-side ship comparison & firepower display** (Phase 34 Track B):
  - Added Side-by-Side Ship Comparison Mode in the Analysis tab (`pinBaseline()`, `comparePreset(id)`, `clearCompare()`), displaying a 14-row comparison table with signed $\Delta$ and `%` columns and overlaying the compared ship's speed-power curve on the SVG propulsion chart.
  - Added Broadside Throw Weight (`lb`) dashboard card and main battery shell/salvo weight details in blueprint hover tooltips.
- **Native desktop GUI full parity** (Phase 34 Track C):
  - Added `Page::Weapons` (`Weapons & Misc`) for configuring torpedo sets, mines, depth charges, and all 5 miscellaneous weight fields.
  - Added native `egui::Painter` Speed-Power Curve chart in `Page::Blueprint` (`Blueprint & Power`) and broadside throw weight in the dashboard header.
  - Added `Page::Spaces` (`Spaces & Benchmarks`) displaying the historical subsystem weight & stability benchmark table, internal space volume budget (`calculate_internal_spaces`), and multi-deck GA arrangement optimizer (`suggest_compartment_placement`).
- **Speed-power curve & interactive blueprint tooltips** (Phase 33 Track A):
  - Added dynamic speed-power curve calculation (`speed_curve: [[speed, model_hp, orig_hp, fn], ...]`) across 10 kt to max speed + 2 kt via exposed `EngineValues::speed_hp_calc()`.
  - Added SVG speed-power curve chart in Web Workbench Analysis tab (`#speedPowerWrap`) comparing calibrated regime model vs original SpringSharp baseline with cruise/max speed markers.
  - Added interactive inspection tooltips (`#bpTip`) across blueprint SVG elements (hover over belt armor, bridge superstructure, turrets with caliber/mount/armor/ammo details, and secondary mounts).
  - Added longitudinal multi-deck compartment layout SVG schematic in `compartment_placement_html()`.
  - Added color-coded tolerance badges (green $\le 10\%$, yellow $10\%\sim 20\%$, red $> 20\%$) in the preset weight validation table.
- **Native desktop GUI modernization** (Phase 33 Track B):
  - Added 8 historical warship presets dropdown (`PRESET_LIST`, `get_preset()`) in the left navigation panel for instant loading.
  - Replaced integer-code inputs with user-friendly semantic combo boxes for Bow forms, Stern types, Gun types, Mount types, and Turret layouts, plus descriptive engine/drive options and `machinery_weight_factor`.
  - Added live dashboard header with weight convergence badge, primary dimensions, speed, power, GM, and a painted segmented weight distribution bar (Hull, Armour, Machinery, Armament, Magazine, Bunker).
  - Added dedicated `Page::Blueprint` with dual-view vector graphics via `egui::Painter`: Profile View (deckline, belt, keel, bridge, turrets) and Plan View (beam curvature, centerline, turrets, and toggleable 290° firing arc projection sectors).

- **Machinery weight scaling & destroyer calibration** (Phase 32 Track B):
  - Added `engine.machinery_weight_factor` to `EngineValues` (serialized in `.ssr`), scaling machinery weight for advanced or lightweight power plants.
  - Tuned historical destroyer presets: *USS Fletcher* (`0.50`) and *IJN Shimakaze* (`0.42`), accurately modelling 80-100 shp/ton high-pressure compact steam machinery.
  - Slashed destroyer machinery weight error from ~+106%/+150% down to +3% (*Fletcher* 709t vs 690t ref) and +5% (*Shimakaze* 859t vs 820t ref).
  - Reduced overall 8-preset machinery MAPE from **60.63%** down to **29.69%**, relieving residual hull weight squeeze while maintaining all physical feasibility and GM stability gates.
- **Web Workbench blueprint modal & firing arcs** (Phase 32 Track A):
  - Added full-screen / expanded blueprint modal dialog (`#bpModal`) with responsive SVG scaling.
  - Added interactive firing arc projection toggle (`#btnArcs` / `toggleArcLayers()`) overlaying dashed 290° training arc sectors on all main battery turrets.
  - Added standalone SVG blueprint download (`exportSvg()`).
  - Added numeric input for `machinery_weight_factor` in the machinery configuration tab.
- **Server concurrency protection**:
  - Guarded HTTP connection listener loop with `MAX_CONCURRENT_CONNECTIONS = 64` and RAII drop guard returning `503 Service Unavailable: connection limit reached` on saturation.
- **Subsystem weight & stability benchmark validation** (`validate-weights` / `validate_preset_weights`):
  - Benchmarks simulated displacement, hull structure, armour + barbettes, machinery,
    armament + ammunition, normal bunker, and transverse metacentric height ($GM$)
    against published naval-architecture references across all 8 full-design presets.
  - Achieves **3.59%** normal displacement MAPE, **7.64%** armour weight MAPE (6 armoured
    warships), **13.91%** $GM$ MAPE, **14.72%** armoured-ship hull weight MAPE, and **29.69%**
    machinery MAPE, enforced by `preset_weights_and_stability_do_not_regress` in `tests/accuracy_regression.rs`.
  - Updated `validation-audit` to report the 8 full-design preset weight & stability benchmarks.
- **Built-in historical ship presets library** (`src/presets.rs`):
  8 fully configured historical models (*HMS Dreadnought*, *Bismarck*, *USS Iowa*,
  *IJN Yamato*, *USS Baltimore*, *HMS Belfast*, *USS Fletcher*, *IJN Shimakaze*)
  with guaranteed weight convergence and positive physical characteristics.
- **REST preset endpoints**: `/api/presets` and `/api/presets/{id}` in `src/web.rs`.
- **Workbench UI features**:
  - Semantic mount descriptions for gun battery 1 & 2 (`Single`, `Twin`, `Triple`, etc.).
  - Instant one-click historical ship preset selector.
  - Live interactive SVG blueprint rendered in the dashboard (profile view with waterline,
    freeboard shear profile, draught keel, armor belt, profile turrets; plan view with beam
    curvature and caliber-scaled turret mounts).
  - Historical subsystem weight & stability comparison table in the Analysis tab whenever
    a preset design is evaluated, plus a fix for double-counted gun armour (`g.arm_weight`)
    in the dashboard weight distribution bar.
  - Standalone HTML export embeds the live SVG ship blueprint.

## 0.3.0 (2026-09-27) — hardening release

No accuracy baseline changed (`validate-regime` 10.36% case-weighted, blended
9.81%); this release is about the code being easier to change and harder to
misuse.  See `docs/ROADMAP.md` Phase 27.

- **CLI restructured**: `src/main.rs` 1878 -> 230 lines; command bodies moved to
  per-theme `src/cli/` submodules behind a shared `CliError` with exit codes
  2 (usage) / 1 (runtime) / 4 (quality gate not met).  A missing
  `load_from_file`/`save_to_file` import that stopped the binary from compiling
  is fixed, and the now-unused `use springsharp::{...}` block is pruned.
- **Command surface is now tested**: `tests/cli_surface.rs` drives the real
  binary and fails if any command advertised in `--help` is not routed —
  `html` was previously advertised and implemented but never dispatched.
- **Input validation**: `NaN`/`inf` are rejected and every dimensional field is
  range-checked when parsing `.ssr` input, so a malformed file errors instead of
  yielding non-physical results.  `percentile` uses `f64::total_cmp` so a
  non-finite bootstrap sample cannot panic the fit.
- **Convergence is reported**: the armour <-> machinery fixed point iterates to a
  1e-9 relative tolerance (cap 50) instead of a fixed 12 passes and records
  `converged` / `iterations` / `iteration_residual`; a non-converged design is
  flagged in `summary`, `generate_report` and the web console.
- **HTML escaping**: new `features::html_escape`, applied to every interpolated
  user-supplied string in `html`, `quality-report` and `external-report`.
- **Web console hardening**: 8 MB body cap (413), request-line/header length
  caps, 10 s read/write timeouts, per-connection threads instead of a blocking
  accept loop, and `catch_unwind` so a handler panic cannot stop the server.
- **Licence scope**: added `NOTICE` and a README section recording that the
  upstream SpringSharp 3.0 source is proprietary, so MIT covers the original
  Rust code but not the ported formula content.
- `[profile.dev] debug = "line-tables-only"` in both workspaces: core `target/`
  217 MB -> 162 MB for the build+clippy+test dev loop, rlib 20.5 MB -> 10.3 MB,
  with no measurable build-time cost and line numbers preserved in backtraces
- Release profile tuned: `opt-level = 3`, thin LTO, single codegen unit, stripped
  (`panic` stays `unwind` because the web console relies on `catch_unwind`)
- `gui/.cargo/config.toml` pins `build.target-dir = "../target-gui"`, and
  `scripts/check.sh` / CI export `CARGO_TARGET_DIR=target-gui`, so GUI builds can
  no longer create a second ~950 MB `gui/target/` copy of the eframe tree
- `scripts/check.sh` warns when a stray `gui/target/` exists (it is gitignored,
  so it was previously invisible in `git status`)
- CI sets `CARGO_INCREMENTAL=0`; incremental artifacts are the largest single
  contributor to a cached target dir and buy nothing in CI
- CI gained an MSRV job that builds against `rust-version` with
  `RUSTUP_TOOLCHAIN` set, because `rustup default` alone loses to the repository
  `rust-toolchain.toml` directory override (the job would have silently tested
  the pinned dev toolchain instead of the declared minimum)
- Declared `rust-version = "1.80"` and pinned `rust-toolchain.toml` so local and
  CI builds share one toolchain

## 0.2.0 (2026-09-27) — release notes

Highlights since 0.1.0:

- **Modern regime model**: Holtrop-Mennen + per-ship-type confidence-weighted
  factor; nested LOOCV 10.40% case-weighted / 10.45% type-averaged over 123
  cases, curated core 8.09%, versus the original SpringSharp model at 21.05%.
- **Validation**: provenance tiers, LOOCV/holdout, composite quality gate,
  per-case residuals and data audits (`data-consistency`, `data-outliers`).
- **123 historical cases** with a curated core (37) and approximate bulk (86).
- **General arrangement**: multi-deck GA with access network, IMO evacuation
  (density/congestion/assembly) and corridor-polyline escape distances.
- **Rule-style scantlings**, Savitsky planing, CFD-surrogate provenance tooling.
- **Engineering**: CI, zero-dependency core, GUI as a separate workspace,
  `scripts/check.sh`.

The cumulative implementation log below predates this summary.  See
`docs/ROADMAP.md` for the phase-by-phase record.

This is the cumulative implementation log for the Rust SpringSharp port.  For
the current status and roadmap see `docs/ROADMAP.md`; for accuracy baselines see
`docs/VALIDATION.md`.  Data changes are tracked separately in
`data/CHANGELOG.md`.

## Features and fixes

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
- built-in-v1.12: 123 cases, split into a **curated core** (37 cases, declared
  confidence 0.7) and an **approximate bulk** (86 cases, 0.5); 0/123 independently
  verified, 0/123 traceable — `validate`, `data-score`, `data-target` report this honestly
- data verification: `data-consistency` (L*B*T*Cb/35 vs documented displacement) and
  `data-outliers` (screen approximate cases vs the curated power law); fixed two
  flight-deck-vs-hull beam errors (Wasp, Shokaku), which cut the regime MAPE
- per-ship-type factor is a 15% trimmed mean with curated cases weighted 2x;
  nested LOOCV 10.71% -> 10.60% (curated core 8.38% -> 8.18%)
- Fn-aware corrections (per-type linear/quadratic, shared slope) tested and
  rejected: no robust gain on either the full set or the curated core
- 86 additional cases added in four batches (carriers, battleship/cruiser/destroyer);
  the wider sample exposes extreme-Froude weaknesses (regime 8.92% -> 10.71%)
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
- compartment optimizer upgraded to a deterministic **multi-deck** GA (volumes, deck, offset,
  longitudinal position, assembly-station position) with **IMO MSC.1/Circ.1533 evacuation**
  (density-flow curve, effective width, congestion via max flow, assembly-station mustering, edge
  density), **deck-plan corridor polylines** with lateral legs (<= 3 hops / <= 45 m), and a
  **rule-style scantling module** (plate bending + buckling + corrosion, stiffener section modulus,
  deflection); JSON/HTML report occupancy, evacuation/mustering times, congestion, density, escape
  metres, scantling utilisation, structure weight and objective (23 checks)
- regime-aware modern model: Holtrop-Mennen + per-ship-type confidence-weighted 15% trimmed-mean factor (nested LOOCV 10.40% case-weighted / 10.45% type-averaged over 123 cases, curated core 8.09%, vs original 21.05%); original retained as baseline
- `validate-regime`, `regime-table`, `calibrate-regime` and `accuracy-target` commands
- grouped CLI aliases (`calibrate stats`, `validate holdout|audit`, `data score`, `regime show|residuals`, `accuracy target`, `cfd fit`, ...) with flat names preserved
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
