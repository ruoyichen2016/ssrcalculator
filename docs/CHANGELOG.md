# Project change log

## Unreleased (Phases 29–38) — 3D Factored Fleet Tactical Roles, Pure Style Ideal Types & Combat Systems

- **3D Factored fleet tactical roles & ship-type string decoupling** (Phase 38 Track A & B):
  - Removed administrative `ship.ship_type` string dependencies from physics estimation, subsystem resolution, internal crew spaces, and blueprint rendering.
  - Implemented 3-dimensional factored taxonomy of **Fleet Tactical-Functional Ideal Types** without dimensional explosion ($5 + 5 + 4 = 14$ marginal centroids instead of 100 cells):
    - `TacticalEchelon` (5 + 1 Auto): *Battle Line Main Body*, *Fast Wing Vanguard*, *Outer Scouting Screen*, *Inner Fleet Screen*, *Flotilla Assault Line*.
    - `CombatModality` (5 + 1 Auto): *Heavy Artillery Citadel*, *Rapid Fire Surface Gunnery*, *Aviation Deck Projection*, *Massed Torpedo Strike*, *Screen Escort Defense*.
    - `MobilityRegime` (4 + 1 Auto): *Heavy Displacement Block*, *Trans-Froude Fast Capital*, *Slender Wave-Cutting*, *Supercritical High-Fn*.
  - Solved **Aircraft Carrier Differentiation (CV vs CVL vs CVE)** on pure physical invariants ($\Pi_{\text{air}}$, $Fn_{\nabla}$, $L/\nabla^{1/3}$, $C_b$, $\Delta$):
    - **Fleet Carrier (CV, e.g. Essex)**: `BattleLineMainBody` × `AviationDeckProjection` × `TransFroudeFastCapital`.
    - **Light Fleet Carrier (CVL, e.g. Independence)**: `FastWingVanguard` × `AviationDeckProjection` × `SlenderWaveCutting`.
    - **Escort Carrier (CVE, e.g. Casablanca)**: `InnerFleetScreen` × `AviationDeckProjection` × `HeavyDisplacementBlock`.
  - Added hydrodynamic calibration fallback to `HydroDimensionlessFeatures::dominant_ideal_type()` when `ship.ship_type` is empty or arbitrary.
  - Surfaced `Fleet Tactical Role` across text reports, Web Workbench evaluation JSON, and GUI summary.
  - Added round-trip `.ssr` serialization for `systems.tactical_echelon`, `systems.combat_modality`, and `systems.mobility_regime`.
- **Web Workbench & Native Desktop GUI tactical controls & blueprint decoupling** (Phase 38 Track C):
  - Added interactive 3D tactical archetype dropdowns in Web Workbench and combo boxes in Native Desktop GUI (`Page::Weapons`) supporting full manual mix-and-match.
  - Decoupled carrier flight deck and island blueprint rendering in both Web SVG and egui vector painter to trigger from `resolved_modality == CombatModality::AviationDeckProjection`, enabling flight decks on custom and blank hulls.
- **Pure extracted style ideal types & zero flag-state leakage** (Phase 37 Track A):
  - Removed all country/flag-state (`ship.country`) and ship-name (`ship.name`) identifiers from `src/systems.rs`, Web Workbench, and Native Desktop GUI to guarantee zero national label leakage on held-out or custom designs.
  - Added `DimensionlessFeatures` extracting 14 scale-free physical invariants ($\Pi$-groups: $Fn$, $L/\nabla^{1/3}$, $L/B$, $B/T$, $C_b$, $(C_{wp}^2/C_b)(B/T)$, $F/T$, $W_{\text{arm}}/\Delta$, $W_{\text{ord}}/\Delta$, $N_{\text{dp}}/\Delta^{1/3}$, torpedo strike density, $d_{\text{main}}/\sqrt{B}$, $L_{\text{cal}}/45$, endurance ratio, and $\tau$).
  - Replaced national `StyleDoctrine` variants with 4 **Pure Extracted Weberian Engineering Ideal Types**: *Type I · Automated Radar & High-Margin All-Electric*, *Type II · All-Weather Seakeeping & Balanced ASW*, *Type III · Long-Range Optical & Heavy-Ordnance Strike*, and *Type IV · Stereoscopic Precision & Subdivided Survivability*, while preserving full manual mix-and-match across all 6 subsystem slots.
- **Dimensionless hydrodynamic ideal-type speed–power estimation** (Phase 37 Track B):
  - Added `HydroDimensionlessFeatures` and `HydroIdealType` in `src/regime.rs`, reworking `run_loocv_regime()`, `apply_regime_calibration()`, and `apply_blend_calibration()` to weight training cases by dimensionless hydrodynamic similarity and soft ideal-type affinity.
  - Improved 146-case out-of-sample **Nested LOOCV** from **10.36% $\to$ 10.19%** (`validate-regime`, curated core **7.97% $\to$ 7.87%**) and **9.81% $\to$ 9.70%** (`validate-blend`, curated core **7.47% $\to$ 7.45%**).
- **Dimensionless Bouguer stability ($GM$) & subsystem weight rework** (Phase 37 Track C):
  - Reworked transverse metacentric height ($GM$) in `src/model.rs` using dimensionless waterplane flare ($\Pi_{\text{flare}} = C_{wp}/C_b$), wide-beam form factor ($\Pi_{B/T} = B/T$), and high-mounted armoured deck CG leverage ($\Pi_{\text{deck}} = (W_{\text{deck}}/\Delta)(F/T)$), slashing 12-preset **$GM$ MAPE from 13.44% down to 6.60%**.
  - Added dimensionless mount mechanization scaling in `GunBattery::calculate()` and allocated combat-system/auxiliary-generator outfit mass in `ShipModel::calculate()`, reducing 12-preset **Armament+Ammo MAPE from 24.96% down to 15.27%** (`20.82%` $\to$ `14.08%` armoured) and **Hull weight MAPE from 16.84% down to 13.57%** (`11.44%` $\to$ `10.13%` armoured), with tightened regression bounds in `tests/accuracy_regression.rs`.
