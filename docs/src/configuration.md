# Configuration

This project uses a [DotEnv](https://www.dotenv.org/) file for configuring the CPU architecture the operator should be built for, as well as the container registry and cluster to connect to.

Please refer to the `.env.example` file to see what environment variables are available. When you are just getting started, you can copy it to a `.env` file and keep it as-is.

Here is the list of the environment variables available:

| Variable Name        | Description                                                          |
| -------------------- | -------------------------------------------------------------------- |
| `KUBECONFIG`         | Path to the kubeconfig file.                                         |
| `RUST_LOG`           | Logging level (e.g., `info`, `debug`).                               |
| `CPU_ARCH`           | CPU architecture to build the operator for (e.g., `amd64`, `arm64`). |
| `CONTAINER_REGISTRY` | Container registry to push the operator image to.                    |
| `CLUSTER_NAME`       | Name of the Kubernetes cluster to connect to.                        |
| `INSTALL_CRDS`       | Set to `true` to automatically install CRDs.                         |

By default, the configuration points to a local environment, and a local cluster will be created using ctlptl with k3d. Please review the `Cluster.yaml` file:

```yaml
---
apiVersion: ctlptl.dev/v1alpha1
kind: Registry
name: ctlptl-registry
port: 5005
---
apiVersion: ctlptl.dev/v1alpha1
kind: Cluster
product: k3d
registry: ctlptl-registry
```

As for the operator configurations, you can configure the following using custom OpenAPI attributes:

```yaml
info:
  x-kubernetes-operator-api-group: 'example.com'
  x-kubernetes-operator-api-version: 'v1'
  x-kubernetes-operator-resource-ref: 'uuid'
  x-kubernetes-operator-example-metadata-spec-field-ref: 'name'
  x-kubernetes-operator-include-tags:
    - cats
    # - dogs
    # - horses
```

| Attribute Name                                          | Description                                                                                                   |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `x-kubernetes-operator-api-group`                       | The API group of the kubernetes custom resources definitions (CRD's).                                         |
| `x-kubernetes-operator-api-version`                     | The API version of the kubernetes CRD's.                                                                      |
| `x-kubernetes-operator-resource-ref`                    | The reference ID of the data model that should be tracked on the API.                                         |
| `x-kubernetes-operator-example-metadata-spec-field-ref` | The attribute name of the example in OpenAPI spec that should serve as the name of the generated example CRD. |
| `x-kubernetes-operator-include-tags`                    | A list of tags that should be generated from OpenAPI Spec.                                                    |
