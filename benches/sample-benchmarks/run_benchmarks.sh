#!/bin/bash
#
# RJQ Benchmark Script
# Compares RJQ performance against jq using hyperfine
#

set -e

# Configuration
RJQ_BIN="target/release/rjq"
JQ_BIN="jq"
RESULTS_DIR="benchmark_results"
TF_STATE="terraform.tfstate"
WARMUP_RUNS=3

# Define queries for benchmarking
QUERIES=(
  # Basic queries
  ".version"
  ".terraform_version"
  
  # Nested properties
  ".outputs.vpc_id.value"
  ".outputs.subnet_ids.value"
  ".outputs.security_group_ids.value"
  
  # Array indexing
  ".resources[0].instances[0].attributes.tags"
  ".resources[0].instances[0].attributes.id"
  
  # Object properties
  ".resources[0].type"
  ".resources[0].name"
  ".resources[0].provider"
  
  # Functions
  ".resources | length"
  ".resources | keys"
  ".resources[0].instances | length"
  
  # Array iteration
  ".resources[].type"
  
  # Filtering (advanced)
  ".resources[] | select(.type == \"aws_instance\")"
  ".resources[] | select(.type == \"aws_security_group\") | .name"
)

# Configuration checks
check_dependencies() {
  echo "Checking dependencies..."
  
  if ! command -v hyperfine &> /dev/null; then
    echo "Error: hyperfine is not installed. Please install it from https://github.com/sharkdp/hyperfine"
    exit 1
  fi
  
  if ! command -v jq &> /dev/null; then
    echo "Error: jq is not installed. Please install it from https://stedolan.github.io/jq/"
    exit 1
  fi
  
  if [ ! -f "$RJQ_BIN" ]; then
    echo "Error: RJQ binary not found at $RJQ_BIN. Please build the project first with 'cargo build --release'."
    exit 1
  fi
  
  if [ ! -f "$TF_STATE" ]; then
    echo "Error: Terraform state file not found at $TF_STATE."
    exit 1
  fi
  
  echo "All dependencies satisfied."
}

# Create results directory
setup_results_dir() {
  echo "Setting up results directory..."
  mkdir -p "$RESULTS_DIR"
  echo "Results will be saved to $RESULTS_DIR"
}

# Run benchmarks
run_benchmarks() {
  echo "Starting benchmarks..."
  echo "====================\n"
  
  for query in "${QUERIES[@]}"; do
    echo "Benchmarking query: $query"
    
    # Create a descriptive name for the result file
    result_name=$(echo "$query" | sed 's/[^a-zA-Z0-9]/_/g')
    
    # Run hyperfine benchmark
    hyperfine --warmup "$WARMUP_RUNS" \
      --export-markdown "$RESULTS_DIR/${result_name}.md" \
      --export-json "$RESULTS_DIR/${result_name}.json" \
      "$RJQ_BIN -q '$query' $TF_STATE" \
      "$JQ_BIN '$query' $TF_STATE"
      
    echo "-----------------------------------"
  done
}

# Generate summary report
generate_summary() {
  echo "Generating summary report..."
  
  summary_file="$RESULTS_DIR/summary.md"
  
  echo "# RJQ vs jq Benchmark Results" > "$summary_file"
  echo "" >> "$summary_file"
  echo "Benchmarks comparing RJQ with jq on Terraform state file queries." >> "$summary_file"
  echo "" >> "$summary_file"
  echo "_Generated on $(date)_" >> "$summary_file"
  echo "" >> "$summary_file"
  
  echo "## System Information" >> "$summary_file"
  echo "" >> "$summary_file"
  echo "- OS: $(uname -s)" >> "$summary_file"
  echo "- RJQ Version: $("$RJQ_BIN" --version 2>/dev/null || echo 'unknown')" >> "$summary_file"
  echo "- jq Version: $("$JQ_BIN" --version 2>/dev/null || echo 'unknown')" >> "$summary_file"
  echo "" >> "$summary_file"
  
  echo "## Results" >> "$summary_file"
  echo "" >> "$summary_file"
  
  for query in "${QUERIES[@]}"; do
    result_name=$(echo "$query" | sed 's/[^a-zA-Z0-9]/_/g')
    echo "### Query: \`$query\`" >> "$summary_file"
    echo "" >> "$summary_file"
    
    if [ -f "$RESULTS_DIR/${result_name}.md" ]; then
      # Extract just the table from the markdown file
      sed -n '/^|/,/^$/p' "$RESULTS_DIR/${result_name}.md" >> "$summary_file"
      echo "" >> "$summary_file"
    else
      echo "_Results not available_" >> "$summary_file"
      echo "" >> "$summary_file"
    fi
  done
  
  echo "Benchmark summary saved to $summary_file"
}

# Main execution
main() {
  echo "RJQ Benchmark Suite"
  echo "==================="
  
  check_dependencies
  setup_results_dir
  run_benchmarks
  generate_summary
  
  echo "\nBenchmarking completed successfully!"
}

# Run the main function
main
