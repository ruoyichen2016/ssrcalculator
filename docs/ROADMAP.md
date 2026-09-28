# SpringSharp Rust Roadmap

## P0 hardening (completed 2026-09-27)

- Version-control baseline (git) and reproducible build (`default-run = "springsharp"`)
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all clean
- Honest provenance: `SourceType`/`Provenance` on every case; hardcoded `DATA_UPGRADES`
  and `case_source_confidence` removed
- `data-targets` now measures official-trial/model-test coverage (currently 0/37, fails honestly)
- LOOCV promoted to the primary metric in `calibrate-stats`; in-sample fits labelled diagnostic
- Deterministic holdout split + `validate-holdout` (calibration fitted on 28 train, scored on 9 held-out)
- Docs synced with actual counts (37 cases; unit + integration tests green)

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
  the public historical dataset, validated by nested LOOCV (**8.66%** overall)
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
- Prototype compartment allocation + volume/vertical-center optimizer with placement bounds and magazine/armament coupling + calibre/rounds/mounts-scaled magazine volume + structural continuity + maintenance void constraints
- 37 built-in historical cases + multiple example CSV datasets + replacement-priority + source-summary + quality-score + progress/target/tier + replacement-log + version + multi-target reports
- 84 unit tests + 7 accuracy-regression integration tests

## Remaining

### P1

- Replace public-reference cases with official trial / model-basin data with
  citation + URL + verifier (needed to pass `data-target`); currently unavailable
- Improve the carrier regime (13.1% nested) and add more literature cases
- Replace the synthetic CFD placeholder with a real dataset (>20 samples)
- Replace the prototype compartment optimizer with a full general-arrangement optimizer

### P2/P3

- Full Savitsky planing model validation on published planing-hull data
- Web console: request-size limits and optional concurrency
- More literature cases for carriers and cruisers
