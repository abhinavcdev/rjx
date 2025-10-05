# RJQ Benchmarks

This directory contains benchmarking tools and sample data for comparing RJQ performance with jq.

## Contents

- `run_benchmarks.sh`: Script to run comparative benchmarks
- `terraform.tfstate`: Sample Terraform state file for benchmarking
- `results/`: Directory containing benchmark results (created by the script)

## Running Benchmarks

To run the benchmarks:

```bash
# Make sure you're in the project root directory
cd /path/to/rjq

# Build RJQ in release mode
cargo build --release

# Run the benchmarks
./benches/sample-benchmarks/run_benchmarks.sh
```

## Requirements

- [hyperfine](https://github.com/sharkdp/hyperfine): For running comparative benchmarks
- [jq](https://stedolan.github.io/jq/): For comparison with RJQ

## Interpreting Results

The benchmark script generates both markdown and JSON reports in the `results/` directory. The summary report provides an overview of all benchmarks run.

## Adding New Benchmarks

To add new benchmarks:

1. Add your test data file to this directory
2. Update the `run_benchmarks.sh` script to include your new queries
3. Run the benchmarks

## Sample Results

| Query | RJQ | jq | Speedup |
|-------|-----|-----|--------|
| `.version` | 2.9ms | 19.9ms | 6.8x |
| `.resources[0].type` | 3.7ms | 18.7ms | 5.1x |
| `.resources[] | select(.type == "aws_instance")` | 4.4ms | 19.8ms | 4.5x |
| `.resources | length` | 2.8ms | 19.9ms | 7.1x |
