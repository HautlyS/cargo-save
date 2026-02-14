# Contributing to cargo-save

Thank you for your interest in contributing to cargo-save! We welcome contributions from the community.

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the maintainers.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to see if the problem has already been reported. When you create a bug report, please include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples to demonstrate the steps**
- **Describe the behavior you observed and what behavior you expected**
- **Include information about your configuration**:
  - Rust version (`rustc --version`)
  - Cargo version (`cargo --version`)
  - Operating system
  - Git version (`git --version`)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- **Use a clear and descriptive title**
- **Provide a step-by-step description of the suggested enhancement**
- **Provide specific examples to demonstrate the enhancement**
- **Explain why this enhancement would be useful**

### Pull Requests

1. Fork the repository
2. Create a new branch from `main` for your changes
3. Make your changes
4. Run tests: `cargo test`
5. Ensure your code follows the project's style: `cargo fmt` and `cargo clippy`
6. Update documentation if needed
7. Add an entry to the CHANGELOG.md
8. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- A Unix-like environment (Linux, macOS, or WSL on Windows)

### Building

```bash
git clone https://github.com/HautlyS/cargo-save.git
cd cargo-save
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Running Examples

```bash
# Basic example
cargo run --example basic

# CI integration example
cargo run --example ci_integration

# Git integration example
cargo run --example git_integration

# Custom build tool example
cargo run --example custom_build_tool
```

## Style Guidelines

### Rust Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy -- -D warnings`
- Write documentation comments for all public items
- Use meaningful variable names
- Keep functions focused and concise

### Documentation

- Use `///` for public API documentation
- Use `//!` for module-level documentation
- Include examples in documentation where appropriate
- Keep the README.md up to date

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

Example:
```
Add support for Git LFS files

- Detect LFS pointer files
- Hash pointer content instead of actual file content
- Add test for LFS detection

Fixes #123
```

## Testing

### Writing Tests

- Add unit tests for new functions
- Add integration tests for new features
- Test edge cases and error conditions
- Use descriptive test names

Example:
```rust
#[test]
fn test_compute_features_hash_with_multiple_features() {
    let cache = CacheManager::new().unwrap();
    let args = vec!["--features".to_string(), "feat1,feat2".to_string()];
    let hash = cache.compute_features_hash(&args);
    
    // Hash should be deterministic
    let hash2 = cache.compute_features_hash(&args);
    assert_eq!(hash, hash2);
}
```

### Test Organization

- Unit tests: In the same file as the code they test, in a `#[cfg(test)]` module
- Integration tests: In the `tests/` directory
- Examples: In the `examples/` directory

## Project Structure

```
cargo-save/
├── Cargo.toml          # Project configuration
├── src/
│   ├── lib.rs         # Library API
│   └── main.rs        # CLI entry point
├── examples/          # Runnable examples
├── tests/             # Integration tests
├── docs/              # Additional documentation
├── .github/           # GitHub workflows
└── README.md          # Project readme
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create a git tag: `git tag -a v0.3.0 -m "Release version 0.3.0"`
4. Push the tag: `git push origin v0.3.0`
5. The CI will automatically create a release

## Getting Help

- Join discussions in GitHub Issues
- Check the documentation in the `docs/` directory
- Read the [README.md](README.md) for usage examples

## Recognition

Contributors will be acknowledged in the project README.

Thank you for contributing to cargo-save!
