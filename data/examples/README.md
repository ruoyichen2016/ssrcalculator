# Example external ship datasets

Ship-case CSVs (`name,type,length_ft,beam_ft,draught_ft,displacement_tons,block_coefficient,speed_kt,reference_hp,confidence,source`):

- `sample_ships.csv`: minimal user-created examples for testing the import path.
- `more_ships.csv`: additional historical-type examples for testing merge calibration.
- `fast_ships.csv`, `escort_ships.csv`, `small_ships.csv`: small/fast-craft diagnostics.
- `planing_ships.csv`, `more_planing.csv`: planing-hull calibration examples.

CFD sample CSV (columns `fn,empirical_hp,cfd_hp`):

- `cfd_samples.csv`: **synthetic placeholder** (9 rows, constant `empirical_hp`).
  It exercises the CFD-surrogate pipeline but is far below the 20-sample
  minimum for evidence; `cfd-apply` refuses to use it unless `force` is given.
  Run `cargo run -- cfd status data/examples/cfd_samples.csv`.

Use with:

```bash
cargo run -- import-data data/examples/sample_ships.csv
cargo run -- calibrate-external data/examples/more_ships.csv
cargo run -- cfd-status data/examples/cfd_samples.csv
```
