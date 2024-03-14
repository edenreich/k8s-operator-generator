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
5. Add the generated types to `src/crdgen.rs`.
6. Generate CRD's `task generate`.
7. Create a development cluster: `task cluster-create`.
8. Deploy CRD's `task deploy-crds`.
9. Run the operator: `task run`.
10. Try out to deploy the `manifests/samples` by running `kubectl apply -f manifests/samples/`.

### Workflow

1. Download the latest Open API Specification of your API.
2. Generate the controllers by running `task build`.
3. Add the controllers to `src/controllers/mod.rs`.
4. Use the controllers to `src/main.rs`.
5. Modify the controllers, add the business logic for `CRUD`.
6. Mark the file in `.openapi-generator-ignore` so the next time you run `task build` it will not be overwritten.

### Usage

```sh
task --list-all
```
