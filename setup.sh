#!/usr/bin/env bash
set -euo pipefail

# claude-monitor setup
# Detects your Apple Developer Team ID, prompts for a unique bundle prefix,
# patches project.yml, and regenerates the Xcode project.

BLUE=$(tput setaf 4 2>/dev/null || true)
GREEN=$(tput setaf 2 2>/dev/null || true)
YELLOW=$(tput setaf 3 2>/dev/null || true)
RED=$(tput setaf 1 2>/dev/null || true)
RESET=$(tput sgr0 2>/dev/null || true)

info() { echo "${BLUE}==>${RESET} $*"; }
ok()   { echo "${GREEN}OK${RESET}  $*"; }
warn() { echo "${YELLOW}!${RESET}   $*"; }
die()  { echo "${RED}FAIL${RESET} $*" >&2; exit 1; }

# Must run from repo root
[ -f project.yml ] || die "Run this from the repo root (project.yml not found here)."

# Preflight
command -v xcodebuild >/dev/null || die "Xcode not installed. Get it from the Mac App Store."
command -v xcodegen >/dev/null   || die "XcodeGen not installed. Run: brew install xcodegen"

# Step 1: find the Team ID
info "Looking for an Apple Development signing certificate..."
IDENTITIES=$(security find-identity -v -p codesigning 2>/dev/null || true)
TEAM_IDS=$(printf '%s\n' "$IDENTITIES" | grep -oE '\([A-Z0-9]{10}\)' | tr -d '()' | sort -u || true)
COUNT=$(printf '%s\n' "$TEAM_IDS" | grep -c . || true)

if [ "$COUNT" -eq 0 ]; then
    warn "No signing certificate found. Do this once, then re-run ./setup.sh:"
    echo ""
    echo "  1. Add your Apple ID:"
    echo "     Xcode → Settings → Accounts → + → Apple ID"
    echo ""
    echo "  2. Generate a stub Xcode project so you have something to sign:"
    echo "       xcodegen generate"
    echo "       open ClaudeWidget.xcodeproj"
    echo ""
    echo "  3. In Xcode, select the 'ClaudeWidgetApp' target →"
    echo "     'Signing & Capabilities' tab → Team dropdown →"
    echo "     pick '(Your Name) — Personal Team'."
    echo "     Xcode issues the cert immediately."
    echo ""
    echo "  4. Close Xcode and re-run: ./setup.sh"
    exit 1
fi

if [ "$COUNT" -eq 1 ]; then
    TEAM_ID=$TEAM_IDS
    ok "Found Team ID: $TEAM_ID"
else
    warn "Multiple signing teams found. Pick one:"
    i=1
    while read -r line; do
        echo "  $i) $line"
        i=$((i+1))
    done <<< "$TEAM_IDS"
    echo ""
    read -rp "Number: " CHOICE
    TEAM_ID=$(printf '%s\n' "$TEAM_IDS" | sed -n "${CHOICE}p")
    [ -n "$TEAM_ID" ] || die "Invalid selection."
    ok "Using: $TEAM_ID"
fi

# Step 2: pick a bundle prefix
DEFAULT_PREFIX="dev.${USER}.claudemonitor"
echo ""
echo "Each Mac app needs a globally unique bundle ID. For free Personal Teams,"
echo "the prefix must not collide with anyone else's app — avoid 'com.example'."
echo ""
read -rp "Bundle ID prefix [$DEFAULT_PREFIX]: " BUNDLE_PREFIX
BUNDLE_PREFIX="${BUNDLE_PREFIX:-$DEFAULT_PREFIX}"

if ! [[ "$BUNDLE_PREFIX" =~ ^[a-z][a-z0-9.-]*$ ]]; then
    die "Invalid prefix '$BUNDLE_PREFIX'. Use lowercase letters, digits, dots, hyphens only."
fi

# Step 3: patch project.yml
info "Patching project.yml..."

CURRENT_PREFIX=$(grep -E '^  bundleIdPrefix:' project.yml | awk '{print $2}' || true)
[ -n "$CURRENT_PREFIX" ] || die "Could not find bundleIdPrefix in project.yml."

# macOS sed: -i '' for in-place
sed -i '' \
    -e "s|${CURRENT_PREFIX}|${BUNDLE_PREFIX}|g" \
    -e "s|DEVELOPMENT_TEAM: \"[^\"]*\"|DEVELOPMENT_TEAM: \"${TEAM_ID}\"|" \
    project.yml

ok "project.yml updated:"
echo "    bundleIdPrefix:    $BUNDLE_PREFIX"
echo "    DEVELOPMENT_TEAM:  $TEAM_ID"

# Step 4: regenerate Xcode project
info "Running xcodegen generate..."
xcodegen generate >/dev/null
ok "Xcode project regenerated."

echo ""
echo "${GREEN}Setup complete.${RESET} Build and launch with:"
echo ""
echo "  xcodebuild -project ClaudeWidget.xcodeproj -scheme ClaudeWidgetApp \\"
echo "             -configuration Debug -destination 'platform=macOS' build"
echo ""
echo "  open ~/Library/Developer/Xcode/DerivedData/ClaudeWidget-*/Build/Products/Debug/ClaudeWidgetApp.app"
echo ""
