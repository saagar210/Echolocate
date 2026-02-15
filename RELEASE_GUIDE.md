# Release Guide for Echolocate

## Overview

This guide covers the release process for Echolocate, including building, testing, signing, and distributing across multiple platforms (macOS, Linux, Windows).

---

## Release Process

### Step 1: Prepare Release

```bash
# Ensure all changes are committed
git status

# Update version in package.json and Cargo.toml
# Format: vX.Y.Z (e.g., v1.0.0)
VERSION="1.0.0"

# Create git tag
git tag -a "v$VERSION" -m "Release v$VERSION"

# Push tag to trigger GitHub Actions
git push origin "v$VERSION"
```

### Step 2: Monitor CI/CD

The GitHub Actions workflow will automatically:
1. Build on all platforms (macOS, Linux, Windows)
2. Run tests and linting
3. Create release artifacts
4. Generate checksums
5. Create draft GitHub release

**Status Page:** https://github.com/saagar210/Echolocate/actions

### Step 3: Verify Artifacts

Once builds complete:
```bash
# Download release artifacts
cd ~/Downloads
wget https://github.com/saagar210/Echolocate/releases/download/v$VERSION/CHECKSUMS.sha256
wget https://github.com/saagar210/Echolocate/releases/download/v$VERSION/Echolocate*.dmg
wget https://github.com/saagar210/Echolocate/releases/download/v$VERSION/Echolocate*.AppImage
wget https://github.com/saagar210/Echolocate/releases/download/v$VERSION/Echolocate*.msi

# Verify checksums
sha256sum -c CHECKSUMS.sha256
```

### Step 4: Test on Each Platform

#### macOS
```bash
hdiutil attach Echolocate-x86_64.dmg
# Opens Finder - drag Echolocate.app to Applications
# Test: Open Applications > Echolocate
# Test: Check System Preferences > Privacy & Security
```

#### Linux
```bash
chmod +x Echolocate-x86_64.AppImage
./Echolocate-x86_64.AppImage
# Verify: App launches and scans work
# Verify: Database is created in ~/.config/echolocate/
```

#### Windows
```powershell
# Run as Administrator
.\Echolocate-Setup.exe
# Follow installer wizard
# Verify: App launches from Start Menu
# Verify: Scan functionality works
```

### Step 5: Publish Release

```bash
# Go to https://github.com/saagar210/Echolocate/releases
# Find the draft release
# Edit release notes (auto-generated from commits)
# Click "Publish release"
```

### Step 6: Post-Release

```bash
# Announce on:
# - GitHub Discussions
# - Twitter/X @EcholocateApp
# - Product Hunt (if major release)
# - HackerNews (if interesting)

# Monitor for issues:
# - GitHub Issues
# - Support email
# - Discord/Community channel
```

---

## Automated CI/CD Workflows

### Release Workflow (`.github/workflows/release.yml`)

**Triggers:** Git tag push matching `v*.*.*`

**Steps:**
1. Create GitHub release (draft)
2. Build on Linux (AppImage format)
3. Build on macOS (DMG format, both Intel & ARM)
4. Build on Windows (MSI + NSIS installer)
5. Generate SHA256 checksums
6. Upload all artifacts to release

**Time:** ~30-45 minutes (first build), ~10-15 minutes (cached)

### Frontend CI (`.github/workflows/frontend-ci.yml`)

**Triggers:** Push to main/develop, PR to main/develop

**Steps:**
1. Lint (ESLint)
2. Type check (TypeScript)
3. Unit tests (Vitest)
4. Build (Vite)
5. Coverage report

**Time:** ~5 minutes

### Backend CI (`.github/workflows/backend-ci.yml`)

**Triggers:** Push to main/develop, PR to main/develop

**Steps:**
1. cargo test
2. cargo clippy
3. cargo fmt --check
4. cargo audit

**Time:** ~10 minutes (cold), ~2 minutes (cached)

---

## Build Requirements

### Local Build (for testing releases locally)

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
brew install node

# Build
npm install
npm run build
```

#### Linux (Ubuntu/Debian)
```bash
# Install dependencies
sudo apt-get install -y \
  build-essential \
  libgtk-3-dev \
  libwebkit2gtk-4.0-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Build
npm install
npm run build
```

#### Windows
```powershell
# Install Visual Studio Build Tools (as admin)
choco install visualstudio2022-workload-nativedesktop -y

# Install Rust
curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
choco install nodejs -y

# Build
npm install
npm run build
```

---

## Version Numbering

Follow Semantic Versioning: `MAJOR.MINOR.PATCH`

### Major (X.0.0)
- Breaking API changes
- Significant new features
- Major UI overhaul

### Minor (1.X.0)
- New features (backward compatible)
- Major bug fixes
- Performance improvements

### Patch (1.0.X)
- Bug fixes
- Documentation updates
- Security patches

### Example
```
v1.0.0 - Initial stable release
v1.1.0 - IPv6 support added
v1.1.1 - Bug fix for IPv6 scanning
v2.0.0 - Complete UI redesign
```

---

## Release Notes Template

```markdown
# v1.0.0 - Release Name

