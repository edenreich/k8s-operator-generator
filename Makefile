VERSION ?= latest

.PHONY: help generate-crds generate-client generate build build-container deploy clean

help:
	@echo "Available targets:"
	@echo "  generate-crds      generate the CRD's from Rust code"
	@echo "  generate-client    generate the Rust client code"
	@echo "  generate           generate both the client and the CRD's"
	@echo "  install-crds       install the CRD's in the cluster"
	@echo "  build              build the Rust binaries"
	@echo "  run                run the operator locally (useful for debugging)"
	@echo "  build-container    build the operator container image"
	@echo "  deploy             deploy the operator in the cluster"
	@echo "  clean              remove the generated files"

default: help

generate-crds:
	@rm -rf manifests/crds
	@mkdir -p manifests/crds
	@docker run --rm -v ${PWD}:/app -w /app --name generate-crds rust:1.69.0-slim-bullseye \
		/bin/bash -c \
		"apt-get update && \
		apt-get install -y pkg-config libssl-dev && \
		useradd -m -s /bin/bash -u 1000 rust && \
		cargo run --bin generate-crds > manifests/crds/all.yaml && chown -R 1000:1000 manifests"

generate-client:
	@rm -rf client
	@docker run --rm --user 1000 -v ${PWD}:/app -w /app --name generate-cli openapitools/openapi-generator-cli generate \
		-i openapi.yaml \
		-g rust \
		-o client \
		--additional-properties packageName=cats_api_client

generate: generate-crds generate-client

install-crds: generate-crds
	@kubectl apply -f manifests/crds/

build:
	@cargo build --bins --release

run: build
	@cargo run --bin operator

build-container:
	@docker build -t eu.gcr.io/repository/cats:$(VERSION) .

deploy: generate-crds install-crds build-container
	@kubectl apply -f manifests/operator/

clean:
	@cargo clean
