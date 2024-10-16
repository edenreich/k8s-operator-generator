ARG TARGET_ARCH=aarch64-unknown-linux-musl

FROM clux/muslrust:1.84.0-nightly AS build
WORKDIR /app

ARG TARGET_ARCH
ENV TARGET_ARCH=${TARGET_ARCH}

COPY crates/k8s-operator/ k8s-operator
COPY crates/client-sdk client-sdk

RUN rustup target add ${TARGET_ARCH} && \
    cd k8s-operator && cargo build \
    --release \
    --no-default-features \
    --target ${TARGET_ARCH}

FROM gcr.io/distroless/static:nonroot
ARG TARGET_ARCH
COPY --from=build /app/k8s-operator/target/${TARGET_ARCH}/release/k8s_operator /operator
USER nonroot:nonroot
ENTRYPOINT [ "/operator" ]