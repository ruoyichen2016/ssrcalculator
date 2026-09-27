# Data changelog

## built-in-v1.5 (2026-09-27) — provenance honesty fix

- All 37 built-in cases are now explicitly `SourceType::PublicReference`.
- Removed the hardcoded `DATA_UPGRADES` / `case_source_confidence` table.
- **Correction:** the "verified fraction" figures in v1–v1.4 below measured
  self-declared confidence, not independent verification.  Under the corrected
  definition (official trial / model-basin test only) the database is
  **0/37 independently verified**.
- `data-targets` now measures official-trial/model-test coverage and fails
  honestly instead of passing on a hardcoded confidence list.
- Added a deterministic holdout split (`validate-holdout`): calibration is
  fitted on 28 training cases and scored on 9 held-out cases.
- LOOCV is now the primary accuracy metric; in-sample MAPE is labelled as a
  diagnostic, optimistic number.


## built-in-v1 (initial)

- 30 historical cases
- 14 high-confidence ("well-documented public historical references")
- 16 medium-confidence ("public references; approximate displacement/power")
- verified fraction: 14/30 = 46.7%

## built-in-v1.1

Upgraded 10 cases to high confidence after reviewing additional public
references:

- HMS Dreadnought (1906)
- Hood (1920)
- Baltimore CA (1943)
- Cleveland CL (1942)
- Brooklyn CL (1937)
- Alaska CB (1944)
- Zara class CA (1931)
- Gneisenau (1938)
- Dido CL (1940)
- Tribal class DD (1938)

Result:

- verified fraction: 24/30 = 80.0%
- `data-target 0.8` passed

## built-in-v1.2

Upgraded 4 more cases:

- Kongo (refit 1936)
- Atlanta class CL (1942)
- Independence CVL (1943)
- Bogue CVE (1943)

Result:

- verified fraction: 28/30 = 93.3%
- `data-target 0.9` passed

## built-in-v1.3

Upgraded the final 2 cases after checking public references:

- Somers DD (1937): displacement 1,840 t, 36 kn design, 52,000 shp
- Benson DD (1942): 1,620 t, 37.5 kn, 50,000 shp

Result:

- verified fraction: 30/30 = 100%
- all built-in cases are now high-confidence

## built-in-v1.4 (superseded by v1.5)

Added 7 new well-documented classes using public references:

- Porter class DD (1936)
- Sims class DD (1939)
- Mahan class DD (1936)
- New Orleans class CA (1934)
- Kuma class CL (1920)
- Sendai class CL (1924)
- Admiral Hipper class CA (1939)

Result:

- 37 built-in cases
- verified fraction: 37/37 = 100%
- `data-target 0.9` passes

## built-in-v2 (planned)

Goals:

- replace public-reference data with official model-test data where available
- add new ship classes and more speed/power points
- keep all cases within `validate` MAPE bounds

Remaining replacement work can be listed with:

```bash
cargo run -- data-replace-log
cargo run -- data-priority
cargo run -- data-targets
```

## Alongside examples

- `sample_ships.csv`
- `more_ships.csv`
- `fast_ships.csv`
- `escort_ships.csv`
- `small_ships.csv`
- `planing_ships.csv`
- `more_planing.csv`
- `cfd_samples.csv`
