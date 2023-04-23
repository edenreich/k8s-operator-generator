FROM rust:1.69.0-alpine3.17 AS build
RUN apk add --no-cache --update libressl-dev musl-dev
WORKDIR /app
COPY . .
RUN cargo build --bin operator --target x86_64-unknown-linux-musl --release --no-default-features

FROM gcr.io/distroless/static:nonroot
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/operator /operator
ENTRYPOINT [ "/operator" ]
