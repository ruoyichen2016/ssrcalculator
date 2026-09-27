# Built-in and external data

## Built-in historical database

- Location: `src/accuracy.rs` (`historical_cases`)
- Purpose: validation and calibration of the Rust SpringSharp engine.
- Current size: 37 cases (battleships, carriers, cruisers, destroyers).
- Provenance: every case carries
  `Provenance { source_type, confidence, source, verification_date }`.

### Verification vs confidence

| Field | Meaning |
|---|---|
| `source_type` | How the case was obtained. **Only `OfficialTrial` and `ModelTest` count as independently verified.** |
| `confidence` | Declared, coarse confidence (0..1). It does **not** imply verification. |
| `source` | Citation / description of the reference. |

Current status: **0/37 independently verified** — all cases are
`SourceType::PublicReference` (Conway's/Jane's/class-trial summaries; displacement
and power are approximate).  `data-targets` therefore fails honestly; this is
expected until official trial or model-basin data is added.

Useful commands:

```bash
cargo run -- data-score      # mean declared confidence + verified count
cargo run -- data-progress   # verified vs public-reference counts + provenance coverage
cargo run -- data-tiers      # confidence tier counts
cargo run -- data-priority   # lowest-confidence cases first
cargo run -- data-replace-log
cargo run -- data-targets    # official-trial/model-test coverage targets (currently not met)
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
   `provenance.source_type` to `OfficialTrial` or `ModelTest`, with a real
   citation in `source`.
4. Re-run `cargo run -- validate`, `cargo run -- validate-holdout`,
   `cargo run -- data-score`, `cargo run -- data-targets`.
5. Repeat until the provenance target passes.  (The current public-reference
   database intentionally fails `data-target 0.8`.)
