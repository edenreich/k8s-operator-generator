#!/bin/sh

set -e

HOOK_DIR=".git/hooks"
HOOK_FILE="pre-commit"

echo "Installing pre-commit hook..."

# Copy the pre-commit file
cp ./hooks/$HOOK_FILE $HOOK_DIR/$HOOK_FILE
chmod +x $HOOK_DIR/$HOOK_FILE

echo "Pre-commit hook installed."
