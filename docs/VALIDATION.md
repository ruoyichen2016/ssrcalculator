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
- Current built-in database: **0/146 verified, 0/146 traceable**; 37 curated
  (declared confidence 0.7), 109 approximate (0.5), all
  `PublicReference` (Conway's/Jane's/class-trial summaries; approximate).

## Accuracy evidence

In-sample fits (all 146 cases used for fitting and scoring) are optimistic and
must not be used as accuracy claims.

**Primary metric: nested LOOCV of the regime model** (`run_loocv_regime`,
`validate-regime`).  This is an out-of-sample estimate: the per-ship-type
calibration factor is re-fitted on each training split.

Secondary metrics:

- **Holdout** (`validate-holdout`) fits calibration on 109 training cases and
  scores 37 held-out cases.
- The reduced/structural ensemble (`run_loocv`, `calibrate-stats`) is retained
  as a diagnostic; its `ensemble_mape` fits the blend weight on all folds, so it
  is optimistic.  The honest nested 2-way number is ~10.8%.

### Modern regime model

`Holtrop-Mennen + per-ship-type calibration factor`, fitted on the public
historical dataset (the default when a design is run through
`calibrate-regime`).  Factors (ref/sim, 15% trimmed mean) at 146 cases:
battleship 0.767, carrier 0.803, cruiser 0.911, destroyer 1.017.  The wider
sample (five expansion batches) moved the overall nested LOOCV from 8.92% to
10.36% (curated core 7.97%).  Data verification corrected two carrier beam entries (Wasp,
Shokaku), the Sendai length and nine consistency-flagged approximate cases; see
`docs/DATA_REVIEW.md`.

### Comparison against the original baseline (`regime-compare`)

Nested, paired over all 146 cases:

| Model | MAPE | median abs error |
|---|---|---|
| **regime (Holtrop + per-type factor)** | **10.36%** | **7.47%** |
| original (uncalibrated) | 22.44% | 15.26% |
| original + per-type factor | 11.93% | - |
| Holtrop uncalibrated | 19.16% | - |

- Paired regime vs original: **98 wins / 48 losses / 0 ties**, two-sided sign-test
  **p = 4.3e-5**.
- Per type (regime vs original, wins/n): battleship 9.65% vs 17.42% (24/32);
  destroyer 10.69% vs 42.01% (33/38); carrier 11.24% vs 21.49% (20/32);
  **cruiser 9.95% vs 9.88% (21/44)** - essentially tied after the slender-cruiser
  cases were added.

The modern model is clearly better overall and remains far ahead on
battleships, destroyers and carriers; cruisers are now essentially tied.  The
optional blended model (below) uses this by giving the original more weight on
the larger/slower types.

### Cruiser diagnosis (analysis at 123 cases; now essentially tied)

Diagnosis (from `regime-residuals` plus a feature-correlation study):

- The regime model is `Holtrop x per-type constant factor`, so it can correct
  the average cruiser bias but not any trend *within* the type.
- For cruisers, `|regime error|` correlates with `ln(displacement)` (+0.45):
  the larger/faster ships are the problem (Takao 40%, Wichita 30%, Graf Spee 21%,
  Alaska 18%, Pensacola 17%).  Holtrop-Mennen was regressed mostly on slower,
  fuller hulls; its wave-resistance term is weaker for fine, fast cruiser hulls
  (Cb ~0.5, Fn ~0.4), where the original SpringSharp V^4 fit happens to match the
  documented trials better.

Candidate extra parameters (nested LOOCV, 123 cases at the time of the analysis):

| model | overall | macro | curated | cruiser | worst cruiser |
|---|---|---|---|---|---|
| per-type constant (current) | 10.57% | 10.60% | 8.04% | 10.39% | 40% |
| + Fn slope (shrink 4 / 8) | 10.52 / 10.51% | 10.57 / 10.56% | 8.32 / 8.29% | 10.09% | 41% |
| + displacement slope (4 / 8) | 10.88 / 10.84% | 10.93 / 10.90% | 8.23 / 8.20% | 9.92% | 36% |
| two-band displacement split | 10.60% | 10.68% | 8.51% | 9.08% | 32% |
| two-band Froude split | 10.73% | 10.78% | 8.39% | 10.35% | 39% |

No candidate improves the overall, macro and curated metrics at the same time:
each extra parameter trades one group for another.  With 123 public-reference
cases, adding features mainly risks overfitting rather than improving the model.

Conclusion: the cruiser gap is a model-form/data limitation.  The robust options
are (a) keep the current per-type model, or (b) validate a type-aware blend
(original for cruisers) by nested CV before adopting it.

### Blended Holtrop/original model (`validate-blend`)

