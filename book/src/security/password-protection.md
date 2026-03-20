# Password Protection

NozyWallet uses Argon2 for secure password hashing.

## How It Works

- **Argon2**: Industry-standard password hashing algorithm
- **Salt**: Randomly generated for each wallet
- **No Plain Text**: Passwords are never stored in plain text
- **Verification**: Password verification without storing the password

## Setting a Password

When creating or restoring a wallet, you'll be prompted to set a password:

```bash
cargo run --bin nozy new
# Enter password: [your password]
# Confirm password: [your password]
```

## Password Requirements

While NozyWallet doesn't enforce strict requirements, we recommend:
- At least 12 characters
- Mix of uppercase, lowercase, numbers, and symbols
- Avoid common words or phrases
- Use a password manager

## Password Recovery

**Important**: NozyWallet cannot recover your password. If you forget it:
- You can restore your wallet using your mnemonic phrase
- The mnemonic phrase is the ultimate backup

## Security Considerations

- Passwords are hashed using Argon2 with secure parameters
- Each wallet has a unique salt
- Password verification is constant-time to prevent timing attacks
