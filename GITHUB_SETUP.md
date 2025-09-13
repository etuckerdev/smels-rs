# GitHub Repository Setup Guide

This guide will help you set up your SMELS repository on GitHub with all the necessary configurations.

## 1. Create GitHub Repository

1. Go to [GitHub.com](https://github.com) and sign in
2. Click the "+" icon → "New repository"
3. Repository name: `smels-rs`
4. Description: "Small Model Error Log Summarizer for multiple programming languages"
5. Make it **Public** (for open source)
6. **DO NOT** initialize with README, .gitignore, or license (we already have these)
7. Click "Create repository"

## 2. Push Your Code

```bash
# Add the GitHub remote
git remote add origin https://github.com/etuckerdev/smels-rs.git

# Push the code
git push -u origin master
```

## 3. Set Up Repository Secrets

Go to your repository Settings → Secrets and variables → Actions

### Required Secrets:

#### For Publishing to crates.io
- `CRATES_IO_TOKEN`: Your crates.io API token
  - Get it from: https://crates.io/me (Account Settings → API Tokens)
  - Create a new token with publish permissions

## 4. Update Workflow Files

✅ **Already Done!** The workflow files are already configured with the correct username (`etuckerdev`).

The following files have been set up correctly:
- `.github/workflows/release.yml` - Uses `etuckerdev` for repository owner checks
- `.github/dependabot.yml` - Uses `etuckerdev` for reviewers and assignees
- `CONTRIBUTING.md` - Uses `etuckerdev` for upstream remote URL

## 5. Enable GitHub Features

### Branch Protection (Recommended)
1. Go to Settings → Branches
2. Add rule for `master`/`main` branch:
   - Require pull request reviews
   - Require status checks (CI)
   - Include administrators

### GitHub Pages (Optional)
If you want to host documentation:
1. Go to Settings → Pages
2. Source: "GitHub Actions"

## 6. Repository Settings

### General
- ✅ Issues
- ✅ Discussions (optional)
- ✅ Projects (optional)
- ✅ Wiki (optional)
- ✅ Sponsorships (if you want donations)

### Description & Topics
Add these topics to help discoverability:
- `rust`
- `cli`
- `error-analysis`
- `debugging`
- `ai`
- `developer-tools`

## 7. First Release

Once everything is set up:

```bash
# Update version in Cargo.toml if needed
# Then create a tag
git tag v0.1.0
git push origin v0.1.0
```

This will trigger the release workflow and publish to crates.io.

## 8. Repository Badges

✅ **Already Done!** The README.md already includes the correct badges with your username:

```markdown
[![CI](https://github.com/etuckerdev/smels-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/etuckerdev/smels-rs/actions/workflows/ci.yml)
[![Security Audit](https://github.com/etuckerdev/smels-rs/actions/workflows/security.yml/badge.svg)](https://github.com/etuckerdev/smels-rs/actions/workflows/security.yml)
[![Crates.io](https://img.shields.io/crates/v/smels.svg)](https://crates.io/crates/smels)
[![Documentation](https://docs.rs/smels/badge.svg)](https://docs.rs/smels)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
```

## 9. Community Setup

- Add issue templates (bug report, feature request)
- Set up funding (GitHub Sponsors, optional)
- Create milestones for releases
- Add repository to Rust ecosystem lists

## Troubleshooting

### CI/CD Issues
- Check the Actions tab for failed runs
- Ensure all secrets are set correctly
- ✅ **Username configuration is already correct** (`etuckerdev`)

### Publishing Issues
- Make sure `CRATES_IO_TOKEN` has publish permissions
- Check that you're the owner of the crate on crates.io
- Verify version numbers are incremented

### Permission Issues
- Ensure you're the repository owner for releases
- Check branch protection settings

## Next Steps

1. Set up issue templates
2. Create a project board
3. Add repository to awesome-rust lists
4. Set up Discord/Slack for community
5. Create a website (optional)

Need help? Check the [Contributing Guide](CONTRIBUTING.md) or open an issue!