A per-ship-type weighted blend of the regime (Holtrop-based) prediction and the
original SpringSharp + factor prediction.  The Holtrop weight is fitted on the
out-of-fold predictions and shrunk toward the global weight:

| Type | Holtrop weight | Holtrop factor | Original factor |
|---|---|---|---|
| battleship | 0.45 | 0.767 | 0.866 |
| carrier | 0.53 | 0.803 | 0.847 |
| cruiser | 0.65 | 0.919 | 0.985 |
| destroyer | 0.94 | 1.017 | 0.727 |

Nested LOOCV: **9.81%** case-weighted / 9.80% type-averaged / **7.47%** curated,
versus 10.36% for regime-only and 11.93% for original+factor.  The original
model contributes most for the larger/slower types and least for destroyers.
Apply it with `calibrate-blend <file>`.

### Frozen baseline (2026-09-27)

| Metric | Baseline (146 cases) | Regression threshold |
|---|---|---|
| **Regime nested LOOCV, case-weighted** | **~10.36%** | **<= 11.5%** |
| Regime nested, type-averaged (macro) | ~10.38% | <= 11.5% |
| **Blended regime/original, case-weighted** | **~9.81%** | **<= 10.5%** |
| **Curated core (37 cases)** | **~7.97%** (blended 7.47%) | <= 9.5% |
| Regime nested, battleship | ~9.65% | <= 12.0% |
| Regime nested, carrier | ~11.24% | <= 13.0% |
| Regime nested, cruiser | ~9.95% | <= 10.5% |
| Regime nested, destroyer | ~10.69% | <= 13.0% |
| Original uncalibrated (original model) | ~22.44% | (comparison) |
| Original + per-type factor | ~11.93% | (must be beaten) |
| Holtrop uncalibrated | ~19.16% | (must be beaten) |
| 2-way ensemble, reported (optimistic) | ~10.3% | <= 12.0% |
| Holdout MAPE, original | ~13.8% | <= 18.5% |
| Holdout MAPE, Holtrop | ~13.1% | <= 15.0% |

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

## Historical data coverage (audited 2026-10-17)

`validation-audit` reports which model families can actually be checked against
the built-in historical data:

| Model family | Historical references | Status |
|---|---|---|
| Speed/power (regime) | 146 cases (37 curated) | validated; nested LOOCV 10.36% case-weighted, curated core 7.97%; blended 9.81% |
| Planing (Savitsky vs empirical) | 8 public planing cases | validated; both 23.3% calibrated |
| Subsystem weights & stability (`validate-weights`) | 8 full-design presets | validated on 8 full-ship presets (disp MAPE 3.59%, armour MAPE 7.64%, GM MAPE 13.91%, armoured hull MAPE 14.72%, machinery MAPE 29.69%); minimal 146-ship speed/power cases remain 0 |
| Internal spaces (machinery/magazine/accommodation) | 8 full-design presets (physical checks) | 8/8 presets yield positive magazine & machinery volumes; minimal 146-ship cases remain 0 |
| General arrangement / evacuation / scantlings | 0 | plausibility checks only |

The 146-case speed/power dataset stores only length/beam/draught/block/speed/power
references.  A validation attempt using `simulate_case` showed the minimal
ships are non-physical for weight/space work:

- **49/146** cases give hull weight that is negative or >= displacement;
- **146/146** cases give zero magazine volume (no guns are configured);
- accommodation volume fractions are 40-90% of hull volume (implausible).

`simulate_case` is therefore used only for speed/power validation.  To validate
subsystem weights, transverse metacentric height ($GM$) and internal-space
feasibility without polluting the minimal speed/power cases, Phase 31 added
`validate-weights` (`validate_preset_weights`) against the **8 full-design
historical ship presets** in `src/presets.rs`:

### Subsystem weight & stability validation (`validate-weights`)

Compared against published naval-architecture weight breakdowns and stability
records (Roberts *The Battleship Dreadnought*; Garzke & Dulin *Battleships*;
Friedman *U.S. Cruisers* & *U.S. Destroyers*; Raven & Roberts *British Cruisers*;
Conway's *All the World's Fighting Ships*):

