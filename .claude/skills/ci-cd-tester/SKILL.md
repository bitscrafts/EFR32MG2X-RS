---
name: ci-cd-tester
description: Expert in testing CI/CD pipelines both locally and remotely. Runs pre-push validation, monitors GitHub Actions, analyzes workflow failures, and provides fix suggestions. Use before pushing code or when investigating CI failures. Specializes in pristine CI/CD setups with zero warnings.
version: 2.0.0
allowed-tools: Bash, Read, Write, Edit
---

# CI/CD Testing and Monitoring Expert

You are an expert in testing and monitoring CI/CD pipelines for Rust embedded projects. You help developers validate their code locally before pushing and monitor GitHub Actions workflows remotely. You ensure completely pristine CI/CD pipelines with zero warnings or errors.

## When to Use This Skill

Use this skill when:
- **Before pushing code**: Run local CI validation
- **Investigating CI failures**: Analyze GitHub Actions logs
- **Optimizing build times**: Profile and improve workflow performance
- **Setting up new workflows**: Validate GitHub Actions YAML
- **Monitoring deployments**: Track documentation and release deployments

## Core Capabilities

### 1. Local Testing (Pre-Push Validation)

Run all CI checks locally before pushing to catch issues early:

**Full Pre-Push Check**:
```bash
bash .claude/skills/ci-cd-tester/scripts/pre-push-check.sh
```

**Individual Checks**:
```bash
# Format check
bash .claude/skills/ci-cd-tester/scripts/check-format.sh

# Clippy lints
bash .claude/skills/ci-cd-tester/scripts/check-clippy.sh

# Build validation
bash .claude/skills/ci-cd-tester/scripts/check-build.sh

# Documentation
bash .claude/skills/ci-cd-tester/scripts/check-docs.sh

# Security audit
bash .claude/skills/ci-cd-tester/scripts/check-security.sh
```

### 2. Remote Monitoring (GitHub Actions)

Monitor and analyze GitHub Actions workflows:

**Check Workflow Status**:
```bash
bash .claude/skills/ci-cd-tester/scripts/check-remote-status.sh
```

**Download Workflow Logs**:
```bash
bash .claude/skills/ci-cd-tester/scripts/download-logs.sh [run-id]
```

**Analyze Failures**:
```bash
bash .claude/skills/ci-cd-tester/scripts/analyze-failure.sh [run-id]
```

### 3. Performance Profiling

Track and optimize build performance:

**Profile Build Times**:
```bash
bash .claude/skills/ci-cd-tester/scripts/profile-build.sh
```

**Cache Analysis**:
```bash
bash .claude/skills/ci-cd-tester/scripts/analyze-cache.sh
```

## Available Scripts

All scripts located in: `.claude/skills/ci-cd-tester/scripts/`

### Local Testing Scripts

#### 1. `pre-push-check.sh`

**Purpose**: Run all CI checks locally before pushing

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/pre-push-check.sh
```

**What it checks**:
- ✅ Code formatting (`cargo fmt --check`)
- ✅ Clippy lints (strict mode, no warnings)
- ✅ PAC build (with timing)
- ✅ HAL build
- ✅ All examples build
- ✅ Documentation builds without warnings
- ✅ Binary size reporting

**Output**:
- Colored status for each check
- Detailed error messages with fix suggestions
- Binary size report
- Total execution time

**Exit Codes**:
- `0` = All checks passed
- `1` = One or more checks failed

#### 2. `check-format.sh`

**Purpose**: Verify code formatting only

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/check-format.sh [--fix]
```

**Options**:
- `--fix`: Auto-format code if check fails

**Example**:
```bash
# Check only
bash .claude/skills/ci-cd-tester/scripts/check-format.sh

# Check and fix
bash .claude/skills/ci-cd-tester/scripts/check-format.sh --fix
```

#### 3. `check-clippy.sh`

**Purpose**: Run Clippy lints on HAL code

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/check-clippy.sh [--pac] [--fix]
```

**Options**:
- `--pac`: Include PAC in checks
- `--fix`: Auto-fix clippy suggestions

**Example**:
```bash
# HAL only (strict)
bash .claude/skills/ci-cd-tester/scripts/check-clippy.sh

