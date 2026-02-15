# Binary Signing & Notarization Setup Guide

## Overview

This guide provides step-by-step instructions for setting up code signing for Echolocate across all platforms (macOS, Windows, Linux).

---

## macOS Code Signing & Notarization

### Prerequisites

1. **Apple Developer Account** ($99/year)
   - Enroll at: https://developer.apple.com/programs/
   - Complete verification (1-2 business days)

2. **Developer ID Certificate**
   - Type: "Developer ID Application"
   - Used for: Distribution outside App Store

### Step 1: Create Certificate

```bash
# Option A: Via Xcode
# 1. Open Xcode
# 2. Preferences > Accounts > Add Apple ID
# 3. Manage Certificates > "+" > Developer ID Application

# Option B: Via Developer Portal
# 1. Visit https://developer.apple.com/account/resources/certificates
# 2. Create new "Developer ID Application" certificate
# 3. Download and install in Keychain
```

### Step 2: Export Certificate for CI/CD

```bash
# Find your certificate in Keychain Access
security find-identity -v -p codesigning

# Export to .p12 file
# 1. Open Keychain Access
# 2. Find "Developer ID Application: Your Name"
# 3. Right-click > Export
# 4. Save as .p12 with password

# Convert to base64 for GitHub Secrets
base64 -i certificate.p12 -o certificate.b64

# Copy contents of certificate.b64
cat certificate.b64 | pbcopy
```

### Step 3: Configure GitHub Secrets

Navigate to: `https://github.com/YOUR_USERNAME/Echolocate/settings/secrets/actions`

Add these secrets:

```
MACOS_CERTIFICATE: <contents of certificate.b64>
MACOS_CERTIFICATE_PWD: <certificate password>
APPLE_SIGNING_IDENTITY: <certificate name, e.g., "Developer ID Application: Your Name (TEAM_ID)">
APPLE_NOTARIZATION_USER: <your Apple ID email>
APPLE_NOTARIZATION_PASSWORD: <app-specific password - see below>
APPLE_TEAM_ID: <10-character team ID>
```

### Step 4: Generate App-Specific Password

```bash
# Visit: https://appleid.apple.com/account/manage
# Security > App-Specific Passwords > Generate

# Use this password for APPLE_NOTARIZATION_PASSWORD secret
```

### Step 5: Update release.yml Workflow

The workflow in `.github/workflows/release.yml` already includes:

```yaml
- name: Import Code Signing Certificate
  env:
    MACOS_CERTIFICATE: ${{ secrets.MACOS_CERTIFICATE }}
    MACOS_CERTIFICATE_PWD: ${{ secrets.MACOS_CERTIFICATE_PWD }}
  run: |
    # Decode and import certificate
    echo $MACOS_CERTIFICATE | base64 --decode > certificate.p12

    # Create temporary keychain
    security create-keychain -p temp_password build.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p temp_password build.keychain

    # Import certificate
    security import certificate.p12 \
      -k build.keychain \
      -P "$MACOS_CERTIFICATE_PWD" \
      -T /usr/bin/codesign

    # Allow codesign to access keychain
    security set-key-partition-list \
      -S apple-tool:,apple: \
      -s -k temp_password \
      build.keychain

- name: Code Sign App
  env:
    APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
  run: |
    # Sign the app bundle
    codesign --force --deep --verify --verbose \
      --options runtime \
      --sign "$APPLE_SIGNING_IDENTITY" \
      "src-tauri/target/release/bundle/macos/Echolocate.app"

    # Verify signature
    codesign --verify --verbose=4 \
      "src-tauri/target/release/bundle/macos/Echolocate.app"

- name: Notarize App
  env:
    APPLE_NOTARIZATION_USER: ${{ secrets.APPLE_NOTARIZATION_USER }}
    APPLE_NOTARIZATION_PASSWORD: ${{ secrets.APPLE_NOTARIZATION_PASSWORD }}
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
  run: |
    # Create DMG
    hdiutil create -volname "Echolocate" \
      -srcfolder "src-tauri/target/release/bundle/macos/Echolocate.app" \
      -ov -format UDZO "Echolocate.dmg"

    # Submit for notarization
    xcrun notarytool submit "Echolocate.dmg" \
      --apple-id "$APPLE_NOTARIZATION_USER" \
      --password "$APPLE_NOTARIZATION_PASSWORD" \
      --team-id "$APPLE_TEAM_ID" \
      --wait

    # Staple notarization ticket
    xcrun stapler staple "Echolocate.dmg"

    # Verify stapling
    xcrun stapler validate "Echolocate.dmg"
```

