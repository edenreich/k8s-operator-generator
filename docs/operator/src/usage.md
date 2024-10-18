# Usage

This tool operates on a straightforward principle: the OpenAPI Specification is the single source of truth. Any adjustments or changes should be made directly in the OpenAPI Specification, eliminating the need for any code modifications.

Assuming you have an API running somewhere, you most likely have also an OpenAPI Specification file, which defining all of the endpoints of your API (if you don't, please search on YouTube for the following `Documentation Driven Development with OpenAPI`, there are a lot of tutorials how to write a `Swagger` / `OpenAPI` files).

This tool will read the OpenAPI Specification you give it and will generate the Operator along aside with its `Kubernetes Custom Resource Definition (CRD's)` including the necessary `Role Based Access Control (RBAC)` policies, to ensure the operator is only maintaining resources from the given resource groups.

Based on the Endpoints configured in the OpenAPI Specification the tool will know what type of operation needs to be implemented in code. As an example if you have a POST endpoint, the tool will generate the necessary code for creating your data model on the API endpoint, if it's a PUT, the tool will generate the update code and so on and so forth.

If your OpenAPI Specification does not include a Delete endpoint, a warning will appear in the operator logs when you attempt to delete the Kubernetes CRD, indicating that the Delete operation is not implemented.

This tool is coming in the form of a CLI that you can run in the terminal, the CLI is located in `crates/k8s-codegen` and it's built using <a href="https://docs.rs/clap/latest/clap/" target="_blank">Clap</a>.

To see the available commands run:

```bash
cargo run --package k8s_codegen generate --help
```

You should see the following output:

```bash
Usage: k8s_codegen generate [OPTIONS]

Options:
  -a, --all
  -l, --lib
  -m, --manifests
  -c, --controllers
  -t, --types
  -h, --help         Print help
```

You can either generate the code along aside with everything, or generate only the specifics.

For simplicity, let's generate everything, run:

```bash
cargo run --package k8s_codegen generate --all
```

Note that 2 crates are completely generated after running this command, you should see `crates/k8s-crdgen` and `crates/k8s-operator`.

The crate `k8s-crdgen` is a simple rust code that translates to a binary which generating the Kubernetes CRD's out of the Rust data models.

The crate `k8s-operator` contains the actual code that uses the CRD's and executes the Create, Read, Update and Delete (CRUD) operations.

You can also use the `Taskfile` located in the root directory and instead run:

```bash
task oas-generate-rust-client
task oas-generate-docs
task generate-code
task generate-crds
```

Or run just:

```bash
task generate
```

Which will run all the above commands at once.

To list the available tasks you can run:

```bash
task --list
```

After working with the existing `openapi.yaml` file, which is a dummy one. Feel free to change it with your actual API and regenerate everything by, deleting the generated code:

```bash
rm -rf crates/{k8s-crdgen,k8s-operator,client-sdk}
```

Comment out temporarily the workspaces in the root `Cargo.toml` and replace the `openapi.yaml` with your actual API and run:

```bash
task generate
```

To troubleshoot errors with your `OpenAPI Spec`, you can also run:

```bash
task oas-validate
```

Please also provide examples in your OpenAPI Specification because these will help you generate the example CRD's for testing purposes.
