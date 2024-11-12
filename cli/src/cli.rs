use clap::{Parser, Subcommand};

/// Command-line interface for the Kubernetes Operator Codegen tool.
///
/// This struct defines the CLI structure and available commands for the tool.
#[derive(Parser)]
#[command(
    name = "Kubernetes Operator Codegen",
    version = "v1.5.0",
    author = "Eden Reich <eden.reich@gmail.com>",
    arg_required_else_help = true
)]
pub struct Cli {
    /// The command to execute.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available commands for the Kubernetes Operator Codegen tool.
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new operator project.
    ///
    /// This command sets up the necessary directory structure and generates
    /// template files for a new operator project.
    #[command(about = "Initialize a new operator project")]
    Init {
        /// Path to initialize the project.
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
    /// including library files, manifests, controllers, and types based on the
    /// OpenAPI specification provided.
    #[command(about = "Generate Kubernetes operator code from an OpenAPI specification")]
    Generate {
        /// Path to the OpenAPI specification file.
        #[arg(required = true, help = "Path to the OpenAPI specification file")]
        openapi_file: String,
        /// Generate all code.
        #[arg(short, long, help = "Generate all code")]
        all: bool,
        /// Generate the lib.rs file.
        #[arg(short, long, help = "Generate the lib.rs file")]
        lib: bool,
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