### Step 6: Test Signing Locally

```bash
# Build app
npm run build

# Sign
codesign --force --deep --sign "Developer ID Application: Your Name" \
  "src-tauri/target/release/bundle/macos/Echolocate.app"

# Verify
codesign --verify --verbose=4 \
  "src-tauri/target/release/bundle/macos/Echolocate.app"

# Check signature
spctl -a -vv "src-tauri/target/release/bundle/macos/Echolocate.app"
```

---

## Windows Code Signing

### Prerequisites

1. **EV Code Signing Certificate** (Optional, $300-500/year)
   - Recommended providers: DigiCert, Sectigo, GlobalSign
   - Requires: Business verification (1-2 weeks)

2. **Self-Signed Certificate** (Development only)
   - Free, but shows warning to users
   - Not recommended for production

### Option A: EV Certificate (Production)

#### Step 1: Purchase Certificate

```powershell
# From DigiCert/Sectigo/GlobalSign
# 1. Purchase EV Code Signing Certificate
# 2. Complete business verification
# 3. Receive USB token or cloud HSM credentials
```

#### Step 2: Export Certificate

```powershell
# If using USB token:
# 1. Insert token
# 2. Open certmgr.msc
# 3. Export certificate with private key to .pfx

# Convert to base64 for GitHub
[Convert]::ToBase64String([IO.File]::ReadAllBytes("certificate.pfx")) | Out-File certificate.b64
```

#### Step 3: Configure GitHub Secrets

```
WINDOWS_CERTIFICATE_BASE64: <contents of certificate.b64>
WINDOWS_CERTIFICATE_PASSWORD: <certificate password>
WINDOWS_SIGNING_IDENTITY: <certificate subject name>
```

#### Step 4: Sign in CI/CD

Add to `.github/workflows/release.yml`:

```yaml
- name: Import Windows Certificate
  env:
    WINDOWS_CERTIFICATE_BASE64: ${{ secrets.WINDOWS_CERTIFICATE_BASE64 }}
    WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
  run: |
    # Decode certificate
    [IO.File]::WriteAllBytes(
      "certificate.pfx",
      [Convert]::FromBase64String($env:WINDOWS_CERTIFICATE_BASE64)
    )
  shell: pwsh

- name: Sign Windows Binaries
  env:
    WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
    WINDOWS_SIGNING_IDENTITY: ${{ secrets.WINDOWS_SIGNING_IDENTITY }}
  run: |
    # Import certificate
    $cert = Import-PfxCertificate `
      -FilePath certificate.pfx `
      -CertStoreLocation Cert:\CurrentUser\My `
      -Password (ConvertTo-SecureString -String $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force)

    # Sign EXE
    & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe" sign `
      /fd SHA256 `
      /td SHA256 `
      /tr http://timestamp.digicert.com `
      /n "$env:WINDOWS_SIGNING_IDENTITY" `
      "src-tauri\target\release\Echolocate.exe"

    # Sign MSI
    & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe" sign `
      /fd SHA256 `
      /td SHA256 `
      /tr http://timestamp.digicert.com `
      /n "$env:WINDOWS_SIGNING_IDENTITY" `
      "src-tauri\target\release\bundle\msi\Echolocate_*_x64_en-US.msi"

    # Verify signatures
    & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe" verify `
      /pa "src-tauri\target\release\Echolocate.exe"
  shell: pwsh
```

### Option B: Self-Signed Certificate (Development)

```powershell
# Create self-signed certificate
$cert = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject "CN=Echolocate Development" `
  -KeyUsage DigitalSignature `
  -FriendlyName "Echolocate Dev Cert" `
  -CertStoreLocation "Cert:\CurrentUser\My" `
  -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3")

# Export certificate
Export-PfxCertificate `
  -Cert $cert `
  -FilePath dev-certificate.pfx `
  -Password (ConvertTo-SecureString -String "dev_password" -AsPlainText -Force)

# Sign with self-signed cert
signtool sign /fd SHA256 /f dev-certificate.pfx /p dev_password Echolocate.exe
```

**Note:** Self-signed certificates will show "Unknown Publisher" warning.

---

## Linux Signing (Optional)

