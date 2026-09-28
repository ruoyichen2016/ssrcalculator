# Validation and data-quality methodology

## Definitions

Every built-in case carries an explicit provenance record:

```text
Provenance {
    source_type,        # OfficialTrial | ModelTest | PublicReference | Synthetic
    displacement_type,  # Normal | Standard | FullLoad | Trial | Deep | Unspecified
    confidence,         # declared 0..1, coarse; NOT verification
    source,             # citation / description
    source_url,         # URL for the specific source
    verifier,           # person/team who checked the case
    verification_date,
}
```

- **Verified** means `source_type` is `OfficialTrial` or `ModelTest` only.
  Self-declared `confidence` can never make a case count as verified.
- **Traceable** means citation + URL + named verifier are all present.
- Current built-in database: **0/52 verified, 0/52 traceable**, all
  `PublicReference` (Conway's/Jane's/class-trial summaries; approximate).

## Accuracy evidence

In-sample fits (all 52 cases used for fitting and scoring) are optimistic and
must not be used as accuracy claims.

**Primary metric: nested LOOCV of the regime model** (`run_loocv_regime`,
`validate-regime`).  This is an out-of-sample estimate: the per-ship-type
calibration factor is re-fitted on each training split.

Secondary metrics:

- **Holdout** (`validate-holdout`) fits calibration on 39 training cases and
  scores 13 held-out cases.
- The reduced/structural ensemble (`run_loocv`, `calibrate-stats`) is retained
  as a diagnostic; its `ensemble_mape` fits the blend weight on all folds, so it
  is optimistic.  The honest nested 2-way number is ~10.8%.

### Modern regime model

`Holtrop-Mennen + per-ship-type calibration factor`, fitted on the public
historical dataset (the default when a design is run through
`calibrate-regime`).  Factors (ref/sim, 15% trimmed mean): battleship 0.760,
carrier 0.778, cruiser 0.922, destroyer 1.053.  The carrier factor was refitted
after adding 15 literature carrier cases (carrier nested LOOCV 13.13% -> 10.68%).

### Frozen baseline (2026-09-27)

| Metric | Baseline (52 cases) | Regression threshold |
|---|---|---|
| **Regime nested LOOCV, case-weighted** | **~8.92%** | **<= 10.0%** |
| Regime nested, type-averaged (macro) | ~8.39% | <= 9.5% |
| Regime nested, battleship | ~8.50% | <= 9.6% |
| Regime nested, carrier | ~10.68% | <= 12.5% |
| Regime nested, cruiser | ~7.84% | <= 8.8% |
| Regime nested, destroyer | ~6.53% | <= 7.3% |
| Original + per-type factor (baseline) | ~11.67% | (must be beaten) |
| Holtrop uncalibrated | ~22.24% | (must be beaten) |
| 2-way ensemble, reported (optimistic) | ~10.0% | <= 12.0% |
| Holdout MAPE, original | ~16.1% | <= 18.5% |
| Holdout MAPE, Holtrop | ~12.9% | <= 14.5% |

These are enforced by `tests/accuracy_regression.rs`.  If a change is a
deliberate improvement, update the baseline here and the constants in that file
together.

**Factor estimator.**  The per-ship-type factor is a 15% trimmed mean of
`reference / simulated`, which rejects outlier cases.  A nested-LOOCV sweep over
5-20% trims and the geometric mean showed a stable ~0.1pp improvement over the
plain mean; a per-type *linear-in-Fn* factor was tested and rejected (it
overfits: overall nested error rises from ~8.9% to 11.7%+).

> Official trial / model-basin data is **not available** to this project.  The
> database is public historical reference data and is explicitly approximate;
> the regime factors are fitted to it and should be treated as empirical
> corrections, not as validation against measured trial data.

## Planing (Savitsky) validation

`savitsky.rs` implements the Savitsky (1964) planing equations (lift balance,
centre of pressure, equilibrium trim, ITTC-1957 friction).  `planing-validate`
compares it with the empirical planing model on the 8 public planing cases in
`planing_ships.csv` + `more_planing.csv`, assuming deadrise 12-14 deg,
LCG = 0.42 L and propulsive coefficient 0.55:

| Model | Raw MAPE | Calibrated MAPE |
|---|---|---|
| Savitsky | 73.3% | 23.3% |
| Empirical | 56.6% | 23.3% |

The two models tie once calibrated, so the empirical model remains the default
planing path and Savitsky is available as an alternative (`savitsky-hp`).  The
sample is small and the deadrise/LCG assumptions are not measured; this is a
sanity/consistency check, not a formal Savitsky validation.

## CFD surrogate provenance

CFD samples carry a provenance tier: `measured_cfd`, `literature`, `analytic`
or `synthetic` (a 3-column row defaults to `synthetic`).  Only
**>= 20 `measured_cfd`** samples count as evidence:

- `cfd-status` reports the mix and `usable_for_evidence`.
- `cfd-apply` refuses to write a proxy unless there are enough measured CFD
  samples (or `force` is passed).
- `data/examples/cfd_reference.csv` contains 30 **analytic** rows (Savitsky vs
  the empirical planing model) to exercise the pipeline.  It is **not** evidence,
  and the project currently has **0 measured CFD samples**.

## Composite quality gate (`data-target`)

```text
composite = 0.4 * official_model_fraction
          + 0.2 * traceability_fraction
          + 0.4 * loocv_accuracy_score

loocv_accuracy_score = 1.0                       if MAPE <= 10%
                     = 0.0                       if MAPE >= 30%
                     = linear interpolation      in between
```

Baseline composite (52 public-reference cases, nested regime LOOCV 9.06%):
**0.400**.  `data-target 0.8` (and all `data-targets` thresholds) therefore fail
honestly until official trial / model-basin data with citations, URLs and named
verifiers is added.

## Regularised structural fits

The Ridge/Lasso structural-form multipliers are fitted on features **and**
target normalized to unit RMS, so the lambda grid `1e-4 .. 1e4` is meaningful.
`select_ridge_lambda` / `select_lasso_lambda` choose lambda by LOOCV, and
`tests/accuracy_regression.rs` asserts the grid is not a no-op.

## Known limitations

- All displacement types are currently declared `Normal` and unverified.
- The holdout split is deterministic by index (stable, but not stratified by
  ship type).
- `calibrate-*` commands still fit on all 52 cases by design; use LOOCV and
  `validate-holdout` for accuracy claims.
- `data-target` also includes an accuracy component, so it can move when the
  model changes even if the data does not.
- The planing validation rests on 8 public cases with assumed deadrise/LCG.
- CFD evidence is still absent: 0 measured CFD samples; only analytic reference
  rows are bundled.
- Many of the 15 added carrier cases are approximate (design/trial figures,
  normal vs standard displacement mixed); treat the carrier factor as empirical.
- The compartment optimizer now has decks, an IMO MSC.1/Circ.1533-style
  evacuation model (density curve, congestion via max flow, assembly-station
  mustering, edge density), corridor-polylines on a schematic deck plan and a
  rule-style scantling module.  It is still a *schematic* plan: one nominal
  corridor/stair width per edge, no counterflow or assembly-area sizing, and the
  scantlings use planning pressures/corrosion margins rather than a licensed
  class-rule scantling package.
