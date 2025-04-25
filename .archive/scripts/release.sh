#!/bin/bash
set -e

# Script to prepare a new release

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

VERSION=$1

# Validate version format
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format X.Y.Z"
    exit 1
fi

# Ensure we're on dev branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "dev" ]; then
    echo "Error: Must be on dev branch to create a release"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "Error: Working directory has uncommitted changes"
    exit 1
fi

# Update version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update CHANGELOG.md
DATE=$(date +%Y-%m-%d)
sed -i "s/## \[Unreleased\]/## [Unreleased]\n\n### Added\n\n### Changed\n\n### Fixed\n\n## [$VERSION] - $DATE/" CHANGELOG.md

# Update links at bottom of CHANGELOG.md
if grep -q "\[unreleased\]:" CHANGELOG.md; then
    sed -i "s|\[unreleased\]: .*|[unreleased]: https://github.com/jwliles/smv/compare/v$VERSION...HEAD|" CHANGELOG.md
else
    echo "" >> CHANGELOG.md
    echo "[unreleased]: https://github.com/jwliles/smv/compare/v$VERSION...HEAD" >> CHANGELOG.md
fi

if grep -q "\[$VERSION\]:" CHANGELOG.md; then
    # Already has this version link
    :
else
    # Find the previous version
    PREV_VERSION=$(grep -oP '## \[\K[0-9]+\.[0-9]+\.[0-9]+(?=\])' CHANGELOG.md | head -n 2 | tail -n 1)
    if [ -n "$PREV_VERSION" ]; then
        # Add link comparing this version to previous
        echo "[$VERSION]: https://github.com/jwliles/smv/compare/v$PREV_VERSION...v$VERSION" >> CHANGELOG.md
    else
        # First version
        echo "[$VERSION]: https://github.com/jwliles/smv/releases/tag/v$VERSION" >> CHANGELOG.md
    fi
fi

# Commit the changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore(release): prepare for version $VERSION"

# Create release branch
git checkout -b release/$VERSION

echo ""
echo "Release $VERSION prepared!"
echo ""
echo "Next steps:"
echo "1. Review and edit CHANGELOG.md as needed"
echo "2. Push the branch: git push origin release/$VERSION"
echo "3. Create a PR from release/$VERSION to main"
echo "4. After merging, tag the release: git tag v$VERSION"
echo "5. Push the tag: git push origin v$VERSION"
echo "6. GitHub Actions will build and publish the release"

exit 0