<p align="center">
  <img src="logo/kopgen.webp" alt="Kopgen Logo" width="400"/>
</p>
<h1 align="center">🦀 Kopgen 🦀</h1>

<p align="center">
<a href="https://github.com/edenreich/kopgen/actions"><img src="https://github.com/edenreich/kopgen/actions/workflows/ci.yml/badge.svg" alt="CI Status"/></a>
<a href="https://github.com/edenreich/kopgen/releases"><img src="https://img.shields.io/github/v/release/edenreich/kopgen?color=blue&style=flat-square" alt="Version"/></a>
<a href="https://github.com/edenreich/kopgen/actions"><img src="https://github.com/edenreich/kopgen/actions/workflows/docs.yml/badge.svg" alt="Documentation Status"/></a>
<a href="https://github.com/edenreich/kopgen/blob/main/LICENSE"><img src="https://img.shields.io/github/license/edenreich/kopgen?color=blue&style=flat-square" alt="License"/></a>
</p>

Welcome to the Kopgen tool! This project provides a generator for creating a Create, Read, Update and Delete (CRUD) Kubernetes operator from an OpenAPI Specification (OAS), all written in the powerful and efficient Rust language.

If you have a running API with a valid OpenAPI Specification but lack an operator to manage its complexity, this generator is the perfect solution. Kubernetes operators also align seamlessly with the GitOps methodology, enhancing your operational efficiency.

It's a well-established fact that using YAML to define API resources not only simplifies complexity but also enhances collaboration between developers and operations teams. This approach allows teams to concentrate on what truly matters - delivering value.

- [Documentation](#documentation)
- [Quick Start Guide](#quick-start-guide)
- [Contributing](#contributing)
- [Need Help?](#need-help)
- [Motivation](#motivation)
- [Why OpenAPI Specification?](#why-openapi-specification)

## Documentation

The documentation is available here: https://edenreich.github.io/kopgen/introduction.html

For documentation Github-Pages is being used.
It's built using rust mdbook which already comes pre-installed on the DevContainer.

If you need to make adjustment to the documentation you can serve it locally, run:

```bash
task docs
```

## Quick Start Guide

Before you proceed, ensure that [Docker](https://docs.docker.com/engine/install/) is installed on your system.

Download and install the latest kopgen CLI:

```sh
curl -sSL https://raw.githubusercontent.com/edenreich/kopgen/refs/heads/main/scripts/install.sh | sh
```

Start a new project in an empty folder:

```bash
kopgen init .
```

This will generate the project for the operator including a dev container environment, therefore it is recommended you use VScode.

Open the project in VScode, a prompt to open the project inside of a Dev Container will be shown, click `open`.

A Dev Container is essentially an encapsulate environment with all the necessary tools you would need for the development of this project.

To get started, follow these steps:

1. Make sure you have Docker and VSCode installed.
2. Run: `kopgen init <directory>`.
3. Open the directory in vscode: `code <directory>`.
4. You supposed to be prompted to open DevContainer, click on "Reopen in Container".
5. Configure: `cp .env.example .env`.
6. Generate the operator including all of its dependencies, run: `task generate`.
7. Run the operator: `task run`.

## Contributing

If you'd like to contribute to this project, we highly recommend using **Visual Studio Code**. VS Code comes with a pre-configured development environment through our dev container, ensuring you have all the necessary project tooling set up seamlessly.

Additionally, please install the Git pre-commit hooks. These hooks help maintain our project's coding standards by automatically checking your commits for compliance before they're finalized.

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

## Why OpenAPI Specification?

The OpenAPI Specification (OAS), formerly known as Swagger, is a standard for defining APIs. It provides a way to describe the capabilities of your API in a machine-readable format. This has several benefits:

1. **Documentation**: With OAS, your API documentation is always up-to-date because it's generated directly from your API definition. This makes it easier for developers to understand what your API does and how to use it.

2. **Client SDK Generation**: Tools like the OpenAPI Generator can generate client SDKs in various languages from an OAS document. This means developers can start using your API in their language of choice without having to write a lot of boilerplate code.

3. **Server Stub Generation**: Similarly, you can generate server stubs from an OAS document. This can speed up the initial development of your API.

4. **Validation**: You can use your OAS document to validate API requests and responses. This helps catch errors before they become problems.

5. **Integration**: Because OAS is a standard, there are many tools that can import OAS documents and provide additional functionality, such as testing tools, API gateways, and more.

In this project, we use the OpenAPI Specification to generate the necessary types and controllers for our Kubernetes operator. This allows us to easily keep our operator in sync with the latest version of our API.
