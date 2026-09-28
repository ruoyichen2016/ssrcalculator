# SpringSharp Rust Roadmap

## P0 hardening (completed 2026-09-27)

- Version-control baseline (git) and reproducible build (`default-run = "springsharp"`)
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all clean
- Honest provenance: `SourceType`/`Provenance` on every case; hardcoded `DATA_UPGRADES`
  and `case_source_confidence` removed
- `data-targets` now measures official-trial/model-test coverage (currently 0/37, fails honestly)
- LOOCV promoted to the primary metric in `calibrate-stats`; in-sample fits labelled diagnostic
- Deterministic holdout split + `validate-holdout` (calibration fitted on 28 train, scored on 9 held-out)
- Docs synced with actual counts (37 cases, 80 unit tests)

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
- 80 unit tests + 5 accuracy-regression integration tests

## Remaining

### P1

- Replace public-reference cases with official trial / model-basin data with citation + URL + verifier (needed to pass `data-target`)
- Regime-aware model selection (Fn / ship type) to cut overall LOOCV MAPE from ~10.5%

### P2/P3

- Full Savitsky planing model validation
- Train/validate a real CFD surrogate on larger CFD datasets
- Replace prototype compartment optimizer with a full multi-constraint general-arrangement optimizer
