## Summary

[![CI](https://github.com/edenreich/k8s-operator-template/actions/workflows/ci.yml/badge.svg)](https://github.com/edenreich/k8s-operator-template/actions/workflows/ci.yml)

A template for creating a CRUD Kubernetes operator from OpenApi Specification(OAS) written in Rust.

### Prerequisites

The following dependencies are needed:

- [docker](https://docs.docker.com/engine/install/)
- [k3d](https://k3d.io/v5.6.0/#releases)
- [ctlptl](https://formulae.brew.sh/formula/ctlptl)
- [task](https://taskfile.dev/installation/)

### Quick Start

1. Download OAS: `task oas-download`.
2. Run `task generate`.
3. Import the generated types and controllers from `src/types/*` and `src/controllers/*` to `src/main.rs`.
4. Create a development cluster: `task cluster-create`.
5. Deploy CRD's `task deploy-crds`.
6. Run the operator: `task run`.
7. Try out to deploy the `manifests/samples` by running `kubectl apply -f manifests/samples/`.
8. Mark the controller as implemented in `.openapi-generator-ignore` file.

### Usage

```sh
task --list-all
```
