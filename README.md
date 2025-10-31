# Delivery Scheduling
** The root directory must not have any spaces in it in order for the Makefile to interpret paths properly

*Report, inputs, and outputs have been redacted from this repo

## Running

Inputs are taken from `/inputs/`
Outputs are deposited in `/output/`

### Containerized (Docker)

To run within docker, use the `Makefile`. Run the following commands from the root directory of this project.
```
make build      # Build Docker image
make run        # Run the container
make clean      # Delete old output CSV
make rebuild    # Full reset and run
```

### Not-containerized

To run outside of docker, use `cargo run` in of `/rust/`

## Testing
Tests are located in `/rust/src/tests.rs`

To run within docker, use the `Makefile`. Run `make test` from the root directory of this project.
To run tests outside of docker, use `cargo test` in of `/rust/`

## Metrics
CSVs of order counts and order records are depositied in `/output/`
Python scripts to create graphs and calculate metrics are in `/metrics/scripts/`

### Examples 
CSVs for current algorithm with 3 reserves are in `/metrics/improved/csv/`
Graphs for current algorithm with 3 reserves are in `/metrics/improved/graphs/`

## Additional files
`/rust/src/main_2FIFO.rs` is the first algorithm implemented for this problem, performs worse than the current one in `main.rs`.
`/metrics/*` are the metrics referenced in the report.

