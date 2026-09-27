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

## Calibration

- Reduced form: `P = exp(b0) * Disp^b1 * V^b2 * L^b3`
- Structural form: Holtrop component multipliers
- Froude-number segmented calibration
- Confidence-weighted variants
- Ridge / Lasso regularized structural fits
- QR / SVD cross-check
- LOOCV report
- Ensemble prediction with LOOCV-optimized weights
- Bootstrap 5%-50%-95% intervals
- External CSV import / merge calibration

## Internal Space Model

- Hull volume estimate
- Machinery volume by power, era, powerplant type
- Magazine volume
- Accommodation volume by ship type
- Fuel volume
- Stores volume by range
- Void volume
- Override fields in `.ssr` / Web form

## Reporting / GUI

- Plain-text report
- HTML report
- Web console with design form
- External CSV validation / calibration in Web console

## Validation

- 37 built-in historical cases, all currently `SourceType::PublicReference`
- Explicit `Provenance { source_type, confidence, source, verification_date }` per case
- External CSV support
- `validate-holdout`: calibration fitted on 28 training cases, scored on 9 held-out cases
- LOOCV is the primary metric in `calibrate-stats`; in-sample fits are diagnostic only
- 77 unit tests
