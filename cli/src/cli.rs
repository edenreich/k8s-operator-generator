use clap::{Parser, Subcommand};

/// Command-line interface for the Kubernetes Operator Generator tool.
///
/// This struct defines the CLI structure and available commands for the tool.
#[derive(Parser)]
#[command(
    name = "Kubernetes Operator Generator (kopgen)",
    version = "v1.12.1",
    author = "Eden Reich <eden.reich@gmail.com>",
    arg_required_else_help = true
)]
pub struct Cli {
    /// The command to execute.
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// The name of the kubernetes operator.
    /// Example: Cats Operator
    /// Default: Example Operator
    #[arg(
        long,
        env = "KUBERNETES_OPERATOR_NAME",
        help = "The name of the kubernetes operator",
        default_value = "Example Operator"
    )]
    pub kubernetes_operator_name: String,

    /// The author of the kubernetes operator.
    /// Example: John Doe
    /// Default: Unknown
    #[arg(
        long,
        env = "KUBERNETES_OPERATOR_AUTHOR",
        help = "The author of the kubernetes operator",
        default_value = "Unknown"
    )]
    pub kubernetes_operator_author: String,

    /// The API group of the kubernetes operator.
    /// Example: cats.example.com
    /// Default: example.com
    #[arg(
        long,
        env = "KUBERNETES_OPERATOR_API_GROUP",
        default_value = "example.com",
        help = "The API group of the kubernetes operator"
    )]
    pub kubernetes_operator_api_group: String,

    /// The API version of the operator.
    /// Example: v1alpha1
    /// Default: v1
    #[arg(
        long,
        env = "KUBERNETES_OPERATOR_API_VERSION",
        default_value = "v1",
        help = "The API version of the kubernetes operator"
    )]
    pub kubernetes_operator_api_version: String,

    /// The resource reference of the operator.
    /// Example: uuid
    /// Default: uuid
    #[arg(
        long,
        env = "KUBERNETES_OPERATOR_RESOURCE_REF",
        default_value = "uuid",
        help = "The resource reference the operator will use to track resources"
    )]
    pub kubernetes_operator_resource_ref: String,

    /// The example metadata spec field reference.
    /// Example: name
    /// Default: name
    /// This is the field in the example of OAS that will be used as the resource metadata name.
    #[arg(
        long,
        env = "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF",
        default_value = "name",
        help = "The example metadata spec field reference"
    )]
    pub kubernetes_operator_example_metadata_spec_field_ref: String,

    /// The tags to include in the operator.
    /// Example: cats, dogs, birds
    /// Default: None
    /// This is a comma-separated list of tags to include in the operator code generation.
    #[arg(
        long,
        env = "KUBERNETES_OPERATOR_INCLUDE_TAGS",
        help = "The tags to include in the operator",
        value_delimiter = ','
    )]
    pub kubernetes_operator_include_tags: Vec<String>,

    /// The name of the secret to use for the operator.
    /// Example: operator-secret
    /// Default: operator-secret
    /// This is the name of the secret that the operator will use to store the access token.
    /// The secret must be created in the same namespace as the operator.
    /// The secret must contain a key named access_token: <Value>
    #[arg(
        long,
        env = "KUBERNETES_OPERATOR_SECRET_NAME",
        default_value = "operator-secret",
        help = "The name of the secret to use for the operator"
    )]
    pub secret_name: String,
}

/// Available commands for the Kubernetes Operator Generator tool.
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new operator project.
    ///
    /// This command sets up the necessary directory structure and generates
    /// template files for a new operator project.
    #[command(about = "Initialize a new operator project")]
    Init {
        /// Path to initialize the project.
        /// Example: /path/to/project
        #[arg(required = true, help = "Path to initialize the project")]
        path: String,
    },
    /// Hydrate the OpenAPI specification with x-kubernetes-operator-* extensions.
    ///
    /// This command updates the OpenAPI specification with additional metadata
    /// from the provided configuration.
    #[command(about = "Hydrate the OpenAPI specification with x-kubernetes-operator-* extensions")]
    Hydrate {
        /// Path to the OpenAPI specification file.
        #[arg(required = true, help = "Path to the OpenAPI specification file")]
        openapi_file: String,
    },
    /// Generate Kubernetes operator code from an OpenAPI specification.
    ///
    /// This command generates various components of a Kubernetes operator project
    /// including manifests, controllers, and types based on the OpenAPI
    /// specification provided.
    #[command(about = "Generate Kubernetes operator code from an OpenAPI specification")]
    Generate {
        /// Path to the OpenAPI specification file.
        #[arg(required = true, help = "Path to the OpenAPI specification file")]
        openapi_file: String,
        /// Path to the project directory.
        #[arg(required = true, help = "Path to the project directory")]
        path: String,
        /// Generate all code.
        #[arg(short, long, help = "Generate all code")]
        all: bool,
        /// Generate the manifests.
        #[arg(short, long, help = "Generate the manifests")]
        manifests: bool,
        /// Generate the controllers.
        #[arg(short, long, help = "Generate the controllers")]
        controllers: bool,
        /// Generate the types.
        #[arg(short, long, help = "Generate the types")]
        types: bool,
    },
}
