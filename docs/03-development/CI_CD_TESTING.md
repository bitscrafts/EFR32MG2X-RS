# CI/CD Pipeline Testing Guide

This guide explains how to test the GitHub Actions CI/CD pipeline both locally and on GitHub.

---

## Table of Contents

1. [Quick Start - View Live Workflows](#quick-start---view-live-workflows)
2. [Local Testing Before Push](#local-testing-before-push)
3. [Testing Individual Workflows](#testing-individual-workflows)
4. [Monitoring GitHub Actions](#monitoring-github-actions)
5. [Troubleshooting Common Issues](#troubleshooting-common-issues)
6. [Advanced Testing](#advanced-testing)

---

## Quick Start - View Live Workflows

### 1. Check Workflow Status on GitHub

Visit the Actions tab in your repository:
```
https://github.com/bitscrafts/EFR32MG2X-RS/actions
```

You should see three workflows:
- **CI** - Runs on every push/PR
- **Documentation** - Runs on push to master
- **Security Audit** - Runs weekly + on dependency changes

### 2. View Latest Workflow Run

After pushing, wait 1-2 minutes and refresh the Actions page. You should see:
- Workflow name and status (⏳ In Progress, ✅ Success, ❌ Failure)
- Duration and timestamp
- Triggered by your commit

Click on a workflow run to see detailed logs for each job.

---

## Local Testing Before Push

### Testing Format Check

Ensure your code passes formatting before pushing:

```bash
# Check formatting (what CI runs)
cargo fmt --all -- --check

# If it fails, auto-format the code
cargo fmt --all
```

**Expected Output**:
- ✅ No output = formatting is correct
- ❌ "Diff in ..." = needs formatting

### Testing Clippy Lints

Run the same Clippy checks as CI:

```bash
# Test PAC clippy (may have warnings we can't fix)
cargo clippy -p efr32mg24-pac --target thumbv8m.main-none-eabihf

# Test HAL clippy (MUST pass with no warnings)
cargo clippy -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf -- -D warnings
```

**Expected Output**:
- ✅ "Finished" with no warnings
- ❌ "warning:" or "error:" = fix before pushing

**Common Clippy Warnings**:
- Unused variables: Remove or prefix with `_`
- Needless borrows: Simplify references
- Complex match: Refactor to if-let

### Testing Builds

Test all build targets that CI runs:

```bash
# 1. Build PAC (takes ~2.6 minutes first time)
cargo build -p efr32mg24-pac --target thumbv8m.main-none-eabihf --release

# 2. Build HAL library
cargo build -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf --release

# 3. Build all examples
cd efr32mg24-hal
cargo build --examples --features rt --target thumbv8m.main-none-eabihf --release
cd ..

# 4. Quick check all in one command
cargo build --workspace --features rt --target thumbv8m.main-none-eabihf --release
```

**Expected Output**:
- ✅ "Finished release" with no errors
- ❌ "error:" = compilation failure

### Testing Documentation Build

Verify rustdoc builds without warnings:

```bash
# Build documentation (what CI checks)
cargo doc --no-deps -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf

# Check for warnings in output
cargo doc --no-deps -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf 2>&1 | grep -i warning

# Open docs locally to preview
cargo doc --no-deps -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf --open
```

**Expected Output**:
- ✅ No "warning:" in output
- ❌ Documentation warnings = add missing doc comments

**Common Doc Warnings**:
- Missing doc comments on public items
- Broken intra-doc links
- Invalid code examples in doc comments

### Pre-Push Checklist

Run all checks in sequence:

```bash
#!/bin/bash
# Save as: scripts/pre-push-check.sh

set -e  # Exit on first error

echo "🔍 Running pre-push checks..."

echo ""
echo "1️⃣  Checking formatting..."
cargo fmt --all -- --check

echo ""
echo "2️⃣  Running Clippy on HAL..."
cargo clippy -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf -- -D warnings

echo ""
echo "3️⃣  Building PAC..."
cargo build -p efr32mg24-pac --target thumbv8m.main-none-eabihf --release

echo ""
echo "4️⃣  Building HAL and examples..."
cargo build -p efr32mg24-hal --examples --features rt --target thumbv8m.main-none-eabihf --release

echo ""
echo "5️⃣  Building documentation..."
cargo doc --no-deps -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf 2>&1 | tee /tmp/doc-output.txt
if grep -qi "warning" /tmp/doc-output.txt; then
    echo "❌ Documentation has warnings"
    exit 1
fi

echo ""
echo "✅ All checks passed! Safe to push."
```

Make executable and run:
```bash
chmod +x scripts/pre-push-check.sh
./scripts/pre-push-check.sh
```

---

## Testing Individual Workflows

### Testing CI Workflow Locally

Use [act](https://github.com/nektos/act) to run GitHub Actions locally:

```bash
# Install act (macOS)
brew install act

# Run CI workflow locally
act push -W .github/workflows/ci.yml

# Run specific job only
act push -W .github/workflows/ci.yml -j format
act push -W .github/workflows/ci.yml -j clippy
act push -W .github/workflows/ci.yml -j build-hal
```

**Note**: Local execution may differ from GitHub runners. Use this for quick testing only.

### Testing Documentation Workflow

Build docs exactly as the workflow does:

```bash
# Simulate the docs workflow
cargo doc --no-deps \
  -p efr32mg24-hal \
  --features rt \
  --target thumbv8m.main-none-eabihf \
  --release

# Create index redirect (as workflow does)
cat > target/thumbv8m.main-none-eabihf/doc/index.html << 'EOF'
<!DOCTYPE html>
<html><head>
<meta http-equiv="refresh" content="0; url=efr32mg24_hal/index.html">
<title>Redirecting...</title>
</head><body>
<p>Redirecting to <a href="efr32mg24_hal/index.html">documentation</a>...</p>
</body></html>
EOF

# View locally (simulates GitHub Pages)
cd target/thumbv8m.main-none-eabihf/doc
python3 -m http.server 8000
# Open: http://localhost:8000
```

### Testing Security Audit Workflow

Run the same security checks locally:

```bash
# Install cargo-audit
cargo install cargo-audit --locked

# Run security audit
cargo audit --deny warnings

# Check for yanked crates
cargo audit --deny yanked

# Install cargo-outdated
cargo install cargo-outdated --locked

# Check outdated dependencies
cargo outdated --root-deps-only

# Install cargo-license
cargo install cargo-license --locked

# Check licenses
cargo license --json | jq -r '.[] | "\(.name): \(.license)"'
```

---

## Monitoring GitHub Actions

### View Workflow Run Details

1. Go to Actions tab: `https://github.com/bitscrafts/EFR32MG2X-RS/actions`
2. Click on a workflow run
3. View job details:
   - Click on each job name to see logs
   - Expand steps to see detailed output
   - Download logs for offline analysis

### Understanding Job Status

**Status Icons**:
- ⏳ Yellow circle = In progress
- ✅ Green checkmark = Success
- ❌ Red X = Failure
- ⚠️ Yellow exclamation = Warnings
- ⏭️ Gray = Skipped

**Job Dependencies**:
```
CI Workflow:
├── format (independent)
├── clippy (independent)
├── build-pac (independent)
├── build-hal (depends on build-pac)
├── docs (independent)
├── test (independent)
└── quality-gate (depends on all above)
```

### Checking Cache Performance

Look for cache hit/miss in logs:

```
Cache restored from key: Linux-pac-abc123...
```

**Cache Hit** = Much faster build (~30 seconds)
**Cache Miss** = Full build (~2.6 minutes for PAC)

### Monitoring Build Times

Track performance over time:

| Job | Expected Time | Notes |
|-----|---------------|-------|
| format | ~10 seconds | Very fast |
| clippy | ~1 minute | Depends on cache |
| build-pac | ~2.6 minutes | First run without cache |
| build-pac | ~30 seconds | With cache hit |
| build-hal | ~1 minute | After PAC cached |
| docs | ~1.5 minutes | Includes doc generation |
| test | ~30 seconds | Currently minimal tests |

---

## Troubleshooting Common Issues

### Issue 1: Formatting Check Fails

**Error Message**:
```
Diff in /path/to/file.rs at line X:
```

**Solution**:
```bash
# Auto-fix formatting
cargo fmt --all

# Commit and push
git add .
git commit -m "fix: Apply cargo fmt"
git push
```

### Issue 2: Clippy Warnings

**Error Message**:
```
warning: unused variable: `foo`
```

**Solution**:
```bash
# Run clippy to see all warnings
cargo clippy -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf

# Fix each warning, then test again
cargo clippy -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf -- -D warnings
```

**Common Fixes**:
- Prefix unused variables with `_`: `let _foo = ...`
- Remove dead code
- Simplify complex expressions
- Add `#[allow(clippy::lint_name)]` if unavoidable

### Issue 3: Build Failures

**Error Message**:
```
error[E0425]: cannot find value `x` in this scope
```

**Solution**:
```bash
# Test build locally
cargo build -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf

# Fix compilation errors
# Commit and push
```

### Issue 4: Documentation Warnings

**Error Message**:
```
warning: missing documentation for a public function
```

**Solution**:
```rust
/// Brief description of the function.
///
/// # Examples
///
/// ```no_run
/// // Example usage
/// ```
pub fn my_function() {
    // ...
}
```

### Issue 5: PAC Cache Not Working

**Symptoms**: PAC build takes 2.6 minutes every time

**Solution**:
- Check if `efr32mg24-pac/src/**` files changed
- Cache key includes hash of PAC files
- Changes invalidate cache (expected behavior)
- PAC changes are rare after initial setup

### Issue 6: Documentation Deployment Fails

**Error**: "GitHub Pages not enabled"

**Solution**:
1. Go to repository Settings
2. Navigate to Pages section
3. Under "Build and deployment", select "Source: GitHub Actions"
4. Re-run the docs workflow

### Issue 7: Security Audit Fails

**Error Message**:
```
error: Vulnerable crates found!
ID:      RUSTSEC-YYYY-XXXX
Crate:   some-crate
```

**Solution**:
```bash
# Update vulnerable dependency
cargo update -p some-crate

# Or update all dependencies
cargo update

# Verify fix
cargo audit

# Commit Cargo.lock
git add Cargo.lock
git commit -m "fix: Update dependencies for security fix"
git push
```

---

## Advanced Testing

### Testing Pull Requests

Create a test PR to verify the full pipeline:

```bash
# Create feature branch
git checkout -b test/ci-validation

# Make a small change
echo "# CI Test" >> README.md

# Commit and push
git add README.md
git commit -m "test: Validate CI pipeline"
git push -u origin test/ci-validation

# Create PR on GitHub
# Watch all checks run
# Verify quality gate passes
```

### Testing Matrix Builds (Future)

If you add matrix builds for multiple Rust versions:

```yaml
strategy:
  matrix:
    rust: [stable, beta]
```

Test locally with different Rust versions:

```bash
# Install multiple toolchains
rustup install stable
rustup install beta

# Test with stable
rustup default stable
cargo build --features rt --target thumbv8m.main-none-eabihf

# Test with beta
rustup default beta
cargo build --features rt --target thumbv8m.main-none-eabihf
```

### Performance Profiling

Track build times and optimize:

```bash
# Time full workspace build
time cargo build --workspace --features rt --target thumbv8m.main-none-eabihf --release

# Profile specific package
cargo build -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf --release --timings

# Open timing report
open target/cargo-timings/cargo-timing.html
```

### Simulating GitHub Environment

Use Docker to match GitHub runners exactly:

```bash
# Pull GitHub Actions Ubuntu image
docker pull ghcr.io/catthehacker/ubuntu:act-latest

# Run tests in container
docker run -v $(pwd):/workspace -w /workspace \
  ghcr.io/catthehacker/ubuntu:act-latest \
  /bin/bash -c "
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    rustup target add thumbv8m.main-none-eabihf
    cargo build --features rt --target thumbv8m.main-none-eabihf
  "
```

---

## Continuous Monitoring

### Set Up Notifications

Enable GitHub notifications for workflow failures:

1. Go to repository Settings → Notifications
2. Enable "Actions workflow run failures"
3. Choose notification method (email, GitHub notifications)

### Weekly Review Checklist

Every week, check:

- [ ] Security audit passed (check Actions tab)
- [ ] No outdated dependencies with known issues
- [ ] Documentation deployed successfully
- [ ] All recent PRs passed CI
- [ ] No unusual build time increases

### Monthly Tasks

Once per month:

```bash
# Update all dependencies
cargo update

# Run full test suite
./scripts/pre-push-check.sh

# Check for outdated dependencies
cargo outdated

# Review and update lockfile
git add Cargo.lock
git commit -m "chore: Update dependencies"
git push
```

---

## Quick Reference

### Essential Commands

```bash
# Before every push
cargo fmt --all -- --check
cargo clippy -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf -- -D warnings
cargo build --workspace --features rt --target thumbv8m.main-none-eabihf --release

# Security checks
cargo audit
cargo outdated

# Documentation
cargo doc --no-deps -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf --open
```

### GitHub URLs

- **Actions**: https://github.com/bitscrafts/EFR32MG2X-RS/actions
- **Documentation** (after Pages enabled): https://bitscrafts.github.io/EFR32MG2X-RS/
- **Security Advisories**: https://github.com/bitscrafts/EFR32MG2X-RS/security

---

## Next Steps

1. ✅ Run local pre-push checks before every push
2. ✅ Monitor first few workflow runs on GitHub
3. ✅ Enable GitHub Pages for automatic documentation
4. ✅ Set up email notifications for failures
5. ✅ Review security audit results weekly

---

**Last Updated**: 2025-12-13
**Maintained By**: Project Team
**CI/CD Version**: 1.0.0

<!-- META: last_updated=2025-12-13 version=1.0.0 type=Documentation status=Complete -->
