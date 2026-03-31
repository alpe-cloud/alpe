# Alpe

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

- [Rust](https://rustup.rs/) ≥ 1.85.0 (pinned via `rust-toolchain.toml`)
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

## Documentation

Published rustdoc is available at the [GitHub Pages site](https://alpe-cloud.github.io/alpe/).

## License

Licensed under [Apache License, Version 2.0](LICENSE).
