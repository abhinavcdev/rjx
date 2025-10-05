# Contributing to RJQ

Thank you for considering contributing to RJQ! This document provides guidelines and instructions to help you get started.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## How Can I Contribute?

### Reporting Bugs

Before submitting a bug report:
- Check the issue tracker to avoid duplicates
- Collect information about the issue (OS, Rust version, etc.)
- Include a minimal reproducible example if possible

When submitting a bug report, please include:
- A clear and descriptive title
- Steps to reproduce the behavior
- Expected behavior vs. actual behavior
- Any relevant logs or error messages

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, include:
- A clear and descriptive title
- Detailed explanation of the proposed functionality
- Examples of how the feature would be used
- Why this enhancement would be useful to most RJQ users

### Pull Requests

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run the tests to ensure everything works
5. Commit your changes (`git commit -m 'Add some amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Development Setup

### Prerequisites

- Rust (stable channel)
- Cargo
- Optional: jq (for benchmarking)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/rjq.git
cd rjq

# Build the project
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific tests
cargo test -- test_name
```

### Benchmarking

```bash
# Run benchmarks
cargo bench
```

## Coding Guidelines

### Rust Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html)
- Use `cargo fmt` to format your code
- Use `cargo clippy` to catch common mistakes

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters
- Reference issues and pull requests after the first line

## Documentation

- Update documentation when changing code
- Use clear and consistent language
- Include examples for new features

## Additional Notes

### Issue Labels

- `bug`: Something isn't working
- `enhancement`: New feature or request
- `documentation`: Improvements to documentation
- `good first issue`: Good for newcomers

## Questions?

Feel free to ask questions by opening an issue with the label `question`.

Thank you for contributing to RJQ!
