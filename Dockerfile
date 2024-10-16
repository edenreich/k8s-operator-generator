FROM clux/muslrust:1.79.0-nightly AS build
WORKDIR /app

COPY crates/k8s-operator/ k8s-operator
COPY crates/client-sdk client-sdk

RUN rustup target add aarch64-unknown-linux-musl && \
    cd k8s-operator && cargo build \
    --release \
    --no-default-features \
    --target aarch64-unknown-linux-musl

FROM gcr.io/distroless/static:nonroot
COPY --from=build /app/k8s-operator/target/aarch64-unknown-linux-musl/release/k8s_operator /operator
USER nonroot:nonroot
ENTRYPOINT [ "/operator" ]