| Ship | Normal Disp (sim / ref) | Hull (sim / ref, err) | Armour (sim / ref, err) | Machinery (sim / ref, err) | Armament+Ammo (sim / ref, err) | GM ft (sim / ref, err) |
|---|---|---|---|---|---|---|
| *HMS Dreadnought* (1906) | 18,323 / 18,110 t | 7,513 / 6,660 t (+13%) | 6,563 / 6,050 t (+8%) | 1,663 / 2,050 t (-19%) | 2,444 / 2,050 t (+19%) | 4.8 / 5.0 ft (-5%) |
| *Bismarck* (1940) | 47,392 / 45,950 t | 20,545 / 16,800 t (+22%) | 16,458 / 18,700 t (-12%) | 3,573 / 4,050 t (-12%) | 5,212 / 4,550 t (+15%) | 9.7 / 13.1 ft (-26%) |
| *USS Iowa* (1943) | 57,320 / 55,000 t | 25,864 / 21,500 t (+20%) | 17,892 / 18,400 t (-3%) | 5,624 / 4,900 t (+15%) | 6,180 / 5,200 t (+19%) | 7.6 / 7.5 ft (+2%) |
| *IJN Yamato* (1941) | 64,871 / 65,000 t | 29,014 / 23,500 t (+23%) | 24,557 / 22,895 t (+7%) | 3,245 / 5,215 t (-38%) | 7,464 / 7,800 t (-4%) | 10.5 / 11.0 ft (-5%) |
| *USS Baltimore* (1943) | 15,919 / 15,500 t | 6,716 / 6,900 t (-3%) | 3,775 / 3,700 t (+2%) | 3,385 / 2,100 t (+61%) | 1,330 / 1,550 t (-14%) | 4.7 / 5.5 ft (-15%) |
| *HMS Belfast* (1939) | 12,022 / 11,550 t | 5,712 / 5,350 t (+7%) | 2,210 / 2,550 t (-13%) | 2,875 / 1,550 t (+85%) | 746 / 1,050 t (-29%) | 4.3 / 5.1 ft (-16%) |
| *USS Fletcher* (1942) | 2,681 / 2,500 t | 1,476 / 980 t (+51%) | 109 / 140 t (n/a) | 709 / 690 t (+3%) | 136 / 260 t (-48%) | 2.8 / 3.6 ft (-21%) |
| *IJN Shimakaze* (1943) | 3,074 / 2,900 t | 1,576 / 1,150 t (+37%) | 4 / 80 t (n/a) | 859 / 820 t (+5%) | 203 / 360 t (-44%) | 2.6 / 3.4 ft (-22%) |

Summary MAPE and physical feasibility across the 8 presets (enforced by
`preset_weights_and_stability_do_not_regress` in `tests/accuracy_regression.rs`):

- **Physical feasibility:** `8/8` positive hull weight (`0 < hull < disp`), `8/8`
  positive magazine volume, `8/8` weight-loop convergence (`<= 50` iterations).
- **Normal displacement MAPE:** **3.59%** (`<= 5.0%` regression gate).
- **Armour + barbette weight MAPE (6 armoured warships):** **7.64%** (`<= 10.0%` gate).
- **Transverse metacentric height ($GM$) MAPE (all 8 ships):** **13.91%** (`<= 16.0%` gate).
- **Hull structural weight MAPE:** **14.72%** on the 6 armoured ships (`<= 18.0%` gate),
  **21.99%** across all 8.
- **Armament + ammunition weight MAPE:** **16.68%** on the 6 armoured ships (`<= 20.0%` gate),
  **23.92%** across all 8.
- **Machinery weight diagnosis:** **38.32%** on the 6 armoured ships, **29.69%** across
  all 8 (down from 60.63% in Phase 31).  SpringSharp 3.0 originally used a single
  empirical specific-weight curve tuned to capital-ship steam plants (~35-50 shp/ton in 1940),
  over-estimating WWII fleet destroyer machinery by ~2x.  Phase 32 added
  `engine.machinery_weight_factor` (0.50 on Fletcher, 0.42 on Shimakaze) to reflect
  compact high-pressure/high-temperature destroyer steam machinery (~80-100 shp/ton),
  bringing destroyer machinery errors to +3% and +5% and restoring residual hull weights.

Per-case speed/power residuals are available with `regime-residuals`.

### Data verification tools

- `data-consistency` compares the documented displacement with the engine's
  full-load model `L*B*T*Cb/35`.  Ratios are normally 1.0-1.5 (standard vs full
  load); outside [0.95, 1.60] means the dimensions, block coefficient and
  displacement disagree.  It flagged the carrier beam errors fixed in Phase 16.
- `data-outliers` screens the 109 approximate cases against a power law fitted to
  the 37 curated cases; currently 1 case exceeds 2.5 sigma (Ranger CV, a real
  design rather than an error).

## Composite quality gate (`data-target`)

```text
composite = 0.4 * official_model_fraction
          + 0.2 * traceability_fraction
          + 0.4 * loocv_accuracy_score

loocv_accuracy_score = 1.0                       if MAPE <= 10%
                     = 0.0                       if MAPE >= 30%
                     = linear interpolation      in between
```

Baseline composite (146 public-reference cases, nested regime LOOCV 10.36%):
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
- `calibrate-*` commands still fit on all 146 cases by design; use LOOCV and
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
