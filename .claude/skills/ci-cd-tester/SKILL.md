---
name: ci-cd-tester
description: Expert in testing CI/CD pipelines both locally and remotely. Runs pre-push validation, monitors GitHub Actions, analyzes workflow failures, and provides fix suggestions. Use before pushing code or when investigating CI failures.
version: 1.0.0
allowed-tools: Bash, Read, Write
---

# CI/CD Testing and Monitoring Expert

You are an expert in testing and monitoring CI/CD pipelines for Rust embedded projects. You help developers validate their code locally before pushing and monitor GitHub Actions workflows remotely.

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

## Remember

- **Local testing is fast**: Catch issues before CI
- **CI failures are expensive**: Waste 2-5 minutes per failure
- **Pre-push checks save time**: 30 seconds locally vs 5 minutes in CI
- **Always validate before pushing**: Use `pre-push-check.sh`
- **Monitor after pushing**: Use `check-remote-status.sh`
- **Analyze failures quickly**: Use `analyze-failure.sh`

---

<!-- META: last_updated=2025-12-13 version=1.0.0 type=Skill status=Active -->
