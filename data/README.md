# Built-in and external data

## Built-in historical database

- Location: `src/accuracy.rs` (`historical_cases`)
- Purpose: validation and calibration of the Rust SpringSharp engine.
- Current size: 77 cases (battleships, carriers, cruisers, destroyers).
- Provenance: every case carries `Provenance { source_type,
  displacement_type, confidence, source, source_url, verifier,
  verification_date }`.

### Verification, traceability and confidence

| Field | Meaning |
|---|---|
| `source_type` | How the case was obtained. **Only `OfficialTrial` and `ModelTest` count as independently verified.** |
| `displacement_type` | Which displacement convention the reference power figure uses (`Normal`, `Standard`, `FullLoad`, `Trial`, `Deep`, `Unspecified`). |
| `confidence` | Declared, coarse confidence (0..1). It does **not** imply verification. |
| `source` | Citation / description of the reference. |
| `source_url` | URL for the specific source. |
| `verifier` | Person/team who checked the case. |
| `verification_date` | Date the case was checked. |

A case is **traceable** only when `source`, `source_url` and `verifier` are all
present.  Traceable + verified are the two provenance inputs to the composite
quality gate.

Current status: **0/77 independently verified, 0/77 traceable** — all cases are
`SourceType::PublicReference` with declared displacement type `Normal`
(Conway's/Jane's/class-trial summaries; displacement and power are approximate).
`data-target` therefore fails honestly; this is expected until official trial or
model-basin data with citations, URLs and verifiers is added.

Useful commands:

```bash
cargo run -- data-score      # declared confidence + verified count + composite gate
cargo run -- data-progress   # verified vs public-reference counts + traceability
cargo run -- data-tiers      # confidence tier counts
cargo run -- data-priority   # lowest-confidence cases first
cargo run -- data-replace-log
cargo run -- data-target     # composite gate + per-ship-type breakdown
cargo run -- data-targets    # composite-quality targets (currently not met)

Composite gate: `0.4*official + 0.2*traceable + 0.4*LOOCV-accuracy`.  See
`docs/VALIDATION.md` for the formula, baselines and thresholds.
```

## External data

Place user-supplied ship/model-test data in `data/examples/` or any path and import it:

```bash
cargo run -- import-data data/examples/sample_ships.csv
cargo run -- calibrate-external data/examples/sample_ships.csv
```

CSV columns:

```text
name,type,length_ft,beam_ft,draught_ft,displacement_tons,block_coefficient,speed_kt,reference_hp,confidence,source
```

## Replacement workflow

1. Run `cargo run -- data-priority` to see the lowest-confidence cases.
2. Obtain an official trial report or model-basin test for that ship.
3. Add/update the case in `historical_cases` and set
   `provenance.source_type` to `OfficialTrial` or `ModelTest`, and fill in
   `source`, `source_url`, `verifier`, `verification_date` and
   `displacement_type`.
4. Re-run `cargo run -- validate`, `cargo run -- validate-holdout`,
   `cargo run -- data-score`, `cargo run -- data-target`.
5. Repeat until the composite gate passes.  (The current public-reference
   database intentionally fails `data-target 0.8`.)
