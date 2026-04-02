# Alpe

[![CI](https://github.com/alpe-cloud/alpe/actions/workflows/ci.yml/badge.svg)](https://github.com/alpe-cloud/alpe/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/alpe-cloud/alpe/graph/badge.svg)](https://codecov.io/gh/alpe-cloud/alpe)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**Sovereign cloud platform for Europe.** Managed services on European infrastructure, under European law, in open source.

Alpe provides managed compute, database, and storage resources with built-in sovereignty guarantees. Every resource is tagged with a jurisdiction, and the platform enforces that data never leaves its designated EU member state.

## Crate Architecture

| Crate | Description |
|-------|-------------|
| `alpe-core` | Core domain types, error hierarchy, and pure business logic |
| `alpe-api` | Axum-based HTTP API server |
| `alpe-auth` | JWT authentication, password hashing, and RBAC |
| `alpe-cli` | Command-line interface (`alpe` binary) |
| `alpe-sdk` | Rust SDK client for the API |
| `alpe-test` | Shared test infrastructure (dev-dependency only) |
| `alpe-operator-db` | Kubernetes operator for managed databases |
| `alpe-operator-storage` | Kubernetes operator for managed object storage |
| `alpe-operator-app` | Kubernetes operator for managed app deployments |
| `alpe-substrate` | Infrastructure abstraction layer (Hetzner, bare-metal) |

## Prerequisites

- [Rust](https://rustup.rs/) >= 1.88.0 (pinned via `rust-toolchain.toml`)
- [Docker](https://docs.docker.com/get-docker/) (for integration tests via testcontainers)

## Getting Started

```bash
# Clone the repository
git clone https://github.com/alpe-cloud/alpe.git
cd alpe

# The toolchain is installed automatically via rust-toolchain.toml
cargo build --workspace

# Run all tests
cargo test --workspace

# Run clippy lints
cargo clippy --workspace -- -D warnings

# Check formatting
cargo fmt --check

# Generate and open documentation
cargo doc --workspace --no-deps --open
```

## Cargo Aliases

Defined in `.cargo/config.toml` for fast development loops:

| Alias | Command | Purpose |
|-------|---------|---------|
| `cargo t` | `cargo test -p alpe-core` | Fast red-green loop on core types |
| `cargo ti` | `cargo test -p alpe-api` | Integration tests |
| `cargo ta` | `cargo test --workspace` | Run everything |
| `cargo d` | `cargo doc --workspace --no-deps --open` | Local doc preview |

## Contributing

### Pre-commit Hooks

This project uses [pre-commit](https://pre-commit.com/) to run quality checks before each commit:

```bash
# Install pre-commit (one-time) — pick one:
brew install pre-commit      # macOS
pipx install pre-commit      # cross-platform

# Install the git hooks (one-time, per clone)
pre-commit install

# Run all hooks manually
pre-commit run --all-files
```

The hooks enforce: formatting (`cargo fmt`), linting (`cargo clippy`), documentation builds, and dependency policy (`cargo deny`).

### Quality Tools

| Tool | Purpose | Install |
|------|---------|---------|
| `cargo-deny` | License, advisory, and dependency policy | `cargo install cargo-deny` |
| `cargo-machete` | Detect unused dependencies | `cargo install cargo-machete` |

## Documentation

Published rustdoc is available at the [GitHub Pages site](https://alpe-cloud.github.io/alpe/).

## License

Licensed under [Apache License, Version 2.0](LICENSE).
