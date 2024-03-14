## Summary

A template for creating CRUD Kubernetes operators written in Rust.

### Prerequisites

The following dependencies are needed:

- [docker](https://docs.docker.com/engine/install/)
- [k3d](https://k3d.io/v5.6.0/#releases)
- [ctlptl](https://formulae.brew.sh/formula/ctlptl)
- [task](https://taskfile.dev/installation/)

### Quick Start

1. Download OAS: `task oas-download`.
2. Build the project: `task build`.
3. Import the generated types from `src/lib.rs` and use them in `src/main.rs`.
4. Add the generated controllers to `src/controllers/mod.rs` and use them in `src/main.rs`.
5. Generate CRD's `task generate`.
6. Create a development cluster: `task cluster-create`.
7. Deploy CRD's `task deploy-crds`.
8. Run the operator: `task run`.
9. Try out to deploy the `manifests/samples` by running `kubectl apply -f manifests/samples/`.

### Usage

```sh
task --list-all
```
