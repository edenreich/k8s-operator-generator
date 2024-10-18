#!/usr/bin/zsh

sudo apt-get update && sudo apt-get install -y pkg-config libssl-dev nodejs npm

# Install favorite prompt theme
git clone --depth=1 https://github.com/romkatv/powerlevel10k.git ${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/themes/powerlevel10k

# Install task
TASK_VERSION=v3.39.2
curl -s https://taskfile.dev/install.sh | sudo sh -s -- -b /usr/local/bin $TASK_VERSION

# Install ctlptl
CTLPTL_VERSION="0.8.34"
curl -fsSL https://github.com/tilt-dev/ctlptl/releases/download/v$CTLPTL_VERSION/ctlptl.$CTLPTL_VERSION.linux.x86_64.tar.gz | sudo tar -xzv -C /usr/local/bin ctlptl


# Install k3d
K3D_VERSION="v5.7.4"
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | TAG=$K3D_VERSION bash

# Install kubectl
KUBECTL_VERSION="v1.31.0"
curl -LO https://dl.k8s.io/release/$KUBECTL_VERSION/bin/linux/amd64/kubectl
chmod +x kubectl
sudo mv kubectl /usr/local/bin/kubectl