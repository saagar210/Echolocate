#!/bin/bash
# Generate release notes from git commits since last tag

set -e

LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
CURRENT_TAG=$1

if [ -z "$CURRENT_TAG" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "# Release $CURRENT_TAG"
echo ""
echo "## What's New"
echo ""

if [ -n "$LAST_TAG" ]; then
    echo "### Features"
    git log "$LAST_TAG..$CURRENT_TAG" --oneline --grep="feat" | sed 's/^/- /'
    echo ""

    echo "### Bugfixes"
    git log "$LAST_TAG..$CURRENT_TAG" --oneline --grep="fix" | sed 's/^/- /'
    echo ""

    echo "### Performance"
    git log "$LAST_TAG..$CURRENT_TAG" --oneline --grep="perf" | sed 's/^/- /'
    echo ""

    echo "### Other Changes"
    git log "$LAST_TAG..$CURRENT_TAG" --oneline --grep -v "^(feat|fix|perf|docs|test)" | sed 's/^/- /'
else
    echo "### All Changes"
    git log --oneline | sed 's/^/- /'
fi

echo ""
echo "## Platform Support"
echo ""
echo "- ✅ macOS (Intel & Apple Silicon)"
echo "- ✅ Linux (Ubuntu 20.04+, Fedora 33+)"
echo "- ✅ Windows (10/11)"
echo ""

echo "## Installation"
echo ""
echo "### macOS"
echo '```bash'
echo "download https://github.com/saagar210/Echolocate/releases/download/$CURRENT_TAG/Echolocate-x86_64.dmg"
echo "open Echolocate-x86_64.dmg"
echo '```'
echo ""

echo "### Linux"
echo '```bash'
echo "chmod +x Echolocate-x86_64.AppImage"
echo "./Echolocate-x86_64.AppImage"
echo '```'
echo ""

echo "### Windows"
echo '```'
echo "Download and run Echolocate-Setup.exe"
echo '```'
echo ""

echo "## Verification"
echo ""
echo "All releases are signed and checksummed. Verify with:"
echo '```bash'
echo "sha256sum -c CHECKSUMS.sha256"
echo '```'
echo ""

echo "## Known Issues"
echo ""
echo "- See [Issues](https://github.com/saagar210/Echolocate/issues) for known issues"