**Released:** 2024-01-15

## What's New

### 🎉 Features
- Custom alert rules with complex conditions
- IPv6 network discovery support
- Performance optimizations (5-10x faster)
- Database query caching layer

### 🐛 Bug Fixes
- Fixed device departed detection
- Fixed concurrent scan issues
- Fixed IPv4 CIDR matching edge cases

### 🚀 Performance
- Query result caching (30x faster for device list)
- Optimized database indexes
- IPv6 link-local filtering

### ⚙️ Other
- Updated dependencies (security)
- Improved error messages
- Better cross-platform support

## Installation

### macOS
Download: [Echolocate-x86_64.dmg](...)
- Intel: Echolocate-x86_64.dmg
- Apple Silicon: Echolocate-aarch64.dmg

### Linux
Download: [Echolocate-x86_64.AppImage](...)

### Windows
Download: [Echolocate-Setup.exe](...)

## Verification

All downloads are signed and checksummed:
```bash
sha256sum -c CHECKSUMS.sha256
```

## Known Issues
- See [GitHub Issues](...)

## Upgrading
- macOS: Replace app in Applications
- Linux: Download new AppImage
- Windows: Run installer (uninstalls old version)

## Support
- [GitHub Issues](...)
- [Documentation](...)
- [Community Discord](...)
```

---

## Code Signing & Notarization

### macOS Code Signing (Phase 5.2)

```bash
# Set environment variables
export APPLE_CERTIFICATE_PASSWORD="..."
export APPLE_SIGNING_IDENTITY="..."

# Build with signing
npm run build -- --sign

# Notarize with Apple
xcrun altool --notarize-app \
  -f "Echolocate.dmg" \
  -t osx \
  -u "apple@example.com" \
  -p "app-specific-password"
```

### Windows Code Signing (Phase 5.2)

```powershell
# Set certificate path
$certPath = "C:\path\to\certificate.pfx"
$certPassword = "..."

# Sign executable
signtool sign /f $certPath /p $certPassword /d "Echolocate" /du "https://echolocate.app" /t http://timestamp.comodoca.com/authenticode Echolocate.exe
```

---

## Installer Generation

### macOS DMG
- Uses `create-dmg` tool
- Copies app from build output
- Adds license agreement
- Creates background image with icon

### Linux AppImage
- Uses `linuxdeploy` tool
- Bundles all dependencies
- Single executable file
- No installation required

### Windows MSI/NSIS
- Tauri built-in support
- Creates Start Menu shortcuts
- Registers file associations
- Uninstall support

---

## Monitoring & Analytics

### Post-Release
- Download statistics (GitHub)
- GitHub Issues (bug reports)
- User feedback
- Performance metrics

### Release Health
- First week: Monitor for critical bugs
- First month: Gather feature requests
- Quarterly: Plan next release

---

## Troubleshooting

### Build Failures

**macOS:**
```bash
# Clear cache
rm -rf target/release
rm -rf src-tauri/target

# Rebuild
npm run build
```

**Linux:**
```bash
# Install missing dependencies
sudo apt-get install libgtk-3-dev libwebkit2gtk-4.0-dev

# Rebuild
npm run build
```

**Windows:**
```powershell
# Visual Studio Build Tools not found
choco install visualstudio2022-workload-nativedesktop

# Rebuild
npm run build
```

### Signing Issues

```bash
# List available certificates (macOS)
security find-identity -v -p codesigning

# Test signature (macOS)
codesign -vvv Echolocate.app

# Verify signature (Windows)
signtool verify /pa Echolocate.exe
```

---

## Rollback Procedure

If critical bug found in release:

```bash
# Delete tag
git tag -d v1.0.0
git push origin :refs/tags/v1.0.0

# Delete GitHub release
# https://github.com/saagar210/Echolocate/releases

# Fix bug and re-release with patch
git tag -a v1.0.1 -m "Critical bugfix"
git push origin v1.0.1
```

---

## Appendix: Checklist

### Pre-Release
- [ ] All tests passing
- [ ] No linting errors
- [ ] Version updated in package.json and Cargo.toml
- [ ] Changelog updated
- [ ] Security audit passed (`cargo audit`)

### Release
- [ ] Tag created and pushed
- [ ] GitHub Actions workflows successful
- [ ] Artifacts downloaded and verified
- [ ] Tested on all platforms
- [ ] GitHub release published

### Post-Release
- [ ] Announcement posted
- [ ] Monitoring enabled
- [ ] Support channels notified
- [ ] Analytics reviewed
- [ ] Next release planned
