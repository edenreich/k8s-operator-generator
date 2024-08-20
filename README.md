# 🦀 Rusty Kubernetes Operator Generator 🦀

[![CI](https://github.com/edenreich/k8s-operator-generator/actions/workflows/ci.yml/badge.svg)](https://github.com/edenreich/k8s-operator-generator/actions/workflows/ci.yml)

Welcome to the Rusty Kubernetes Operator Generator! This project provides a generator for creating a CRUD Kubernetes operator from an OpenAPI Specification (OAS), all written in the powerful and efficient Rust language.

You have a running API with a valid OpenAPI Specification, but no operator to operate the complexty of the API ? this is where this generator comes in handy.

It's a well-established fact that using YAML to define API resources not only simplifies complexity but also enhances collaboration between developers and operations teams. This approach allows teams to concentrate on what truly matters - delivering value.

So let's get 🦀 Rusty 🦀.

- [🦀 Rusty Kubernetes Operator Generator 🦀](#-rusty-kubernetes-operator-generator-)
  - [Getting Started](#getting-started)
    - [Option 1: Manual Installation](#option-1-manual-installation)
    - [Option 2: Use the DevContainer Environment](#option-2-use-the-devcontainer-environment)
  - [Quick Start Guide](#quick-start-guide)
  - [Need Help?](#need-help)
  - [Motivation](#motivation)
  - [Why OpenAPI Specification?](#why-openapi-specification)

## Getting Started

Before you proceed, ensure that [Docker](https://docs.docker.com/engine/install/) is installed on your system. Docker is required for both manual setup and when using the DevContainer environment.

To set up the necessary tools, you have two options:

### Option 1: Manual Installation

Make sure the following tools are installed:

- [k3d](https://k3d.io/v5.6.0/#releases) - Allows you to run Kubernetes clusters inside Docker containers.
- [ctlptl](https://formulae.brew.sh/formula/ctlptl) - A convenient tool to manage your local Kubernetes environment.
- [Task](https://taskfile.dev/installation/) - A simple and efficient task runner, serving as a streamlined alternative to Make.

### Option 2: Use the DevContainer Environment

Alternatively, you can leverage the provided DevContainer environment, which comes pre-configured with all the necessary dependencies, including the tools mentioned above and several useful VSCode extensions to enhance your development experience.

## Quick Start Guide

Ready to get your hands dirty with some Rust and Kubernetes? Follow these steps:

1. **Download the OAS**: Run `task oas-download` to get the latest OpenAPI Specification.
2. **Generate the code**: Use `task generate` to create the necessary types and controllers.
3. **Create a development cluster**: Use `task cluster-create` to set up your local Kubernetes cluster.
4. **Deploy the CRDs (or set INSTALL_CRDS environment variable to true)**: Run `task deploy-crds` to deploy the Custom Resource Definitions to your cluster.
5. **Run the operator**: Start your operator with `task run`.
6. **Test it out**: Apply the example manifests with `kubectl apply -f manifests/examples/` and see your operator in action!

## Need Help?

If you're not sure what to do, just run `task --list-all` to see a list of all available tasks.

## Motivation

Why create a Kubernetes operator in Rust? Great question! Here are a few reasons:

1. **Performance**: Rust is known for its blazing fast performance. It's a system programming language that runs without any extra runtime or garbage collector. This makes Rust a great choice for writing Kubernetes operators where performance is key.

2. **Memory Safety**: Rust's unique ownership model guarantees memory safety without needing a garbage collector. This means you can write high-performance operators without worrying about manual memory management.

3. **Concurrency**: Rust has first-class support for concurrent programming. This is crucial for writing operators, which often need to manage multiple resources concurrently.

4. **Interoperability**: Rust can easily interoperate with C and other languages. This makes it easy to leverage existing libraries when writing your operator.

5. **Tooling**: Rust's tooling is top-notch. With Cargo, Rust's package manager and build system, managing dependencies and building your project is a breeze.

6. **Strong Community**: The Rust community is known for being friendly and helpful, which is always a plus when learning a new language or starting a new project.

So, are you ready to get Rusty? 🦀

## Why OpenAPI Specification?

The OpenAPI Specification (OAS), formerly known as Swagger, is a standard for defining APIs. It provides a way to describe the capabilities of your API in a machine-readable format. This has several benefits:

1. **Documentation**: With OAS, your API documentation is always up-to-date because it's generated directly from your API definition. This makes it easier for developers to understand what your API does and how to use it.

2. **Client SDK Generation**: Tools like the OpenAPI Generator can generate client SDKs in various languages from an OAS document. This means developers can start using your API in their language of choice without having to write a lot of boilerplate code.

3. **Server Stub Generation**: Similarly, you can generate server stubs from an OAS document. This can speed up the initial development of your API.

4. **Validation**: You can use your OAS document to validate API requests and responses. This helps catch errors before they become problems.

5. **Integration**: Because OAS is a standard, there are many tools that can import OAS documents and provide additional functionality, such as testing tools, API gateways, and more.

In this project, we use the OpenAPI Specification to generate the necessary types and controllers for our Kubernetes operator. This allows us to easily keep our operator in sync with the latest version of our API.
