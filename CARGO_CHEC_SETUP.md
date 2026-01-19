# Cargo-Chec Setup

This project uses `cargo-chec` to run multiple Rust checks in parallel for faster feedback.

## What is cargo-chec?

`cargo-chec` is a Cargo subcommand that runs multiple checks (`cargo check`, `cargo clippy`, `cargo fmt`, `cargo test`) in parallel and outputs results as JSON. This provides faster feedback during development.

## Installation

### Automatic Installation

Run the provided script:

**Windows (PowerShell):**
```powershell
.\run_cargo_chec.ps1
```

**Linux/macOS:**
```bash
./run_cargo_chec.sh
```

### Manual Installation

```bash
cargo install cargo-chec
```

## Usage

### Basic Usage

```bash
cargo chec
```

This runs all checks in parallel and outputs JSON results.

### Human-Readable Output

```bash
cargo chec --pretty
```

### Integration in Scripts

The project includes `cargo-chec` in:

1. **Self-Audit Script** (`run_self_audit.ps1`) - Runs as part of security audit
2. **CI Workflow** (`.github/workflows/ci.yml`) - Runs in GitHub Actions
3. **Standalone Script** (`run_cargo_chec.ps1` / `run_cargo_chec.sh`) - Run checks independently

## What Gets Checked

`cargo-chec` runs these checks in parallel:

- ✅ `cargo check` - Compilation check
- ✅ `cargo clippy` - Linting
- ✅ `cargo fmt --check` - Formatting
- ✅ `cargo test` - Unit tests

## CI Integration

The GitHub Actions workflow (`.github/workflows/ci.yml`) includes a `cargo-chec` job that runs on every push and pull request.

## Benefits

1. **Faster Feedback** - Parallel execution reduces total check time
2. **JSON Output** - Easy to parse for automation and tooling
3. **Comprehensive** - Runs multiple checks in one command
4. **Consistent** - Same checks locally and in CI

## Troubleshooting

### Installation Fails

Make sure you have:
- Rust and Cargo installed (`rustc --version`)
- Network access to crates.io
- Sufficient disk space in `~/.cargo/bin`

### Checks Fail

Review the output to see which specific check failed:
- Compilation errors → fix code
- Clippy warnings → address linting issues
- Formatting issues → run `cargo fmt`
- Test failures → fix tests

## Related Tools

- `cargo-audit` - Security vulnerability scanning
- `cargo-deny` - License and dependency checking (configured in `deny.toml`)
- `cargo clippy` - Linting (runs as part of cargo-chec)
- `cargo fmt` - Code formatting (runs as part of cargo-chec)

## References

- [cargo-chec on crates.io](https://crates.io/crates/cargo-chec)
- [cargo-chec documentation](https://docs.rs/crate/cargo-chec)
