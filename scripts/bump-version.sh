#!/bin/bash

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "Bumping version to v${VERSION}"

# Bump version in Cargo.toml
sed -i "s/^version = \"[0-9]\{1,2\}\.[0-9]\{1,2\}\.[0-9]\{1,2\}\(-rc\.[1-9]\{1,1\}\)\?\"/version = \"${VERSION}\"/" cli/Cargo.toml

# Bump cli version
sed -i "s/version = \"v[0-9]\{1,2\}\.[0-9]\{1,2\}\.[0-9]\{1,2\}\(-rc\.[1-9]\{1,1\}\)\?\"/version = \"v${VERSION}\"/" cli/src/cli.rs

# Bump version in MD files
sed -i "s/v[0-9]\{1,2\}\.[0-9]\{1,2\}\.[0-9]\{1,2\}\(-rc\.[0-9]\{1,1\}\)\?/v${VERSION}/g" README.md
