# Contributing

We welcome contributions to NozyWallet! Here's how you can help.

## Getting Started

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Development Setup

```bash
# Clone and build
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd nozywallet
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run --bin nozy new
```

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run clippy for linting (`cargo clippy`)
- Write tests for new features
- Update documentation as needed

## Testing

### Run All Tests

```bash
cargo test
```

### Run Integration Tests

```bash
cargo test -- --ignored
```

### Run Performance Tests

```bash
cargo test performance_tests
```

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add tests for new features
4. Follow the existing code style
5. Write clear commit messages

## Next Steps

- [Project Structure](project-structure.md) - Understand the codebase
- [Testing](testing.md) - Learn about the test suite