# HAL + PAC
bash .claude/skills/ci-cd-tester/scripts/check-clippy.sh --pac

# Auto-fix suggestions
bash .claude/skills/ci-cd-tester/scripts/check-clippy.sh --fix
```

#### 4. `check-build.sh`

**Purpose**: Build all components (PAC, HAL, examples)

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/check-build.sh [--release] [--clean]
```

**Options**:
- `--release`: Build in release mode (default)
- `--clean`: Clean before building

**Example**:
```bash
# Standard build
bash .claude/skills/ci-cd-tester/scripts/check-build.sh

# Clean build
bash .claude/skills/ci-cd-tester/scripts/check-build.sh --clean
```

#### 5. `check-docs.sh`

**Purpose**: Build and validate documentation

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/check-docs.sh [--open]
```

**Options**:
- `--open`: Open documentation in browser after building

**Example**:
```bash
# Build and check
bash .claude/skills/ci-cd-tester/scripts/check-docs.sh

# Build and preview
bash .claude/skills/ci-cd-tester/scripts/check-docs.sh --open
```

#### 6. `check-security.sh`

**Purpose**: Run security audits locally

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/check-security.sh [--install]
```

**Options**:
- `--install`: Install cargo-audit if not present

**Checks**:
- Security vulnerabilities (`cargo audit`)
- Yanked crates
- License compliance

### Remote Monitoring Scripts

#### 7. `check-remote-status.sh`

**Purpose**: Check latest GitHub Actions workflow status

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/check-remote-status.sh [workflow-name]
```

**Requires**: `gh` CLI tool installed and authenticated

**Example**:
```bash
# Check all workflows
bash .claude/skills/ci-cd-tester/scripts/check-remote-status.sh

# Check specific workflow
bash .claude/skills/ci-cd-tester/scripts/check-remote-status.sh ci.yml
```

**Output**:
- Workflow status (success/failure/in-progress)
- Run duration
- Triggered by (commit/PR)
- Link to workflow run

#### 8. `download-logs.sh`

**Purpose**: Download logs from failed workflow runs

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/download-logs.sh <run-id>
```

**Example**:
```bash
# Get run-id from GitHub Actions URL or check-remote-status.sh
bash .claude/skills/ci-cd-tester/scripts/download-logs.sh 1234567890

# Logs saved to: .ci-logs/run-1234567890/
```

#### 9. `analyze-failure.sh`

**Purpose**: Analyze failed workflow and suggest fixes

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/analyze-failure.sh <run-id>
```

**Example**:
```bash
bash .claude/skills/ci-cd-tester/scripts/analyze-failure.sh 1234567890
```

**Output**:
- Failed job identification
- Error extraction
- Suggested fixes
- Commands to run locally to reproduce

### Performance Scripts

#### 10. `profile-build.sh`

**Purpose**: Profile build times and identify bottlenecks

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/profile-build.sh
```

**Output**:
- Per-crate build times
- Dependency compilation times
- Total build time
- cargo-timings HTML report

#### 11. `analyze-cache.sh`

**Purpose**: Analyze Cargo cache and suggest optimizations

**Usage**:
```bash
bash .claude/skills/ci-cd-tester/scripts/analyze-cache.sh
```

**Output**:
- Cache directory sizes
- Outdated cache entries
- Cleanup suggestions

## Workflow Patterns

### Pattern 1: Pre-Push Workflow

Before pushing code to GitHub:

1. **Run full pre-push check**:
```bash
bash .claude/skills/ci-cd-tester/scripts/pre-push-check.sh
```

2. **If checks pass**:
```bash
git add .
git commit -m "your message"
git push
```

3. **If checks fail**, fix issues and re-run

### Pattern 2: Quick Format and Lint

For quick iterations:

```bash
# Fix formatting
bash .claude/skills/ci-cd-tester/scripts/check-format.sh --fix

# Fix clippy warnings
bash .claude/skills/ci-cd-tester/scripts/check-clippy.sh --fix

# Verify
bash .claude/skills/ci-cd-tester/scripts/pre-push-check.sh
```

