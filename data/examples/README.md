# Example external ship datasets

Ship-case CSVs (`name,type,length_ft,beam_ft,draught_ft,displacement_tons,block_coefficient,speed_kt,reference_hp,confidence,source`):

- `sample_ships.csv`: minimal user-created examples for testing the import path.
- `more_ships.csv`: additional historical-type examples for testing merge calibration.
- `fast_ships.csv`, `escort_ships.csv`, `small_ships.csv`: small/fast-craft diagnostics.
- `planing_ships.csv`, `more_planing.csv`: planing-hull calibration examples.

CFD sample CSV (columns `fn,empirical_hp,cfd_hp[,provenance]`):

- `cfd_samples.csv`: **synthetic placeholder** (9 rows, constant `empirical_hp`).
- `cfd_reference.csv`: 30 **analytic** rows generated from Savitsky vs the
  empirical planing model (`cargo run --example generate_cfd_reference`).
  Both are below the 20-**measured**-CFD requirement; `cfd-apply` refuses them
  unless `force` is given.  Add real data with provenance `measured_cfd` to
  enable evidence use.
  Run `cargo run -- cfd status data/examples/cfd_reference.csv`.

Use with:

```bash
cargo run -- import-data data/examples/sample_ships.csv
cargo run -- calibrate-external data/examples/more_ships.csv
cargo run -- cfd-status data/examples/cfd_samples.csv
cargo run -- cfd-status data/examples/cfd_reference.csv
cargo run -- planing-validate
```
