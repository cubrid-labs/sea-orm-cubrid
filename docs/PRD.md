# Product Requirements Document

## Product Overview

`sea-orm-cubrid` is a backend bridge crate that allows SeaORM applications to run on CUBRID without forking SeaORM.

The crate implements SeaORM's `ProxyDatabaseTrait` and delegates database I/O to `cubrid-tokio`, while preserving SeaORM's normal entity/query API.

## Goals

1. Provide a production-ready SeaORM proxy backend for CUBRID.
2. Preserve standard SeaORM developer ergonomics (`Entity::find`, `insert`, transactions).
3. Keep implementation small, explicit, and easy to audit.
4. Maintain high confidence with offline tests and high line coverage.

## Non-Goals

1. Forking or patching SeaORM internals.
2. Implementing CUBRID-specific SQL extensions in this crate.
3. Replacing SeaQuery SQL generation with a custom query builder.
4. Adding live DB integration tests to the default test suite.

## Architecture

Core pattern: `ProxyDatabaseTrait`

```text
SeaORM API -> ProxyDatabaseTrait -> CubridProxy -> cubrid-tokio -> CUBRID
```

Design decisions:

- SeaORM is connected through `Database::connect_proxy`.
- SQL is generated with SeaORM's MySQL backend (`DbBackend::MySql`) for compatibility.
- Value conversion layer maps SeaQuery values to CUBRID protocol values and back.
- Proxy uses `Arc<tokio::sync::Mutex<_>>` for async-safe shared client access.

## Example-First: End-to-End Usage

```rust,no_run
use sea_orm::{DbErr, EntityTrait};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = sea_orm_cubrid::connect("cubrid://dba:@localhost:33000/demodb").await?;

    // Any normal SeaORM operation now goes through CubridProxy.
    // let users = user::Entity::find().all(&db).await?;

    db.ping().await?;
    Ok(())
}
```

## Type Mapping

| SeaQuery `Value` | CUBRID `Value` | Reverse mapping |
|---|---|---|
| `Bool(Some(v))` | `Bool(v)` | `Bool(Some(v))` |
| `TinyInt/SmallInt` | `Short` | `SmallInt` |
| `Int` | `Int` | `Int` |
| `BigInt` | `Long` | `BigInt` |
| `Float` | `Float` | `Float` |
| `Double` | `Double` | `Double` |
| `String`/`Char` | `String` | `String` |
| `Bytes` | `Bytes` | `Bytes` |
| `Null` variants | `Null` | nullable `String(None)` fallback |
| Date/Time/Timestamp | text form | `String` text |

## CUBRID Limitations and Behavior

- `RETURNING` is not supported.
- JSON type is not natively supported.
- BOOLEAN semantics are represented using numeric storage behavior.
- No dialect-specific SQL feature extensions are added in this crate.

## Release Readiness Criteria

1. `cargo fmt --check` passes.
2. `cargo clippy --all-targets --all-features -- -D warnings` passes.
3. `cargo test` passes.
4. Coverage report is >= 95% for crate source.
5. README, AGENTS, and CI workflow are present and up to date.
