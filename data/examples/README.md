# Example external ship datasets

- `sample_ships.csv`: minimal user-created examples for testing the import path.
- `more_ships.csv`: additional historical-type examples for testing merge calibration.

Use with:

```bash
cargo run -- import-data data/examples/sample_ships.csv
cargo run -- calibrate-external data/examples/more_ships.csv
```
