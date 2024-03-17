## Summary

[![CI](https://github.com/edenreich/k8s-operator-template/actions/workflows/ci.yml/badge.svg)](https://github.com/edenreich/k8s-operator-template/actions/workflows/ci.yml)

A template for creating a CRUD Kubernetes operator from OpenAPI Specification(OAS) written in Rust.

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
4. Mark the controller as in implementation by adding it to `.openapi-generator-ignore` file.
5. Implement the Data Transfer Object(DTO) in the controller.
6. Create a development cluster: `task cluster-create`.
7. Deploy CRD's `task deploy-crds`.
8. Run the operator: `task run`.
9. Try out to deploy the `manifests/examples` by running `kubectl apply -f manifests/examples/`.

### Usage

```sh
task --list-all
```
