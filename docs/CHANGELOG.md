# Project change log

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
