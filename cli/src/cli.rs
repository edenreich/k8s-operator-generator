use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "Kubernetes Operator Codegen",
    version = "v1.4.0",
    author = "Eden Reich <eden.reich@gmail.com>",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new operator project")]
    Init {
        #[arg(required = true, help = "Path to initialize the project")]
        path: String,
    },
    #[command(about = "Hydrate the OpenAPI specification with x-kubernetes-operator-* extensions")]
    Hydrate {
        #[arg(required = true, help = "Path to the OpenAPI specification file")]
        openapi_file: String,
    },
    #[command(about = "Generate Kubernetes operator code from an OpenAPI specification")]
    Generate {
        #[arg(required = true, help = "Path to the OpenAPI specification file")]
        openapi_file: String,
        #[arg(short, long, help = "Generate all code")]
        all: bool,
        #[arg(short, long, help = "Generate the lib.rs file")]
        lib: bool,
        #[arg(short, long, help = "Generate the manifests")]
        manifests: bool,
        #[arg(short, long, help = "Generate the controllers")]
        controllers: bool,
        #[arg(short, long, help = "Generate the types")]
        types: bool,
    },
}
