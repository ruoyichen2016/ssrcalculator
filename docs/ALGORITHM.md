# SpringSharp Rust Algorithm Overview

## Hull / Displacement

- Normal displacement = L * B * T * Cb / 35 (long tons)
- Wetted surface uses SpringSharp's empirical formula.
- Waterplane coefficient is derived from block coefficient and transom width.

## Resistance / Power

### Original SpringSharp model

- Compact empirical friction/wave formulas ported from the C# code.

### Holtrop-Mennen style model

- Friction (ITTC line)
- Viscous form factor
- Wave resistance
- Bulbous bow resistance
- Transom resistance
- Correlation allowance

### Propulsion efficiency

- Wake fraction
- Thrust deduction
- Hull efficiency
- Open-water efficiency
- Relative rotative efficiency
- Shaft efficiency
- QPC

The breakdown (and QPC) applies to the **Holtrop-Mennen** model only.  The
original SpringSharp empirical fit outputs shaft power directly and has no
explicit propulsive coefficient; summaries/reports state that no QPC applies and
`EngineValues::qpc_applicable` records which model the breakdown belongs to.

## Calibration

- Reduced form: `P = exp(b0) * Disp^b1 * V^b2 * L^b3`
- Structural form: Holtrop component multipliers
- Froude-number segmented calibration
- Confidence-weighted variants
- Ridge / Lasso regularized structural fits (features and target normalized to
  unit RMS; lambda grid `1e-4` .. `1e4`, selected by LOOCV)
- QR / SVD cross-check
- LOOCV report
- Ensemble prediction with LOOCV-optimized weights
- Bootstrap 5%-50%-95% intervals
- External CSV import / merge calibration

## Regime model (modern default)

- Base model: Holtrop-Mennen
- Per-ship-type calibration factor `reference / simulated`, fitted to the public
  historical dataset (battleship 0.763, carrier 0.783, cruiser 0.925, destroyer 1.058)
- The original SpringSharp empirical model is retained as a comparison baseline
- Validated by **nested** LOOCV: the factor is re-fitted on each training split
  - regime (Holtrop + factor): 9.06% case-weighted / 8.49% type-averaged (52 cases)
  - original + factor: 11.53%
  - Holtrop uncalibrated: 22.24%
- Applied to a design via `calibrate-regime` (sets `resistance_model` + `power_factor`)

## Internal Space Model

- Hull volume estimate
- Machinery volume by power, era, powerplant type
- Magazine volume
- Accommodation volume by ship type
- Fuel volume
- Stores volume by range
- Void volume
- Override fields in `.ssr` / Web form
- Constraint-aware prototype optimizer with JSON output and an HTML general-arrangement report

## Reporting / GUI

- Plain-text report
- HTML report
- Web console with design form
- External CSV validation / calibration in Web console

## Planing / Savitsky

- Savitsky (1964) planing equilibrium: lift coefficient balance, centre of
  pressure, drag with ITTC-1957 friction and a propulsive coefficient
- `savitsky-hp` command for direct estimates; `planing-validate` compares it
  against the empirical planing model on the public planing cases
- Current result: calibrated Savitsky and empirical tie at 23.3% MAPE on 8
  public PT-boat/MTB/S-boat cases; empirical stays the default

## CFD surrogate provenance

- `cfd_samples.csv` columns: `fn,empirical_hp,cfd_hp[,provenance]`
- Provenance tiers: `measured_cfd`, `literature`, `analytic`, `synthetic`
- Only >= 20 `measured_cfd` samples count as evidence; `cfd-status` reports the
  mix and `cfd-apply` refuses otherwise (unless `force`)
- `data/examples/cfd_reference.csv` provides 30 `analytic` rows to exercise the
  pipeline without claiming measured CFD

## Validation

- 52 built-in historical cases, all currently `SourceType::PublicReference`
- Explicit `Provenance { source_type, displacement_type, confidence, source,
  source_url, verifier, verification_date }` per case
- Verified = official trial / model test only; traceable = citation + URL + verifier
- Composite quality gate (`data-target`): `0.4*official + 0.2*traceable +
  0.4*LOOCV-accuracy`, with per-ship-type breakdown
- `validate-holdout`: calibration fitted on 28 training cases, scored on 9 held-out cases
- Per-type LOOCV (`loocv_by_type`) feeds the per-type quality gate
- Nested-LOOCV regime accuracy (`validate-regime`, `accuracy-target`) is the primary metric;
  in-sample fits are diagnostic only
- `tests/accuracy_regression.rs` freezes overall/per-type LOOCV and holdout baselines
- 90 unit tests + 7 accuracy-regression integration tests
