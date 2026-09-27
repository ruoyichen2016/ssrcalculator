# SpringSharp Rust Roadmap

## P0 hardening (completed 2026-09-27)

- Version-control baseline (git) and reproducible build (`default-run = "springsharp"`)
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all clean
- Honest provenance: `SourceType`/`Provenance` on every case; hardcoded `DATA_UPGRADES`
  and `case_source_confidence` removed
- `data-targets` now measures official-trial/model-test coverage (currently 0/37, fails honestly)
- LOOCV promoted to the primary metric in `calibrate-stats`; in-sample fits labelled diagnostic
- Deterministic holdout split + `validate-holdout` (calibration fitted on 28 train, scored on 9 held-out)
- Docs synced with actual counts (37 cases, 77 tests)

## Completed

- Holtrop-Mennen style resistance/power model
- Propulsion efficiency model
- Historical + external data calibration
- Ridge/Lasso lambda selection
- Internal space volume model with overrides
- Parallel Bootstrap uncertainty
- LOOCV with per-sample residuals
- Ensemble weights optimized by LOOCV
- Web console with design/external/planing endpoints
- Optional native egui/eframe GUI (`--features gui`)
- CFD-surrogate correction framework + CSV fitting + R²/RMSE + LOOCV/holdout + Ridge/λ selection + `cfd-apply` + HTML report + propulsion breakdown in reports
- Prototype compartment allocation + volume/vertical-center optimizer with placement bounds and magazine/armament coupling + calibre/rounds/mounts-scaled magazine volume + structural continuity + maintenance void constraints
- 37 built-in historical cases + multiple example CSV datasets + replacement-priority + source-summary + quality-score + progress/target/tier + replacement-log + version + multi-target reports
- 77 unit tests

## Remaining

### P1

- Replace public-reference cases with official trial / model-basin data (needed to pass `data-targets`)
- Fix Ridge/Lasso regularisation to be scale-aware (current lambda grid is a no-op)

### P2/P3

- Full Savitsky planing model validation
- Train/validate a real CFD surrogate on larger CFD datasets
- Replace prototype compartment optimizer with a full multi-constraint general-arrangement optimizer
