# Contributing to rv

Thanks for your interest in contributing!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/rv.git`
3. Create a branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Run clippy: `cargo clippy`
7. Commit your changes
8. Push and open a Pull Request

## Development

```bash
# Build
cargo build

# Run
cargo run -- .

# Run with release optimizations
cargo run --release -- .

# Check for issues
cargo clippy

# Format code
cargo fmt
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add tests for new functionality
- Keep commits focused and atomic

## Adding New File Types

To add color support for new file types, edit `src/graph.rs`:

1. Add a variant to `FileType` enum
2. Update `FileType::from_extension()` to match the extension
3. Update `FileType::color()` with the desired color

## Adding New Filtered Directories

To add directories to the default filter list, edit `src/scanner.rs`:

1. Add the directory name to the `OUTPUT_DIRS` set

## Reporting Issues

- Check existing issues first
- Include rv version (`cargo pkgid`)
- Include OS and Rust version
- Provide steps to reproduce
