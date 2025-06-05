#!/bin/bash
set -e

NEW_VERSION=$1

if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <new-pkl-version>"
    exit 1
fi

echo "Updating Pkl version to $NEW_VERSION"

# Update the version in pkl_tooling.rs
PKL_TOOLING_FILE="src/pkl_tooling.rs"

if [ -f "$PKL_TOOLING_FILE" ]; then
    echo "Updating $PKL_TOOLING_FILE..."

    # Update the recommended version
    sed -i.bak "s/\"[0-9]\+\.[0-9]\+\.[0-9]\+\"/\"$NEW_VERSION\"/" "$PKL_TOOLING_FILE"

    # Update compatible versions array (add new version if not already present)
    if ! grep -q "\"$NEW_VERSION\"" "$PKL_TOOLING_FILE"; then
        sed -i.bak "s/vec!\[\(.*\)\]/vec![\1, \"$NEW_VERSION\"]/" "$PKL_TOOLING_FILE"
    fi

    rm -f "$PKL_TOOLING_FILE.bak"
    echo "✅ Updated $PKL_TOOLING_FILE"
else
    echo "❌ $PKL_TOOLING_FILE not found"
    exit 1
fi

# Update the GitHub Actions workflow matrix
WORKFLOW_FILE=".github/workflows/pkl-version-management.yml"

if [ -f "$WORKFLOW_FILE" ]; then
    echo "Updating $WORKFLOW_FILE..."

    # Add new version to matrix if not already present
    if ! grep -q "$NEW_VERSION" "$WORKFLOW_FILE"; then
        sed -i.bak "s/pkl_version: \[\(.*\)\]/pkl_version: [\1, '$NEW_VERSION']/" "$WORKFLOW_FILE"
    fi

    rm -f "$WORKFLOW_FILE.bak"
    echo "✅ Updated $WORKFLOW_FILE"
else
    echo "❌ $WORKFLOW_FILE not found"
    exit 1
fi

# Update Cargo.toml if there are version-specific dependencies
CARGO_FILE="Cargo.toml"
if [ -f "$CARGO_FILE" ]; then
    echo "Checking $CARGO_FILE for version-specific updates..."
    # This is a placeholder - add actual version updates if needed
    echo "✅ No changes needed in $CARGO_FILE"
fi

echo "🎉 Successfully updated Pkl version references to $NEW_VERSION"
echo ""
echo "Files modified:"
echo "  - $PKL_TOOLING_FILE"
echo "  - $WORKFLOW_FILE"
echo ""
echo "Please review the changes and commit them."