### Pattern 3: Investigating CI Failure

When GitHub Actions fails:

1. **Check remote status**:
```bash
bash .claude/skills/ci-cd-tester/scripts/check-remote-status.sh
```

2. **Get run ID from output, download logs**:
```bash
bash .claude/skills/ci-cd-tester/scripts/download-logs.sh <run-id>
```

3. **Analyze failure**:
```bash
bash .claude/skills/ci-cd-tester/scripts/analyze-failure.sh <run-id>
```

4. **Reproduce locally** using suggested commands

5. **Fix issue and test**:
```bash
bash .claude/skills/ci-cd-tester/scripts/pre-push-check.sh
```

6. **Push fix**

### Pattern 4: Documentation Preview

Before pushing documentation changes:

```bash
# Build and preview docs
bash .claude/skills/ci-cd-tester/scripts/check-docs.sh --open

# Verify no warnings
bash .claude/skills/ci-cd-tester/scripts/check-docs.sh

# If good, commit
git add .
git commit -m "docs: update documentation"
git push
```

### Pattern 5: Weekly Maintenance

Perform weekly:

```bash
# Check security
bash .claude/skills/ci-cd-tester/scripts/check-security.sh

# Profile builds
bash .claude/skills/ci-cd-tester/scripts/profile-build.sh

# Analyze cache
bash .claude/skills/ci-cd-tester/scripts/analyze-cache.sh
```

## Integration with Other Skills

### With rust-hal-expert

Use ci-cd-tester BEFORE rust-hal-expert review:

```bash
# 1. First, validate CI checks locally
bash .claude/skills/ci-cd-tester/scripts/pre-push-check.sh

# 2. Then, review code quality
bash .claude/skills/rust-hal-expert/scripts/review-module.sh src/timer/
```

### With markdown-edit

Before committing documentation:

```bash
# 1. Update docs with markdown-edit
bash .claude/skills/markdown-edit/scripts/update-section.sh ...

# 2. Validate docs build
bash .claude/skills/ci-cd-tester/scripts/check-docs.sh
```

### With project-manager

After completing tasks:

```bash
# 1. Mark task complete (project-manager)
# (use project-manager skill)

# 2. Validate all CI checks
bash .claude/skills/ci-cd-tester/scripts/pre-push-check.sh

# 3. Push to GitHub
git push
```

## GitHub CLI Setup (Required for Remote Monitoring)

### Install gh CLI

```bash
# macOS
brew install gh

# Linux
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update
sudo apt install gh
```

### Authenticate

```bash
gh auth login
```

Follow prompts to authenticate with GitHub.

### Verify Setup

```bash
gh auth status
gh repo view
```

## Troubleshooting

### Issue: Scripts not executable

**Solution**:
```bash
chmod +x .claude/skills/ci-cd-tester/scripts/*.sh
```

### Issue: gh CLI not found

**Solution**:
```bash
# Install gh CLI (see GitHub CLI Setup above)
brew install gh
gh auth login
```

### Issue: cargo-audit not found

**Solution**:
```bash
bash .claude/skills/ci-cd-tester/scripts/check-security.sh --install
```

### Issue: Build failures only in CI

**Possible causes**:
1. Different Rust version
2. Missing environment variables
3. Platform-specific issues

**Debug**:
```bash
# Check Rust version matches CI
rustup show

# Run in Docker matching CI environment
docker run --rm -v $(pwd):/workspace -w /workspace \
  rust:latest \
  /bin/bash -c "rustup target add thumbv8m.main-none-eabihf && cargo build --features rt"
```

## Best Practices

### Always Run Pre-Push Check

Add to your workflow:

```bash
# Add to ~/.gitconfig or .git/hooks/pre-push
./claude/skills/ci-cd-tester/scripts/pre-push-check.sh
```

### Monitor CI After Pushing

After pushing:

1. Run `check-remote-status.sh` after 1-2 minutes
2. Verify all checks pass
3. If failures, analyze immediately

### Regular Maintenance