- **Electrical auxiliaries, adaptive doctrine matcher & mix-and-match combat systems** (Phase 36 Track A):
  - Created `src/systems.rs` implementing naval doctrine styles (`StyleDoctrine`), continuous multi-dimensional **Adaptive Style Matcher** scoring doctrinal fit across ship era, displacement scale, armament, torpedo/ASW density, and stability margin.
  - Full per-slot **manual mix-and-match** for Main Battery Directors (with optical baseline), Secondary/AA Directors, Ballistic Computers, Surface/Air Radar Suites, Sonar Suites, and Electrical Generation Plants.
  - Dynamic **electrical load balance engine** modeling turret drives, hull/hotel/pumps/steering, fire-control systems, and radar/sensors/CIC, auto-sizing turbo and diesel generator suites with doctrine-specific reserve margins and deficit detection.
  - 4-tier structural weight breakdown (`vital_weight_tons`, `hull_weight_tons`, `deck_weight_tons`, `topweight_tons`) and stability-aware metacentric height leverage ($\Delta GM$).
  - Range-dependent empirical-analytical **Salvo Hit Probability Model** integrating optical baseline, centimetric radar precision, computer solution order, and ship steadiness from 5,000 to 35,000 yd.
  - 100% legacy benchmark fidelity: default `couple_weights: false` ensures all 12 historical presets retain identical convergence and physical feasibility.
- **Web Workbench combat systems & hit probability visualization** (Phase 36 Track B):
  - Added form controls for doctrine style and 6 mix-and-match subsystem dropdowns.
  - Added `#systemsWrap` card in Analysis tab rendering doctrine affinity scores, electrical load vs installed capacity progress bar, overload warnings, and 4-tier weight & $\Delta GM$ breakdown.
  - Overlaid green Salvo Hit Probability curve on Tactical Immunity Zone SVG chart with right-hand percentage axis, hover inspection tooltips, and updated legend.
  - Added masthead director tower, optical rangefinder cross-arms (scaled to baseline length), and radar antenna array atop the superstructure in both profile and plan SVG views.
- **Native Desktop GUI combat systems parity** (Phase 36 Track C):
  - Added Electrical Plant & Combat Systems section in `Page::Weapons` with doctrine combo, 6 slot combo boxes, dynamic load breakdown, generator capacity margin %, and $\Delta GM$ diagnostic.
  - Overlaid green Salvo Hit Probability curve with right-hand percentage axis (0%–30%) on vector Tactical Immunity Zone chart in `Page::Blueprint`.
  - Added director tower, optical crossbars, and radar antenna array to profile blueprint vector rendering.
- **Fire control exterior ballistics & immunity zone modeling** (Phase 35 Track A):
  - Created `src/ballistics.rs` implementing numerical trajectory integration and De Marre armor penetration equations (calibrated air retardation scale `7500.0 * ballistic_coeff`, De Marre constant `0.390`).
  - Computes striking velocity, fall angle, vertical belt armor penetration (including inclined belt armor obliquity), and plunging deck penetration at 1,000 yd intervals up to 35,000 yd.
  - Automated tactical immunity zone solver (`compute_immunity_zone`, `compute_self_immunity_zone`) finding the bracket `[inner_yd, outer_yd]` where belt and deck armor withstand incoming shells.
  - Exposed via Web Workbench evaluation API (`src/web.rs`).
- **Target displacement scaling solver & 12-ship historical preset library** (Phase 35 Track B):
  - Implemented `solve_displacement_scale(orig, target_disp, max_iter, tol)` in `src/features.rs`, proportionally adjusting waterline length, beam, and draught to converge upon target displacement within 4–6 iterations.
  - Added *HMS King George V* (1940, 38,000 t) and *Scharnhorst* (1939, 34,850 t) in `src/presets.rs`, expanding the preset library and benchmark suite to **12 iconic warships**.
  - All 12 presets achieve verified physical feasibility (`12/12` positive hull weight, `12/12` positive magazine volume, `12/12` convergence) with tightened regression benchmarks: normal displacement MAPE 3.02% <= 4%, machinery MAPE 3.96% <= 5%, armour MAPE 10.71% <= 12%, GM MAPE 13.67% <= 16% in `tests/accuracy_regression.rs`.
- **Dual-end GUI modernization & parity** (Phase 35 Track C):
  - Added Imperial / Metric unit toggle in both Web Workbench toolbar (`#unitBtn`) and Native Desktop GUI top bar with instantaneous conversion across dimensions, displacements, speeds, armor thicknesses, and broadside weights.
  - Added interactive Target Displacement solver modal/card in both Web Workbench Calibration tab (`POST /api/solve/displacement`) and Native Desktop GUI (`Page::Calibration`).
  - Added side-by-side ship comparison mode in Native Desktop GUI (`Page::Calibration`) matching the Web Workbench with a 13-metric delta table.
  - Added carrier flight deck (profile + plan view with island) and deck torpedo tube mounts to blueprints in both Web Workbench and Native Desktop GUI.
  - Added vector Armor Penetration & Immunity Zone chart (`#immunityWrap` in Web Workbench, `draw_immunity_chart` in Native Desktop GUI).
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
