#!/bin/sh

set -e

install_kopgen() {
    local VERSION=$1
    local OS_LABEL=$2
    local ARCH_LABEL=$3
    local DOWNLOAD_URL=$4
    echo "Installing kopgen version $VERSION for $OS_LABEL $ARCH_LABEL from $DOWNLOAD_URL"
    curl -sSL "$DOWNLOAD_URL" -o /usr/local/bin/kopgen
    chmod +x /usr/local/bin/kopgen
    echo "kopgen installed successfully!\n"
}

DEP=curl
if ! command -v $DEP >/dev/null 2>&1; then
    echo "Error: $DEP is not installed. Please install $DEP and try again."
    exit 1
fi

VERSION=$(curl -sSL "https://api.github.com/repos/edenreich/kopgen/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
OS=$(uname -s)
ARCH=$(uname -m)
case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-unknown-linux-gnu"
                ARCH_LABEL="x86_64"
                ;;
            aarch64|arm64)
                TARGET="aarch64-unknown-linux-musl"
                ARCH_LABEL="ARM64"
                ;;
            *)
                echo "Unsupported architecture for Linux: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-apple-darwin"
                ARCH_LABEL="x86_64"
                ;;
            aarch64|arm64)
                TARGET="aarch64-apple-darwin"
                ARCH_LABEL="ARM64"
                ;;
            *)
                echo "Unsupported architecture for macOS: $ARCH"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Unsupported operating system: $OS"
        exit 1
        ;;
esac

DOWNLOAD_URL="https://github.com/edenreich/kopgen/releases/download/$VERSION/kopgen_$TARGET"

install_kopgen "$VERSION" "$OS" "$ARCH_LABEL" "$DOWNLOAD_URL"

cat <<- EOF
To get started, follow these steps:

1. Make sure you have Docker and VSCode installed
2. Run: kopgen init <directory>
3. Open the directory in vscode: code <directory>
4. You supposed to be prompted to open DevContainer, click on "Reopen in Container"
5. Configure: cp .env.example .env
6. Generate the operator including all of its dependencies, run: task generate
7. Run the operator: task run
EOF