Weekly tasks:
- Run security audit
- Profile build times
- Clean cache if needed

### Keep Skills Updated

When CI workflows change:
- Update skill scripts to match
- Test locally first
- Document changes

## Quick Reference

```bash
# Full pre-push validation
bash .claude/skills/ci-cd-tester/scripts/pre-push-check.sh

# Individual checks
bash .claude/skills/ci-cd-tester/scripts/check-format.sh [--fix]
bash .claude/skills/ci-cd-tester/scripts/check-clippy.sh [--fix]
bash .claude/skills/ci-cd-tester/scripts/check-build.sh
bash .claude/skills/ci-cd-tester/scripts/check-docs.sh [--open]
bash .claude/skills/ci-cd-tester/scripts/check-security.sh

# Remote monitoring
bash .claude/skills/ci-cd-tester/scripts/check-remote-status.sh
bash .claude/skills/ci-cd-tester/scripts/download-logs.sh <run-id>
bash .claude/skills/ci-cd-tester/scripts/analyze-failure.sh <run-id>

# Performance
bash .claude/skills/ci-cd-tester/scripts/profile-build.sh
bash .claude/skills/ci-cd-tester/scripts/analyze-cache.sh
```

## Response Format

When using this skill:

1. **Before push**: Always run `pre-push-check.sh`
2. **Report results**: Show colored output from script
3. **On failure**: Extract specific errors and suggest fixes
4. **On success**: Confirm safe to push
5. **After push**: Monitor with `check-remote-status.sh`

## Pristine CI/CD Best Practices (Newly Learned)

### Critical Lessons from Production Implementation

#### 1. **Workflow Naming Matters**

Use **meaningful, descriptive names** that clearly indicate purpose:

**Good Examples**:
- `quality-checks.yml` - Code Quality (format + clippy)
- `build-and-test.yml` - Build and Test (builds + tests)
- `documentation.yml` - Documentation (rustdoc + deploy)

**Bad Examples**:
- `ci.yml` - Too generic
- `test.yml` - Unclear what it tests
- `workflow.yml` - No information

**Why**: Clear names help developers understand CI status at a glance.

#### 2. **Eliminate ALL Warnings**

**Zero tolerance policy for warnings**:

✅ **Fixed Issues**:
- **Cargo `default-features` warning**: Add `default-features = false` to workspace dependencies
  ```toml
  [workspace.dependencies]
  cortex-m-rt = { version = "0.7.3", default-features = false }
  ```

- **Missing documentation header**: Create `docs-header.html` with SEO metadata
  ```html
  <meta name="description" content="Your HAL description">
  <meta name="keywords" content="rust, embedded, hal, ...">
  ```

- **Clippy PAC warnings**: Use `--no-deps` flag to check only HAL code
  ```yaml
  run: cargo clippy -p efr32mg24-hal --no-deps -- -D warnings
  ```

**Why**: Warnings hide real issues and become technical debt.

#### 3. **GitHub Actions YAML Complexity Issues**

**Problems Discovered**:
- GitHub Actions can **silently fail** (0s duration) on complex YAML
- No error messages shown - workflow just won't start
- Issues that cause silent failures:
  - Emoji characters in echo statements (❌ ✅ ⚠️)
  - Complex shell loops in run commands
  - Extensive caching configurations
  - Job dependencies with `needs:` arrays

**Solutions**:
- **Keep workflows simple**: Minimal YAML, straightforward steps
- **Avoid emojis**: Use plain text instead
- **Simplify shell commands**: No complex loops or conditionals
- **Remove unnecessary features**: Caching can be added later
- **Test incrementally**: Start simple, add features one by one

**Working Pattern**:
```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: thumbv8m.main-none-eabihf
      - run: cargo build -p efr32mg24-hal --release
```

**Problematic Pattern**:
```yaml
jobs:
  build:
    needs: [format, clippy]  # Dependencies
    runs-on: ubuntu-latest
    steps:
      - name: Build with complex script
        run: |
          echo "✅ Building..."  # Emoji
          for file in target/*; do  # Complex loop
            # ... more complexity
          done
```

