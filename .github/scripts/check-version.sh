#!/usr/bin/env bash

set -e

echo "Checking version requirements for release..."

VERSION=$(jq -r '.version' package.json)

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "❌ Invalid version format: $VERSION"
    echo "Version must follow semantic versioning (e.g., 1.2.3)"
    exit 1
fi

echo "✓ Version format is valid: $VERSION"

if [[ -n "$GITHUB_BASE_REF" && "$GITHUB_BASE_REF" == "master" || "$GITHUB_BASE_REF" == "main" ]]; then
    echo "Checking version bump for master/main branch..."
    
    git fetch origin master:master 2>/dev/null || git fetch origin main:main 2>/dev/null
    
    BASE_BRANCH="${GITHUB_BASE_REF}"
    git show origin/$BASE_BRANCH:package.json > /tmp/base_package.json 2>/dev/null || {
        echo "⚠️  Could not fetch base branch version, skipping version bump check"
        exit 0
    }
    
    BASE_VERSION=$(jq -r '.version' /tmp/base_package.json)
    
    if [[ "$VERSION" == "$BASE_VERSION" ]]; then
        echo "❌ Version must be bumped when merging to $BASE_BRANCH"
        echo "Current: $BASE_VERSION"
        echo "New:     $VERSION"
        exit 1
    fi
    
    echo "✓ Version bumped from $BASE_VERSION to $VERSION"
fi

echo "✓ All version checks passed"
