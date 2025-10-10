# Contributing to NozyWallet

Thank you for your interest in contributing to NozyWallet! This document provides guidelines and standards for contributing to this Zcash Orchard wallet project.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Standards](#development-standards)
- [Code Style Guide](#code-style-guide)
- [Testing Guidelines](#testing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Commit Guidelines](#commit-guidelines)
- [Branch Guidelines](#branch-guidelines)

---

## Code of Conduct

This project follows the high standards of the Zcash community. We expect all contributors to:

- Be respectful and inclusive
- Focus on technical merit
- Provide constructive feedback
- Prioritize security and privacy
- Follow responsible disclosure for security issues

---

## Getting Started

### Prerequisites

```bash
# Install Rust (1.70.0 or later)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/your-username/NozyWallet.git
cd NozyWallet

# Build the project
cargo build

# Run tests
cargo test

# Check code formatting
cargo fmt -- --check

# Run clippy lints
cargo clippy -- -D warnings
```

### Development Environment

- **Rust Version**: 1.70.0 or later
- **IDE**: VS Code with rust-analyzer, or your preferred Rust IDE
- **Required Tools**:
  - `cargo-fmt` for code formatting
  - `cargo-clippy` for linting
  - `cargo-audit` for security auditing

---

## Development Standards

### Following librustzcash Standards

NozyWallet adheres to the coding standards established by the official [librustzcash](https://github.com/zcash/librustzcash) repository. Key principles include:

1. **Type Safety**: Leverage Rust's type system to prevent errors at compile time
2. **Error Handling**: Use `Result` types with descriptive error messages
3. **Documentation**: Document all public APIs with rustdoc comments
4. **Testing**: Write comprehensive unit and integration tests
5. **Security**: Follow secure coding practices, especially for cryptographic operations

### Code Organization

```
NozyWallet/
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # CLI entry point
│   ├── hd_wallet.rs        # HD wallet implementation
│   ├── orchard_tx.rs       # Orchard transaction building
│   ├── zebra_integration.rs # Blockchain RPC integration
│   ├── notes.rs            # Note management
│   ├── storage.rs          # Wallet persistence
│   ├── error.rs            # Error types
│   └── bin/                # Additional binaries
├── tests/                  # Integration tests
└── examples/               # Usage examples
```

---

## Code Style Guide

### Rust Formatting

**All code must be formatted with `rustfmt` using the project's configuration.**

```bash
# Format all code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

### Naming Conventions

Following librustzcash patterns:

```rust
// Types: PascalCase
pub struct OrchardTransactionBuilder { }
pub enum NozyError { }

// Functions and methods: snake_case
pub fn build_single_spend() -> NozyResult<Vec<u8>> { }
pub async fn get_best_block_height(&self) -> NozyResult<u32> { }

// Constants: SCREAMING_SNAKE_CASE
const DEFAULT_FEE_ZATOSHIS: u64 = 10_000;

// Modules: snake_case
mod zebra_integration;
mod hd_wallet;
```

### Documentation

**All public items must have documentation comments:**

```rust
/// Builds an Orchard transaction for spending notes.
///
/// # Arguments
///
/// * `zebra_client` - Client for blockchain RPC calls
/// * `spendable_notes` - Notes to spend in the transaction
/// * `recipient_address` - Unified address of the recipient
/// * `amount_zatoshis` - Amount to send in zatoshis
/// * `fee_zatoshis` - Transaction fee in zatoshis
/// * `memo` - Optional memo bytes (up to 512 bytes)
///
/// # Returns
///
/// A `NozyResult` containing the serialized transaction bytes
///
/// # Errors
///
/// Returns an error if:
/// - Insufficient funds
/// - Invalid recipient address
/// - Blockchain communication fails
/// - Bundle building fails
///
/// # Examples
///
/// ```no_run
/// let tx = builder.build_single_spend(
///     &client,
///     &notes,
///     "u1...",
///     100_000,
///     10_000,
///     None
/// ).await?;
/// ```
pub async fn build_single_spend(
    &self,
    zebra_client: &ZebraClient,
    spendable_notes: &[SpendableNote],
    recipient_address: &str,
    amount_zatoshis: u64,
    fee_zatoshis: u64,
    memo: Option<&[u8]>,
) -> NozyResult<Vec<u8>> {
    // Implementation
}
```

### Error Handling

**Use the `NozyResult` type alias and provide context:**

```rust
// Good: Descriptive error with context
let note_cmx: orchard::note::ExtractedNoteCommitment = note_commitment.into();
let bytes = note_cmx.to_bytes();

if bytes.len() != 32 {
    return Err(NozyError::NoteCommitment(
        format!("Expected 32 bytes, got {}", bytes.len())
    ));
}

// Bad: Generic error without context
let bytes = note_cmx.to_bytes();
if bytes.len() != 32 {
    return Err(NozyError::InvalidOperation("Wrong size".to_string()));
}
```

### Type Safety

**Prefer strong typing over primitive types:**

```rust
// Good: Type-safe wrapper
pub struct ZatoshiAmount(u64);

impl ZatoshiAmount {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    
    pub fn as_zatoshis(&self) -> u64 {
        self.0
    }
}

// Avoid: Bare primitives everywhere
pub fn send_money(amount: u64) { /* ... */ }
```

---

## Testing Guidelines

### Unit Tests

**Write unit tests for all non-trivial functions:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_commitment_conversion() {
        let note = create_test_note();
        let commitment = note.commitment();
        let note_cmx: ExtractedNoteCommitment = commitment.into();
        let bytes = note_cmx.to_bytes();
        
        assert_eq!(bytes.len(), 32);
    }

    #[tokio::test]
    async fn test_unified_address_parsing() {
        let address = "u1test...";
        let result = parse_unified_address(address);
        
        assert!(result.is_ok());
        let orchard_receiver = result.unwrap();
        assert!(orchard_receiver.is_some());
    }
}
```

### Integration Tests

**Place integration tests in the `tests/` directory:**

```rust
// tests/transaction_building.rs
use nozy::{HDWallet, OrchardTransactionBuilder, ZebraClient};

#[tokio::test]
async fn test_full_transaction_flow() {
    let wallet = HDWallet::new().unwrap();
    let client = ZebraClient::new("http://localhost:8232").unwrap();
    let builder = OrchardTransactionBuilder::new(false);
    
    // Test transaction building
    // ...
}
```

### Test Coverage

- **Aim for 80%+ code coverage**
- Test both success and error cases
- Test edge cases and boundary conditions
- Mock external dependencies (blockchain RPC)

---

## Pull Request Process

### Before Submitting

1. **Ensure your code compiles:**
   ```bash
   cargo build
   cargo test
   ```

2. **Format your code:**
   ```bash
   cargo fmt
   ```

3. **Run clippy:**
   ```bash
   cargo clippy -- -D warnings
   ```

4. **Update documentation:**
   - Add rustdoc comments for new public items
   - Update README.md if adding features
   - Add examples if introducing new APIs

### Pull Request Template

```markdown
## Description
[Describe what this PR does]

## Type of Change
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to change)
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review performed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings introduced
- [ ] Tests pass locally
```

### Review Process

1. **Automated Checks**: CI must pass (build, tests, clippy, fmt)
2. **Code Review**: At least one maintainer approval required
3. **Documentation**: Verify all public APIs are documented
4. **Testing**: Ensure adequate test coverage
5. **Security**: Review for security implications

---

## Commit Guidelines

### Commit Message Format

Following [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```bash
# Good commit messages
feat(orchard): implement note commitment conversion
fix(zebra): handle connection timeout gracefully
docs(readme): add installation instructions
refactor(wallet): simplify key derivation logic
test(notes): add unit tests for note scanning

# Bad commit messages
update stuff
fix bug
wip
asdf
```

### Commit Best Practices

- **Atomic commits**: One logical change per commit
- **Clear messages**: Explain what and why, not how
- **Reference issues**: Include issue numbers when applicable
- **Sign commits**: Use GPG signing for security-sensitive changes

---

## Branch Guidelines

### Branch Naming

```
<type>/<short-description>

# Examples
feat/unified-address-parsing
fix/rpc-connection-timeout
docs/contributing-guide
refactor/transaction-builder
```

### Branch Types

- `feat/`: New features
- `fix/`: Bug fixes
- `docs/`: Documentation
- `refactor/`: Code refactoring
- `test/`: Test additions/updates
- `chore/`: Maintenance

### Branch Workflow

1. **Create branch from `main`:**
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feat/my-feature
   ```

2. **Make changes and commit:**
   ```bash
   git add .
   git commit -m "feat(orchard): add new feature"
   ```

3. **Keep branch updated:**
   ```bash
   git fetch origin
   git rebase origin/main
   ```

4. **Push and create PR:**
   ```bash
   git push origin feat/my-feature
   # Create PR on GitHub
   ```

### Merge Strategy

- **Squash and merge**: For feature branches
- **Rebase and merge**: For clean linear history
- **Never force push**: To shared branches (except in specific cases after coordination)

---

## Security Guidelines

### Cryptographic Operations

**Follow librustzcash patterns for all cryptographic code:**

```rust
// Good: Use established libraries
use orchard::keys::SpendingKey;
use argon2::{Argon2, PasswordHasher};

let spending_key = SpendingKey::random(&mut rng);

// Bad: Don't implement custom cryptography
fn my_custom_hash(data: &[u8]) -> Vec<u8> {
    // Custom implementation - AVOID THIS
}
```

### Sensitive Data Handling

```rust
// Good: Zeroize sensitive data
use zeroize::Zeroize;

struct SecretKey {
    bytes: [u8; 32],
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

// Avoid: Leaving sensitive data in memory
let password = String::from("secret"); // String not zeroized
```

### Responsible Disclosure

**For security vulnerabilities:**

1. **DO NOT** open a public issue
2. Email security contact privately
3. Allow time for patch development
4. Coordinate public disclosure

---

## Additional Resources

### Zcash Documentation

- [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)
- [ZIP 32: Shielded Hierarchical Deterministic Wallets](https://zips.z.cash/zip-0032)
- [ZIP 316: Unified Addresses](https://zips.z.cash/zip-0316)
- [Orchard Book](https://zcash.github.io/orchard/)

### librustzcash Resources

- [librustzcash Repository](https://github.com/zcash/librustzcash)
- [zcash_primitives Documentation](https://docs.rs/zcash_primitives/)
- [orchard Documentation](https://docs.rs/orchard/)

### Rust Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Cryptography Guidelines](https://anssi-fr.github.io/rust-guide/)

---

## Questions?

- **Issues**: Open a GitHub issue for bugs or feature requests
- **Discussions**: Use GitHub Discussions for general questions
- **Security**: Email security@nozywallet.example.com for vulnerabilities

---

## License

By contributing to NozyWallet, you agree that your contributions will be licensed under the same licenses as the project:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

---

Thank you for contributing to NozyWallet and the Zcash ecosystem! 🚀

