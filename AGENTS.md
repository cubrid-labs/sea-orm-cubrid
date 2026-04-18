# AGENTS.md

Project knowledge base for AI coding agents.

## Project Overview

`sea-orm-cubrid` provides a SeaORM backend for CUBRID through SeaORM's `ProxyDatabaseTrait` extension point.

- Language: Rust 2021
- Runtime: tokio
- ORM: SeaORM 1.x (proxy mode)
- Driver: cubrid-tokio
- Protocol model: cubrid-protocol

## Architecture

The crate follows SeaORM's proxy pattern:

1. SeaORM builds SQL with `DbBackend::MySql`.
2. `CubridProxy` receives `Statement` and parameters.
3. Conversion layer maps SeaQuery values to CUBRID values.
4. `cubrid-tokio` executes SQL and returns rows.
5. Results are converted into `ProxyRow` for SeaORM.

Key characteristics:

- No SeaORM fork.
- No custom SQL builder.
- Async-safe shared client via `Arc<tokio::sync::Mutex<_>>`.

## Module Responsibilities

- `src/lib.rs`: public API, `connect()`, module exports
- `src/proxy.rs`: `CubridProxy`, `ProxyDatabaseTrait` implementation, statement handling
- `src/convert.rs`: value conversion (`sea_query::Value` <-> `cubrid_protocol::Value`)
- `src/error.rs`: error conversion (`cubrid_tokio::Error` -> `sea_orm::DbErr`)
- `tests/`: offline behavior and conversion coverage

## Code Conventions

- Rust 2021 edition
- `#![deny(unsafe_code)]`
- Formatting: rustfmt, 100 char max line width
- Lints: clippy with warnings denied in CI
- Avoid `unwrap`/`expect` in library code
- Keep public API documented

## Development Workflow (cubrid-labs org standard)

All non-trivial work across cubrid-labs repositories MUST follow this 4-phase cycle:

1. **Oracle Design Review** — Consult Oracle before implementation to validate architecture, API surface, and approach. Raise concerns early.
2. **Implementation** — Build the feature/fix with tests. Follow existing codebase patterns.
3. **Documentation Update** — Update ALL affected docs (README, CHANGELOG, ROADMAP, API docs, SUPPORT_MATRIX, PRD, etc.) in the same PR or as an immediate follow-up. Code without doc updates is incomplete.
4. **Oracle Post-Implementation Review** — Consult Oracle to review the completed work for correctness, edge cases, and consistency before merging.

Skipping any phase requires explicit justification. Trivial changes (typos, single-line fixes) may skip phases 1 and 4.

## Test Structure

- `tests/convert_tests.rs`: all important conversion paths
- `tests/proxy_tests.rs`: query/execute/transaction/ping delegation and row mapping
- `tests/error_tests.rs`: driver error to `DbErr` mapping
- `tests/connect_tests.rs`: connection helper behavior via mock factory and invalid DSN path

All tests are offline and deterministic.