Linux AppImages do not require code signing. Instead, use:

### SHA256 Checksums (Already Implemented)

```bash
# Generated automatically in release.yml
sha256sum Echolocate-x86_64.AppImage > CHECKSUMS.sha256

# Users verify with:
sha256sum -c CHECKSUMS.sha256
```

### GPG Signing (Optional)

```bash
# Generate GPG key
gpg --full-generate-key

# Sign AppImage
gpg --detach-sign --armor Echolocate-x86_64.AppImage

# Verify signature
gpg --verify Echolocate-x86_64.AppImage.asc Echolocate-x86_64.AppImage
```

---

## Verification

### macOS

```bash
# Verify signature
codesign --verify --verbose=4 Echolocate.app

# Check notarization
spctl -a -vv Echolocate.app

# Verify DMG stapling
xcrun stapler validate Echolocate.dmg
```

### Windows

```powershell
# Verify signature
signtool verify /pa Echolocate.exe

# Check certificate details
signtool verify /v Echolocate.exe
```

### Linux

```bash
# Verify checksum
sha256sum -c CHECKSUMS.sha256

# Verify GPG signature (if used)
gpg --verify Echolocate-x86_64.AppImage.asc
```

---

## Troubleshooting

### macOS

**Issue:** "No identity found"
```bash
# List available identities
security find-identity -v -p codesigning

# If none found, import certificate
security import certificate.p12 -k ~/Library/Keychains/login.keychain
```

**Issue:** Notarization fails
```bash
# Check notarization log
xcrun notarytool log <submission-id> \
  --apple-id "$APPLE_ID" \
  --password "$APP_PASSWORD" \
  --team-id "$TEAM_ID"
```

### Windows

**Issue:** "Certificate not found"
```powershell
# List installed certificates
Get-ChildItem Cert:\CurrentUser\My

# Re-import if needed
Import-PfxCertificate -FilePath certificate.pfx `
  -CertStoreLocation Cert:\CurrentUser\My
```

**Issue:** Timestamp server unavailable
```powershell
# Try alternate timestamp servers:
# - http://timestamp.digicert.com (primary)
# - http://timestamp.comodoca.com (alternate)
# - http://timestamp.globalsign.com (alternate)
```

---

## Cost Summary

| Platform | Certificate Type | Annual Cost | Required? |
|----------|------------------|-------------|-----------|
| **macOS** | Developer ID Application | $99 | ✅ Yes (for notarization) |
| **Windows** | EV Code Signing | $300-500 | ⚠️ Recommended |
| **Windows** | Standard Code Signing | $100-200 | ⚠️ Shows SmartScreen warning |
| **Windows** | Self-Signed | Free | ❌ Not for production |
| **Linux** | GPG Key | Free | ℹ️ Optional |

**Total Minimum:** $99/year (macOS only)
**Total Recommended:** $400-600/year (macOS + Windows EV)

---

## Implementation Checklist

### macOS
- [ ] Purchase Apple Developer account ($99)
- [ ] Create Developer ID Application certificate
- [ ] Export certificate to .p12
- [ ] Convert to base64
- [ ] Add GitHub Secrets
- [ ] Generate app-specific password
- [ ] Test signing locally
- [ ] Verify notarization works

### Windows (Production)
- [ ] Purchase EV Code Signing certificate ($300-500)
- [ ] Complete business verification
- [ ] Export certificate to .pfx
- [ ] Convert to base64
- [ ] Add GitHub Secrets
- [ ] Configure signtool in workflow
- [ ] Test signing locally
- [ ] Verify signature appears correctly

### Windows (Development)
- [ ] Create self-signed certificate
- [ ] Sign test builds
- [ ] Document warning for users

### Linux
- [ ] Verify SHA256 checksums generated
- [ ] Optional: Set up GPG signing
- [ ] Document verification for users

---

## Next Steps

1. **Acquire Certificates** (1-2 weeks for Windows)
2. **Configure GitHub Secrets** (30 minutes)
3. **Update Workflows** (already done in release.yml)
4. **Test Full Release** (1 hour)
5. **Document for Users** (add to README)

---

## References

- Apple Code Signing: https://developer.apple.com/support/code-signing/
- Apple Notarization: https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution
- Windows SignTool: https://docs.microsoft.com/en-us/windows/win32/seccrypto/signtool
- GPG Signing: https://gnupg.org/documentation/
