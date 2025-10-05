<div align="center">

# RJQ

**A lightning-fast JSON processor written in Rust**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

</div>

## Overview

RJQ is a high-performance alternative to `jq` written in Rust, designed for processing JSON data with exceptional speed and minimal resource consumption. It provides a familiar query language for filtering, transforming, and manipulating JSON data while delivering significant performance improvements over traditional tools.

## Performance

Benchmarks against real-world data show that RJQ consistently outperforms jq:

| Query Type | Speed Improvement |
|------------|-------------------|
| Simple property access | 4-7x faster |
| Nested property access | 4-5x faster |
| Array indexing | 5-7x faster |
| Array iteration | 4-5x faster |
| Object operations | 4-6x faster |

These improvements are particularly noticeable when processing large JSON files like Terraform state files or API responses.

## Features

### Performance Optimizations
- **Zero-copy Parsing**: Minimizes memory allocations for faster processing
- **Efficient Memory Usage**: Optimized data structures reduce overhead
- **Streamlined Evaluation**: Direct execution model without intermediate representations
- **Built-in Benchmarking**: Compare performance with other JSON processors

### Query Language
- **Property Access**: `.field` or `."field name with spaces"`
- **Array Operations**: `.[0]` for indexing, `.[1:3]` for slicing, `.[]` for iteration
- **Combinators**: Pipe operator (`|`) for chaining operations
- **Filters**: `select(.field == "value")` for conditional filtering
- **Constructors**: Create objects `{key1, key2}` or arrays `[expr1, expr2]`
- **Functions**: `length`, `keys`, `map()` for data transformation

### Output Options
- **Pretty Printing**: Properly indented, readable JSON
- **Compact Mode**: Minimal whitespace for reduced output size
- **Raw Output**: Unwrapped string values for script integration
- **Colorized Display**: Syntax highlighting for better readability

### Developer Experience
- **Helpful Error Messages**: Clear feedback for query syntax issues
- **Familiar Syntax**: Easy transition for jq users
- **Debug Mode**: Detailed information for troubleshooting

## Installation

### From Cargo (Coming Soon)

```bash
cargo install rjq
```

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/rjq.git
cd rjq

# Build the project
cargo build --release

# The binary will be available at target/release/rjq
# You can copy it to a directory in your PATH
cp target/release/rjq ~/.local/bin/  # Linux/macOS
```

## Usage

### Basic Command Structure

```bash
rjq [OPTIONS] -q <QUERY> [FILE]
```

### Common Options

| Option | Description |
|--------|-------------|
| `-q, --query <QUERY>` | The query to run on the JSON input |
| `-p, --pretty` | Pretty print the output |
| `-c, --compact` | Compact output (no whitespace) |
| `-r, --raw` | Raw output (unwrap string values) |
| `-C, --color` | Colorize the output |
| `-b, --benchmark` | Show execution time |
| `--debug` | Show detailed error information |

### Input Sources

RJQ can read JSON from files or stdin:

```bash
# From a file
rjq -q '.name' input.json

# From stdin
cat input.json | rjq -q '.name'
curl -s 'https://api.example.com/data' | rjq -q '.results[]'
```

### Query Examples

```bash
# Access properties
rjq -q '.name' input.json
rjq -q '.address.city' input.json

# Array operations
rjq -q '.users[0]' input.json             # First element
rjq -q '.users[1:3]' input.json           # Slice (elements 1 and 2)
rjq -q '.users[]' input.json              # All elements

# Filtering
rjq -q '.users[] | select(.active == true)' input.json
rjq -q '.items[] | select(.price > 100)' input.json

# Transformations
rjq -q '.address | {city, state}' input.json
rjq -q '.users[] | {name, email}' input.json

# Metadata
rjq -q '.items | length' input.json
rjq -q '.config | keys' input.json
```

## Benchmarking

RJQ includes built-in benchmarking capabilities to measure performance:

```bash
# Show execution time for a query
rjq -q '.users[] | select(.active)' -b large-file.json
```

For comprehensive benchmarks against jq, use the included benchmark script:

```bash
# Run comparative benchmarks (requires hyperfine and jq)
./benches/sample-benchmarks/run_benchmarks.sh
```

### Sample Benchmark Results

The following results were obtained on a Terraform state file (~700KB):

| Query | RJQ | jq | Speedup |
|-------|-----|-----|--------|
| `.version` | 2.9ms | 19.9ms | 6.8x |
| `.resources[0].type` | 3.7ms | 18.7ms | 5.1x |
| `.resources[] | select(.type == "aws_instance")` | 4.4ms | 19.8ms | 4.5x |
| `.resources | length` | 2.8ms | 19.9ms | 7.1x |

## Comparison with jq

### Advantages of RJQ

- **Performance**: Significantly faster for most operations
- **Memory Usage**: Lower memory footprint
- **Startup Time**: Quicker for one-off queries in scripts
- **Error Messages**: Clear and helpful feedback

### Current Limitations

RJQ is under active development and doesn't yet support all jq features:

- Advanced filters and functions (e.g., `map_values`, `to_entries`)
- Custom functions and variables
- Math operations
- Regular expression support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

See the [CONTRIBUTING.md](CONTRIBUTING.md) file for more details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Query Language Reference

RJQ supports a query language similar to jq with the following operators and expressions:

### Basic Operators
- `.` - Identity (returns the input unchanged)
- `.field` - Access a field in an object
- `."field name"` - Access a field with spaces or special characters
- `.[0]` - Access an array element by index
- `.[1:3]` - Array slice (from index 1 up to but not including 3)
- `..` - Recursive descent (find all nested values)
- `.[]` - Array iteration (iterate over all elements)

### Combinators
- `|` - Pipe operator (chain operations)
- `select(...)` - Filter elements based on a condition

### Constructors
- `{field1, field2}` - Create an object with specified fields
- `[expr1, expr2]` - Create an array with results of expressions

### Functions
- `length` - Get length of array, object, or string
- `keys` - Get keys of an object or indices of an array
- `map(expr)` - Apply expression to each element
```
