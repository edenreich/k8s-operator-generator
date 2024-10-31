#!/bin/bash

set -e

git config --global --add safe.directory /workspaces/kopgen

# Sign commits
git config commit.gpgsign true