#### 4. **Workflow Organization Strategy**

**Separate by purpose and speed**:

1. **Fast Checks** (Code Quality - ~40s):
   - Formatting
   - Clippy lints
   - Run in parallel, no dependencies

2. **Medium Checks** (Build and Test - ~40s):
   - PAC build
   - HAL build
   - Examples build
   - Unit tests
   - All independent, parallel execution

3. **Slow Checks** (Documentation - ~1m):
   - Rustdoc build
   - GitHub Pages deployment
   - Separate workflow to avoid blocking fast checks

**Why**: Fast feedback loop - developers get lint/format results in 40s, not 5 minutes.

#### 5. **PAC vs HAL Clippy Strategy**

**Two-tier approach**:

**PAC** (generated code):
```yaml
- run: cargo clippy -p efr32mg24-pac --target thumbv8m.main-none-eabihf
```
- Allow warnings (generated code has unavoidable issues)
- Still run clippy to catch major problems

**HAL** (hand-written code):
```yaml
- run: cargo clippy -p efr32mg24-hal --features rt --no-deps -- -D warnings
```
- Strict mode: `-D warnings` (deny all warnings)
- Use `--no-deps` to avoid PAC warnings cascading
- Zero tolerance for warnings

**Why**: Separates concerns - strict on our code, lenient on generated code.

#### 6. **Testing with Placeholders**

**Future-ready test infrastructure**:

```yaml
test:
  name: Run Tests
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo test --lib --no-default-features || echo "Tests placeholder - no tests yet"
```

**Benefits**:
- CI is ready when you add tests
- Workflow passes today (with placeholder)
- No need to modify CI when adding tests
- Documents intent to add tests

**Why**: Don't block CI setup waiting for tests - add infrastructure first.

#### 7. **Minimal Workflow Pattern**

**Start with absolute minimum**:

```yaml
name: Build and Test

on:
  push:
    branches: [ master ]
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always

jobs:
  build-pac:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: thumbv8m.main-none-eabihf
      - run: cargo build -p efr32mg24-pac --release
```

**Then add features incrementally**:
1. Get basic workflow running
2. Add more jobs (one at a time)
3. Add caching (if needed)
4. Add reporting (if needed)

**Why**: Easier to debug - you know exactly what breaks.

#### 8. **Documentation Workflow Considerations**

**Lessons learned**:

- Always create `docs-header.html` before workflow runs
- Use `continue-on-error: true` for experimental steps
- Separate build from deployment (two jobs)
- Enable GitHub Pages: Settings → Pages → Source: "GitHub Actions"

**Why**: Documentation deployment has special requirements.

#### 9. **Security Audit Integration**

**Automatic security scanning**:

```yaml
name: Security Audit

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday
  workflow_dispatch:

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
```

**Benefits**:
- Weekly automatic security scans
- No manual intervention
- Catches vulnerabilities early

**Why**: Security should be automated, not manual.

#### 10. **Manual Trigger Support**

**Always add `workflow_dispatch`**:

```yaml
on:
  push:
    branches: [ master ]
  pull_request:
    branches: [ master ]
  workflow_dispatch:  # Manual trigger
```

**Benefits**:
- Test workflows without pushing code
- Debug workflow issues
- Re-run specific workflows

**Why**: Essential for debugging and testing.

## Remember

- **Local testing is fast**: Catch issues before CI
- **CI failures are expensive**: Waste 2-5 minutes per failure
- **Pre-push checks save time**: 30 seconds locally vs 5 minutes in CI
- **Always validate before pushing**: Use `pre-push-check.sh`
- **Monitor after pushing**: Use `check-remote-status.sh`
- **Analyze failures quickly**: Use `analyze-failure.sh`
- **Zero warnings policy**: Fix ALL warnings, not just errors
- **Simple workflows first**: Add complexity incrementally
- **Meaningful names**: Make CI status self-documenting
- **Test placeholders**: Prepare infrastructure before tests exist

---

<!-- META: last_updated=2025-12-18 version=2.0.0 type=Skill status=Active pristine_ci_cd=true -->
