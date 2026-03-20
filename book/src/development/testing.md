# Testing

NozyWallet includes comprehensive tests for all major components.

## Running Tests

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

### Run Specific Test

```bash
cargo test test_name
```

## Test Structure

Tests are organized in `src/tests.rs` and alongside source files using `#[cfg(test)]` modules.

## Writing Tests

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Your test code
    }
}
```

### Integration Tests

Integration tests are in `tests/` directory and can be run with `--ignored` flag.

## Test Coverage

The test suite covers:
- Wallet creation and restoration
- Address generation
- Note scanning
- Transaction building
- Error handling
- Cryptographic operations
