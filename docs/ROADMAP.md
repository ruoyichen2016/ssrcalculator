# SpringSharp Rust Roadmap

## P0 hardening (completed 2026-09-27)

- Version-control baseline (git) and reproducible build (`default-run = "springsharp"`)
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all clean
- Honest provenance: `SourceType`/`Provenance` on every case; hardcoded `DATA_UPGRADES`
  and `case_source_confidence` removed
- `data-targets` now measures official-trial/model-test coverage (currently 0/123, fails honestly)
- LOOCV promoted to the primary metric in `calibrate-stats`; in-sample fits labelled diagnostic
- Deterministic holdout split + `validate-holdout` (calibration fitted on 28 train, scored on 9 held-out)
- Docs synced with actual counts (unit + integration tests green)

## Phase 1 — validation credibility (completed 2026-09-27)

- Extended `Provenance` with `displacement_type`, `source_url`, `verifier`,
  `verification_date`; every case must have all of them to count as *traceable*
- Composite quality gate for `data-target`: `0.4*official + 0.2*traceable +
  0.4*LOOCV-accuracy`, printed with a per-ship-type breakdown
- Deterministic holdout split (28 train / 9 held-out) and `validate-holdout`
- `tests/accuracy_regression.rs` freezes overall/per-type LOOCV and holdout baselines
- Ridge/Lasso structural fits now normalize features *and* target; lambda grid
  `1e-4 .. 1e4`; regression test asserts the grid is not a no-op
- QPC is marked applicable only to Holtrop; original-model output states that no
  QPC applies (removes the misleading Holtrop-style QPC on original results)

## Phase 2 — modern regime model + CLI groups (completed 2026-09-27)

- Modern default: Holtrop-Mennen + per-ship-type calibration factor fitted to
  the public historical dataset, validated by nested LOOCV (**10.40%** over 123 cases)
- `validate-regime` compares regime vs original+factor (10.57%) and uncalibrated
  Holtrop (19.42%); original kept as the comparison baseline
- `calibrate-regime`, `regime-table`, `accuracy-target` commands
- Accuracy regression test extended with regime baselines and per-type thresholds
- Composite quality gate now uses the nested regime LOOCV accuracy component
- Grouped CLI aliases (`calibrate stats`, `validate holdout`, `data score`,
  `regime show`, `accuracy target`, `cfd fit`, ...) with flat names preserved
- Docs: `docs/VALIDATION.md` regime baseline; official/model-test data recorded
  as unavailable, so the database is explicitly approximate

## Phase 3 — compartment / CFD / GUI enhancements (completed 2026-09-27)

- `compartments --json` machine-readable general-arrangement output
- `compartments-report` HTML report (volumes, CG, longitudinal positions, constraint checks)
- `cfd-status` command plus a 20-sample minimum guard; `cfd-apply` refuses the
  synthetic 9-row example unless `force` is passed
- GUI Calibration page: ship-type regime lookup, "Apply regime model", nested
  LOOCV comparison, and a public-reference provenance warning
- Example-dataset README documents the synthetic CFD placeholder

## Phase 4 — carrier data, Savitsky, CFD provenance, longitudinal placement (completed 2026-09-27)

- Added 15 literature carrier cases; carrier nested LOOCV 13.13% -> 10.94%
- Fn-segmented per-type factors tested and rejected (worse than single factor)
- Savitsky (1964) planing solver + `savitsky-hp` / `planing-validate`; on 8 public
  planing cases calibrated Savitsky and empirical tie at 23.3% MAPE
- CFD ingestion is provenance-aware (`measured_cfd`/`literature`/`analytic`/
  `synthetic`); `cfd-apply` requires >= 20 measured CFD samples; added
  `data/examples/cfd_reference.csv` (30 analytic rows) to exercise the pipeline
- Compartment optimizer extended to longitudinal positions with multi-constraint
  checks and an objective in JSON/HTML

## Phase 5 — global GA general-arrangement optimizer (completed 2026-09-27)

