#/bin/sh

set -e

HOOK_FILE=".git/hooks/pre-commit"

if [ -f "$HOOK_FILE" ]; then
  rm "$HOOK_FILE"
  echo "Pre-commit hook has been successfully uninstalled."
else
  echo "No pre-commit hook found to uninstall."
fi
