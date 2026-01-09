# Dependabot Security Alert - What to Do

## What is Dependabot?

Dependabot is GitHub's automated security vulnerability scanner. It:
- Scans your `Cargo.toml` and `Cargo.lock` files
- Checks against the RustSec Advisory Database
- Alerts you when dependencies have known security vulnerabilities
- Can automatically create pull requests to fix vulnerabilities

## What the Alert Means

The alert at `https://github.com/LEONINE-DAO/Nozy-wallet/security/dependabot/1` means:
- One or more of your dependencies has a known security vulnerability
- GitHub detected it and created an alert
- You should review and update the affected dependency

## How to Check and Fix

### Option 1: View on GitHub (Easiest)

1. Go to: `https://github.com/LEONINE-DAO/Nozy-wallet/security/dependabot`
2. Click on the alert (#1)
3. Review the vulnerability details
4. Dependabot may have already created a PR to fix it
5. Review and merge the PR if it looks good

### Option 2: Check Locally

```bash
# Install cargo-audit if not already installed
cargo install cargo-audit

# Check for vulnerabilities
cargo audit

# Update dependencies
cargo update

# Check again
cargo audit
```

### Option 3: Update Specific Dependencies

Common dependencies that might have vulnerabilities:
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `chrono` - Time handling (known to have some vulnerabilities)
- `serde` - Serialization
- Other transitive dependencies

## Common Vulnerabilities

### chrono
- `chrono` has known vulnerabilities related to time parsing
- Consider updating to latest version or using `time` crate instead

### tokio
- Usually very secure, but check for updates

### reqwest
- HTTP client, check for updates

## Quick Fix Steps

1. **View the alert on GitHub** - See what dependency is affected
2. **Check if Dependabot created a PR** - Often it auto-creates fixes
3. **Review the PR** - Make sure it doesn't break anything
4. **Test locally** - Run `cargo test` after updating
5. **Merge the PR** - If everything looks good

## Prevention

Enable Dependabot security updates:
1. Go to repository Settings
2. Security → Code security and analysis
3. Enable "Dependabot security updates"
4. This will auto-create PRs for security fixes

## Next Steps

1. Click the alert link to see what's vulnerable
2. Review the vulnerability details
3. Update the dependency (or merge Dependabot's PR)
4. Test your application
5. Commit and push the fix