- Deterministic genetic algorithm over 18 variables (6 volumes, 6 vertical
  centres, 6 longitudinal positions); fixed seed for reproducible results
- Feasibility repair keeps every candidate within volume, void, vertical-ordering,
  armament-position and continuity constraints; safety fallback to the
  known-feasible seed
- Objective mixes vertical CG error, longitudinal CG and unused volume, with
  constraint penalties; JSON/HTML/CLI report objective, generations, evaluations
- New unit tests: determinism/feasibility and armament-position adherence

## Phase 6 — multi-deck access-network GA + robust regime factor (completed 2026-09-27)

- Compartment GA extended to a multi-deck model: 6 volumes, per-compartment deck
  index + within-deck offset, and longitudinal position (24 genes); deck ordering
  and adjacency constraints replace raw vertical-centre search.
- Structural access network: compartments are linked when within 0.24 L and on
  the same/adjacent deck; the GA must produce a **connected** network (>= 5
  edges), enforced by repair and checked in the report.
- JSON/HTML/CLI report decks, access edges and connectivity; 10 constraint checks.
- Regime factor switched to a 15% trimmed mean (outlier-robust); nested LOOCV
  8.92% case-weighted / 8.39% type-averaged (52 cases).  Per-type Fn slopes tested and
  rejected as overfitting.
- New tests: deck/access feasibility; existing determinism and seed-comparison
  tests cover the multi-deck GA.

## Phase 7 — passage capacity, escape distance, structural-weight linkage (completed 2026-09-27)

- Access capacity is now **person-based**: compartment occupancy derived from
  crew (machinery watch, magazine handling, accommodation), sized at 250 persons
  per edge-disjoint route via Edmonds-Karp max flow; accommodation always needs
  at least two independent routes
- Escape distance is measured on the **real deck plan**
  (`sqrt((dx*L)^2 + (ddeck*H)^2)`) with limits of 3 hops / 45 m
- Structural weight uses a **plating + bulkhead + framing formula** (steel
  density, 6 mm decks, 8 mm bulkheads, 35% framing) optimised in the GA objective
  and reported in JSON/HTML/CLI
- Three new constraint checks (13 total); deterministic fallback arrangement is
  guaranteed feasible for all of them

## Phase 9 — IMO evacuation, corridor polylines, rule-based scantlings (completed 2026-09-27)

- Evacuation capacity now follows a simplified IMO MSC.1/Circ.1533 model:
  boundary layer 0.15 m per side, maximum specific flow 1.316 (corridor) /
  1.222 (stair) persons/(m*s), per-compartment evacuation times from crew
  occupancy, and a network max-flow check that the crew reaches the trunk
  within 1800 s
- Escape distance uses real corridor **polylines**: cross-deck routes walk to
  the vertical trunk, change deck and walk back; limit 3 hops / 45 m
- Structural weight replaced by a **classification-style** estimate: actual
  deck/section areas, plate thickness `t = s*sqrt(p/(3.5*sigma))`,
  sigma = 0.6*235 MPa, stiffener area from `Z = p*s*l^2/(8*sigma)`, bulkheads
  under the flooding head, plus main girders
- 15 constraint checks total; tests cover IMO times, escape metres, rule weights

## Phase 10 — full IMO evacuation, deck-plan polylines, rule scantlings (completed 2026-09-27)

- IMO density-flow curve `Fs(D) = (1 - aD) k D` (a = 0.266, k = 1.4/1.3) with
  capacity cap `k/(4a)`; congestion-aware network time from the max-flow rate
  (min cut); edge-density check via curve inversion (<= 4 p/m2)
- Assembly-station gene on the weather deck; mustering polyline/time per
  compartment (IMO light-density walking speed + exit queue) checked at 1800 s
- Deck-plan polylines: two longitudinal side corridors at +/- B/4, lateral
  access legs, vertical trunk; limit 3 hops / 45 m
- New `scantling.rs`: rule-style plate bending + buckling, minimum thickness +
  corrosion addition, standard thickness series, stiffener section modulus,
  bulkhead flooding head, deflection l/200; per-element utilisation reported
