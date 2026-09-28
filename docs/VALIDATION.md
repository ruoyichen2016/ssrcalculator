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
- Current built-in database: **0/37 verified, 0/37 traceable**, all
  `PublicReference` (Conway's/Jane's/class-trial summaries; approximate).

## Accuracy evidence

In-sample fits (all 37 cases used for fitting and scoring) are optimistic and
must not be used as accuracy claims.

- **LOOCV** (`run_loocv`, `calibrate-stats`) is the primary metric.
- **Holdout** (`validate-holdout`) fits calibration on 28 training cases and
  scores 9 held-out cases.
- Per-ship-type LOOCV (`loocv_by_type`) feeds the per-type quality gate.

### Frozen baseline (2026-09-27)

| Metric | Baseline | Regression threshold |
|---|---|---|
| Overall LOOCV ensemble MAPE | ~10.5% | <= 12.0% |
| Overall LOOCV reduced MAPE | ~12.7% | <= 14.2% |
| Overall LOOCV structural MAPE | ~14.9% | <= 16.5% |
| Battleship LOOCV | ~11.1% | <= 12.6% |
| Carrier LOOCV | ~12.6% | <= 14.1% |
| Cruiser LOOCV | ~12.3% | <= 13.8% |
| Destroyer LOOCV | ~5.3% | <= 6.8% |
| Holdout MAPE, original | ~16.5% | <= 18.5% |
| Holdout MAPE, Holtrop | ~10.9% | <= 12.5% |

These are enforced by `tests/accuracy_regression.rs`.  If a change is a
deliberate improvement, update the baseline here and the constants in that file
together.

## Composite quality gate (`data-target`)

```text
composite = 0.4 * official_model_fraction
          + 0.2 * traceability_fraction
          + 0.4 * loocv_accuracy_score

loocv_accuracy_score = 1.0                       if MAPE <= 10%
                     = 0.0                       if MAPE >= 30%
                     = linear interpolation      in between
```

Baseline composite (37 public-reference cases, LOOCV ensemble 10.5%):
**0.390**.  `data-target 0.8` (and all `data-targets` thresholds) therefore fail
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
- `calibrate-*` commands still fit on all 37 cases by design; use LOOCV and
  `validate-holdout` for accuracy claims.
- `data-target` also includes an accuracy component, so it can move when the
  model changes even if the data does not.
