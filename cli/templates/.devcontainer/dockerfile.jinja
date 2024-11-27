FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

ENV TASK_VERSION=v3.39.2 \
    CTLPTL_VERSION=0.8.34 \
    K3D_VERSION=v5.7.4 \
    KUBECTL_VERSION=v1.31.0 \
    MDBOOK_VERSION=v0.4.40 \
    MDBOOK_MERMAID_VERSION=v0.14.0 \
    CARGO_AUDIT_VERSION=0.21.0 \
    ZSH_CUSTOM=/home/vscode/.oh-my-zsh/custom

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev \
    musl-tools && \
    rustup target add \
        aarch64-unknown-linux-musl \
        x86_64-unknown-linux-musl && \
    cd /tmp && \
    wget https://musl.cc/aarch64-linux-musl-cross.tgz && \
    tar -xzf aarch64-linux-musl-cross.tgz && \
    mv aarch64-linux-musl-cross /opt/ && \
    rm -rf aarch64-linux-musl-cross.tgz && \
    wget https://musl.cc/x86_64-linux-musl-cross.tgz && \
    tar -xzf x86_64-linux-musl-cross.tgz && \
    mv x86_64-linux-musl-cross /opt/ && \
    rm -rf x86_64-linux-musl-cross.tgz

RUN apt-get update && \
    # Determine architecture
    architecture=$(uname -m); \
    architecture_pretty="arm64"; \
    if [ "$architecture" = "x86_64" ]; then \
        architecture_pretty="amd64"; \
    fi && \
    echo "Installing depenenices for ${architecture}" && \
    # Install Node.js and npm \
    curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - && \
    apt-get install -y nodejs && \
    # Install prettier
    npm install -g prettier && \
    # Install semantic-release and plugins
    npm install -g semantic-release @semantic-release/changelog @semantic-release/exec @semantic-release/git @semantic-release/github conventional-changelog-conventionalcommits && \
    # Install powerlevel10k theme
    git clone --depth=1 https://github.com/romkatv/powerlevel10k.git ${ZSH_CUSTOM}/themes/powerlevel10k && \
    # Install Task
    curl -s https://taskfile.dev/install.sh | sh -s -- -b /usr/local/bin ${TASK_VERSION} && \
    # Install ctlptl
    curl -fsSL https://github.com/tilt-dev/ctlptl/releases/download/v${CTLPTL_VERSION}/ctlptl.${CTLPTL_VERSION}.linux.${architecture_pretty}.tar.gz | tar -xzv -C /usr/local/bin ctlptl && \
    # Install k3d
    curl -sSL https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | TAG=${K3D_VERSION} bash && \
    # Install kubectl
    curl -sSL https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/${architecture_pretty}/kubectl -o /usr/local/bin/kubectl && \
    chmod +x /usr/local/bin/kubectl && \
    # Install mdBook
    curl -sSL https://github.com/rust-lang/mdBook/releases/download/${MDBOOK_VERSION}/mdbook-${MDBOOK_VERSION}-${architecture}-unknown-linux-musl.tar.gz -o mdbook.tar.gz && \
    tar -xzf mdbook.tar.gz && \
    chmod +x mdbook && \
    mv mdbook /usr/local/bin/mdbook && \
    rm mdbook.tar.gz && \
    # Install mdbook-mermaid
    curl -sSL https://github.com/badboy/mdbook-mermaid/releases/download/${MDBOOK_MERMAID_VERSION}/mdbook-mermaid-${MDBOOK_MERMAID_VERSION}-x86_64-unknown-linux-musl.tar.gz -o mdbook-mermaid.tar.gz && \
    tar -xzf mdbook-mermaid.tar.gz && \
    chmod +x mdbook-mermaid && \
    mv mdbook-mermaid /usr/local/bin/mdbook-mermaid && \
    rm mdbook-mermaid.tar.gz && \
    # Clean up
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

USER vscode

RUN rustc --version && \
    # Install cargo-audit
    cargo install cargo-audit --version ${CARGO_AUDIT_VERSION} --locked

# Install kopgen
RUN curl -sSL https://raw.githubusercontent.com/edenreich/kopgen/refs/heads/main/scripts/install.sh | sh
