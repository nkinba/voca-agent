# Homebrew Tap Setup Guide

## 1. Create the homebrew-tap Repository

Create a new public repository named `homebrew-tap` on GitHub:

```bash
# Create a new directory
mkdir homebrew-tap
cd homebrew-tap

# Initialize git
git init

# Create the Formula directory and copy the formula
mkdir Formula
cp /path/to/voca-agent/Formula/spread.rb Formula/

# Commit and push
git add .
git commit -m "feat: add spread formula"
git remote add origin https://github.com/nkinba/homebrew-tap.git
git push -u origin main
```

## 2. Configure GitHub Secrets

For automated formula updates, create a Personal Access Token (PAT):

1. Go to GitHub Settings > Developer settings > Personal access tokens > Tokens (classic)
2. Generate a new token with `repo` scope
3. Add the token as a repository secret named `HOMEBREW_TAP_TOKEN` in the `voca-agent` repository

## 3. Installation

Once the tap repository is set up, users can install via:

```bash
# Add the tap
brew tap nkinba/tap

# Install spread
brew install spread
```

Or in one command:

```bash
brew install nkinba/tap/spread
```

## 4. Local Installation Test

To test the formula locally before publishing:

```bash
# From the voca-agent repository root
brew install --build-from-source ./Formula/spread.rb

# Verify installation
spread --help

# Uninstall if needed
brew uninstall spread
```

## 5. Updating the Formula

When a new version is released with a `v*` tag, the GitHub Actions workflow will automatically:

1. Build and create the release
2. Compute the SHA256 of the release tarball
3. Update the formula in the `homebrew-tap` repository