- 23 constraint checks total (was 15); 96 unit + 7 integration tests

## Phase 12 — expanded historical sample (completed 2026-09-27)

- Added 25 more public-reference ships: 10 battleships (Nelson, Rodney, Queen
  Elizabeth, Revenge, Colorado, Nagato, Richelieu, Littorio, Tirpitz, Dunkerque),
  10 cruisers (Myoko, Takao, Tone, Aoba, Leander, Southampton, Belfast, Wichita,
  Admiral Graf Spee, Kirov) and 5 destroyers (Shimakaze, Akizuki, Yugumo,
  Le Fantasque, Gleaves); dataset 52 -> 77
- The broader sample exposes genuine extreme-Froude weaknesses (Shimakaze 40.9 kt
  over-predicted, Queen Elizabeth/Revenge low-Fn under-predicted): regime nested
  LOOCV 8.92% -> 10.21% case-weighted / 8.39% -> 10.35% type-averaged
- Still far ahead of the original SpringSharp model (21.58% uncalibrated);
  per type: cruiser 8.5%, carrier 10.7%, battleship 10.7%, destroyer 11.5%
- Holdout re-frozen at 58 train / 19 held-out (original 13.9%, Holtrop 12.5%)

## Phase 13 — third sample expansion (completed 2026-09-27)

- Added 24 more public-reference ships: 6 battleships (Fuso, New Mexico,
  Tennessee, Nevada, Iron Duke, Conte di Cavour), 6 carriers (Zuikaku, Junyo,
  Ryujo, Furious, Eagle, Hermes), 6 cruisers (Pensacola, Portland, San
  Francisco, Furutaka, Suzuya, Algerie) and 6 destroyers (Gridley, Sumner,
  Gearing, Fubuki, Asashio, Z1 Leberecht Maass); dataset 77 -> 101
- Overall regime nested LOOCV held at 10.21% (macro 10.35% -> 10.21%); per type
  moved to battleship 9.31%, carrier 11.60%, cruiser 9.79%, destroyer 10.16%
- Original SpringSharp baseline 21.70%; original+factor 11.51%; Holtrop
  uncalibrated 19.66%
- Holdout re-frozen at 76 train / 25 held-out (original 15.1%, Holtrop 13.1%)

## Phase 14 — fourth sample expansion + repo cleanup (completed 2026-09-27)

- Removed the scratch `examples/segment_experiment.rs`; only the documented
  `examples/generate_cfd_reference.rs` remains.
- Added 22 more public-reference ships: 5 battleship/battlecruiser (Bretagne,
  Courbet, Andrea Doria, Ise, Renown), 5 carriers (Hiyo, Ryuho, Implacable,
  Sangamon, Long Island), 6 cruisers (Nachi, Chikuma, Trento, Duquesne, Emden,
  Fiji) and 6 destroyers (Jervis, Shiratsuyu, Matsu, Z23, Soldati, Mogador);
  dataset 101 -> 123
- Regime nested LOOCV 10.21% -> 10.71% case-weighted (macro 10.21% -> 10.75%);
  per type: battleship 9.84%, carrier 12.23%, cruiser 9.71%, destroyer 11.23%
- Original SpringSharp baseline 21.28%; original+factor 11.34%; Holtrop
  uncalibrated 19.56%
- Holdout re-frozen at 93 train / 30 held-out (original 13.9%, Holtrop 12.6%)

## Phase 15 — confidence tiers, curated reporting, model v2 test (completed 2026-09-27)

- Split the 123 cases into a **curated core** (37 cases, declared confidence 0.7)
  and an **approximate bulk** (86 cases, 0.5); provenance constructors and
  `curated_reference()` added
- Regime reports now include `curated_cases` / `curated_mape`; `validate-regime`
  and `regime-residuals` show the curated subset (8.18% nested LOOCV)
- Per-type factor switched to a **confidence-weighted** 15% trimmed mean
  (curated 2x): full 10.71% -> 10.60%, curated 8.38% -> 8.18%, macro 10.75% -> 10.64%
