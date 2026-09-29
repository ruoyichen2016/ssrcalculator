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
  historical dataset with a confidence-weighted 15% trimmed mean (curated cases
  weighted 2x): battleship 0.781, carrier 0.803, cruiser 0.920, destroyer 1.017
- The original SpringSharp empirical model is retained as a comparison baseline
- Validated by **nested** LOOCV: the factor is re-fitted on each training split
  - regime (Holtrop + factor): 10.40% case-weighted / 10.45% type-averaged (123 cases); curated core 8.09%
  - original (uncalibrated): 21.05%; with per-type factor: 11.16%
  - Holtrop uncalibrated: 19.31%
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
- Multi-deck global genetic-algorithm optimizer: per compartment a volume, a
  deck index + within-deck vertical offset, and a longitudinal position; a
  feasibility repair operator enforces volume, void, deck ordering, structural
  continuity, armament position and access-network connectivity.
- **Evacuation (IMO MSC.1/Circ.1533)**: density-flow curve
  `Fs(D) = (1 - a D) k D` with `a = 0.266`, `k = 1.4` (corridor) / `1.3`
  (stair), capped at `k/(4a)`; effective width `w - 2*0.15 m`; per-compartment
  evacuation time, a network max-flow check (1800 s), a **congestion** time
  `total / max-flow-rate` (min cut) and an edge-density check (<= 4 p/m2).
- **Assembly station**: a gene places the mustering station on the weather deck;
  each compartment has a mustering polyline/time (walking at IMO light-density
  speed plus exit queue), checked against 1800 s.
- **Deck-plan polyline**: two longitudinal side corridors at +/- B/4, so routes
  include lateral access legs; cross-deck routes go lateral -> along corridor ->
  vertical trunk -> along the upper-deck corridor, limited to 3 hops / 45 m.
- **Rule-style scantlings** (`scantling.rs`): actual deck/section areas
  (`0.85*L*B`, `0.85*B*D`); plate `t = s*sqrt(p/(3.5*sigma))`, buckling
  `s/t <= 1.9*sqrt(E/sigma_y)`, minimum thickness + 1.5 mm corrosion addition,
  standard thickness series; stiffener `Z = p*s*l^2/(8*sigma)`; bulkheads under
  the flooding head; deflection `<= l/200`.  Reported per element with
  utilisation.
- JSON/HTML report decks, access edges, occupancy, routes, evacuation and
  mustering times, congestion, edge density, escape metres, scantling
  utilisation, structure weight and the objective.

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

- 123 built-in historical cases, all currently `SourceType::PublicReference`
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
