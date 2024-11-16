#!/bin/sh

set -e

install_kopgen() {
    local ARCH="$1"
    local URL="$2"
    echo "Downloading kopgen for $ARCH..."
    curl -sSL "$URL" -o kopgen
    chmod +x kopgen
    sudo mv kopgen /usr/local/bin/
    echo "kopgen installed successfully!\n"
}

DEP=curl
if ! command -v $DEP >/dev/null 2>&1; then
    echo "Error: $DEP is not installed. Please install $DEP and try again."
    exit 1
fi

VERSION=$(curl -sSL "https://api.github.com/repos/edenreich/kopgen/releases/latest" | grep -oP '"tag_name": "\K(.*)(?=")')
ARCH=$(uname -m)
case "$ARCH" in
    aarch64)
        install_kopgen "ARM64" "https://github.com/edenreich/kopgen/releases/download/$VERSION/kopgen_aarch64-unknown-linux-musl"
        ;;
    x86_64)
        install_kopgen "x86_64" "https://github.com/edenreich/kopgen/releases/download/$VERSION/kopgen_x86_64-unknown-linux-musl"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

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