- **Fn-aware model v2 tested and rejected**: per-type linear (with 0/4/8
  shrinkage), shared-slope, and quadratic corrections gave no robust gain and
  worsened the curated core, so the constant per-type factor is retained
- Regression tests extended with curated-subset assertions

## Phase 16 — data verification tools + fixes (completed 2026-09-27)

- Added `data-consistency`: compares documented displacement with the engine's
  `L*B*T*Cb/35` (full-load) model; flags ratios outside [0.95, 1.60].
- Added `data-outliers`: screens the 86 approximate cases against a power law
  fitted to the 37 curated cases (flags > 2.5 sigma; currently 1, Ranger CV).
- Verification fixes: Wasp and Shokaku used the flight-deck width as beam; both
  corrected to the hull waterline beam (93 -> 80 ft, 95 -> 85 ft).  Regime nested
  LOOCV improved 10.60% -> 10.40% (macro 10.64% -> 10.45%, curated 8.18% -> 8.09%,
  carrier 12.01% -> 11.24%).
- A blanket `Cb = documented*35/(L*B*T)` re-derivation was tested and rejected:
  it improved the curated core (7.65%) but hurt the full set (10.91%, cruisers
  12.08%), so only the unambiguous beam fixes were kept.
- Remaining consistency flags are standard-vs-full-load convention differences,
  not errors; the outlier screen found no further clear data errors.

## Phase 17 — engineering wrap-up: CI + docs (completed 2026-09-27)

- Added `.github/workflows/ci.yml`: a core job (fmt, clippy `-D warnings`,
  `cargo test --all-targets`) and a GUI job (installs the X11/xkbcommon/Wayland
  dev packages and builds `--features gui --bin springsharp-gui`).
- Moved the ~130-bullet implementation log out of `README.md` into
  `docs/CHANGELOG.md`; README is now ~400 lines and points to the log.
- Documented the CI in the README design notes and the documentation index.

## Phase 18 — isolate the GUI build (completed 2026-09-27)

- `scripts/build-gui.sh` builds the optional GUI with `CARGO_TARGET_DIR=target-gui`,
  so the eframe/egui dependency tree (~130 crates) never shares the core target.
- `[profile.dev.package."*"] debug = false` drops debug symbols from dependency
  artifacts (the GUI tree is the only dependency tree), shrinking the cache.
- `target-gui/` ignored by git and removable independently (`rm -rf target-gui`).
- Core builds/tests remain dependency-free and fast.

## Completed

- Holtrop-Mennen style resistance/power model
- Propulsion efficiency model
- Historical + external data calibration
- Ridge/Lasso lambda selection (scale-aware, LOOCV-selected)
- Internal space volume model with overrides
- Parallel Bootstrap uncertainty
- LOOCV with per-sample residuals
- Ensemble weights optimized by LOOCV
- Web console with design/external/planing endpoints
- Optional native egui/eframe GUI (`--features gui`)
- CFD-surrogate correction framework + CSV fitting + R²/RMSE + LOOCV/holdout + Ridge/λ selection + `cfd-apply` + HTML report + propulsion breakdown in reports
- Global GA general-arrangement optimizer + calibre/rounds/mounts-scaled magazine volume + structural continuity + maintenance void constraints
- 52 built-in historical cases + multiple example CSV datasets + replacement-priority + source-summary + quality-score + progress/target/tier + replacement-log + version + multi-target reports
- 94 unit tests + 7 accuracy-regression integration tests (GA suite exercises the capacity/escape checks)

## Remaining

### P1

- Replace public-reference cases with official trial / model-basin data with
  citation + URL + verifier (needed to pass `data-target`); currently unavailable
- Replace the analytic CFD placeholder with real measured CFD (>20 samples)
- Add more cruiser/battleship literature points to reduce the regime weights
- Replace the planning rule checks with a licensed class-rule scantling package and a real deck plan

### P2/P3

- Full Savitsky planing model validation on published planing-hull data
- Web console: request-size limits and optional concurrency
- More literature cases for carriers and cruisers
