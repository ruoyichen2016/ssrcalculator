# SpringSharp Rust Roadmap

## P0 hardening (completed 2026-09-27)

- Version-control baseline (git) and reproducible build (`default-run = "springsharp"`)
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all clean
- Honest provenance: `SourceType`/`Provenance` on every case; hardcoded `DATA_UPGRADES`
  and `case_source_confidence` removed
- `data-targets` now measures official-trial/model-test coverage (currently 0/52, fails honestly)
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
  the public historical dataset, validated by nested LOOCV (**9.06%** over 52 cases)
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
  8.92% case-weighted / 8.39% type-averaged.  Per-type Fn slopes tested and
  rejected as overfitting.
- New tests: deck/access feasibility; existing determinism and seed-comparison
  tests cover the multi-deck GA.

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
- 94 unit tests + 7 accuracy-regression integration tests

## Remaining

### P1

- Replace public-reference cases with official trial / model-basin data with
  citation + URL + verifier (needed to pass `data-target`); currently unavailable
- Replace the analytic CFD placeholder with real measured CFD (>20 samples)
- Add more cruiser/battleship literature points to reduce the regime weights
- Add a real structural model (scantlings, hatches) and dimensioned access trunks to the GA

### P2/P3

- Full Savitsky planing model validation on published planing-hull data
- Web console: request-size limits and optional concurrency
- More literature cases for carriers and cruisers
