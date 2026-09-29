# Historical-ship use and trust boundaries

This project is a personal, historical-warship design tool.  It re-implements
the SpringSharp calculation chain and adds a modern (Holtrop-Mennen based)
regime model plus planning-level general-arrangement tooling.  It is **not**
intended for real ship design, class approval or safety decisions.

## What the data is

The built-in database has **146 public-reference historical cases**:

- **37 curated cases** (declared confidence 0.7): key magnitudes cross-checked
  against two or more public sources;
- **109 approximate cases** (0.5): bulk-imported public references, not yet
  cross-checked (including 23 slender, high-speed destroyers/light cruisers).

There is **no official trial or model-basin data**: 0/146 cases are
independently verified or traceable.  Speeds/powers are a mix of trial and
design values, and displacements are usually standard/normal while hull
dimensions are closer to full load.  Treat every number as approximate.

## What can be trusted

- **Speed/power regime estimates** for ship types and sizes represented in the
  dataset: nested LOOCV 10.36% case-weighted / 10.38% type-averaged over all
  146 cases; **7.97%** on the curated core; the optional blended Holtrop/original
  model reaches 9.81%.  The original SpringSharp model is 22.44% on the same
  data, so the modern models are a clear improvement.
- **Relative comparisons** (larger/faster/heavier) and the
  `historical-compare` command, which lists the closest historical cases.
- The curated core was reviewed line by line (`docs/DATA_REVIEW.md`): 36/37
  values confirmed and 1 corrected (Sendai length), and all 37 are dimensionally
  self-consistent (modeled/documented displacement ratio 0.95-1.59).
- Overfitting controls: nested LOOCV, a deterministic holdout, provenance tiers
  and regression tests (`tests/accuracy_regression.rs`).

## What is planning-only (not validated)

These subsystems are plausible engineering models with internal consistency
checks, but there is no historical reference data to validate them:

- structural weight (rule-style plating/stiffener estimate),
- internal spaces (machinery/magazine/accommodation volumes),
- general-arrangement GA (multi-deck, access network, IMO evacuation,
  corridor-polyline escape, rule-style scantlings),
- CFD surrogate (0 measured CFD samples; only analytic reference rows),
- planing (Savitsky vs empirical tie on 8 public cases),
- the original SpringSharp reconstruction's weight/space outputs, which are
  non-physical for the minimal simulated ships.

Use them for exploration and comparison, not as analysis of a real vessel.

## Recommended workflow

```bash
cargo run -- demo                         # built-in example
cargo run -- report ship.ssr              # text report
cargo run -- historical-compare ship.ssr  # closest historical ships
cargo run -- validate-regime              # model accuracy (nested LOOCV)
cargo run -- validation-audit             # which families have references
cargo run -- data-consistency             # dimension/displacement consistency
cargo run -- data-outliers                # data-error screen
```

## Accuracy caveats

- Errors are concentrated at extreme Froude numbers (high-speed destroyers are
  over-predicted, slow full-form ships under-predicted).
- Fn-aware corrections and blanket block-coefficient re-derivation were tested
  and rejected (they did not improve both the full set and the curated core).
- Without real trial data, ~8-10% MAPE is close to the practical ceiling for
  this dataset.

## License

MIT, personal use.  See [LICENSE](../LICENSE).
