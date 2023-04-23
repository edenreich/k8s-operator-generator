## Summary

A template for creating CRUD Kubernetes operators written in Rust.

### Usage

```sh
make help
```

```
Available targets:
  generate-crds      generate the CRD's from Rust code
  generate-client    generate the Rust client code
  generate           generate both the client and the CRD's
  install-crds       install the CRD's in the cluster
  build              build the Rust binaries
  run                run the operator locally (useful for debugging)
  build-container    build the operator container image
  deploy             deploy the operator in the cluster
  clean              remove the generated files
```