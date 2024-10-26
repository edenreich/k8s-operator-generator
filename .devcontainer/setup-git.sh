#!/bin/bash

set -e

git config --global --add safe.directory /workspaces/k8s-operator-generator

# Sign commits
git config commit.gpgsign true
