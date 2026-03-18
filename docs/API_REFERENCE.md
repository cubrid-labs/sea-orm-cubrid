# API Reference

Complete API documentation for the SeaORM CUBRID backend (`sea-orm-cubrid`).

## Architecture

```mermaid
flowchart TD
    App[Application Code] --> SeaORM[SeaORM Framework]
    SeaORM --> Proxy[CubridProxy]
    Proxy --> Client[CubridClient trait]
    Client --> Tokio[cubrid-tokio]
    Tokio --> Protocol[cubrid-protocol]
    Protocol --> CAS[CAS Protocol / TCP]
    CAS --> CUBRID[(CUBRID Database)]

    subgraph "sea-orm-cubrid"
        Proxy --> Convert[convert module]
        Proxy --> Error[error module]
        Convert --> SeaValues[SeaQuery Values]
        Convert --> CubridValues[CUBRID Values]
    end
```

## Quick Start

```rust
use sea_orm_cubrid::connect;

let db = connect("cubrid://dba@localhost:33000/demodb").await?;
```

## Connection Functions

### `connect(dsn: &str) -> Result<DatabaseConnection, DbErr>`

Connect to CUBRID using a DSN string. Creates a `cubrid_tokio::Client` internally.

```rust
use sea_orm_cubrid::connect;

let db = connect("cubrid://dba:password@localhost:33000/demodb").await?;
```

**DSN format**: `cubrid://[user[:password]]@host[:port]/database`

### `connect_with_client<C>(client: C) -> Result<DatabaseConnection, DbErr>`

Connect using an already-initialized CUBRID client. Use this when you need custom connection setup.

```rust
use sea_orm_cubrid::connect_with_client;

let client = cubrid_tokio::Client::connect("cubrid://dba@localhost:33000/demodb").await?;
let db = connect_with_client(client).await?;
```

**Type constraint**: `C: CubridClient + 'static`

### `connect_with_factory<C, F, Fut>(dsn: &str, factory: F) -> Result<DatabaseConnection, DbErr>`

Connect using a custom async factory function. The factory receives the DSN as an owned `String`.

```rust
use sea_orm_cubrid::connect_with_factory;

let db = connect_with_factory("cubrid://dba@localhost:33000/demodb", |dsn| {
    cubrid_tokio::Client::connect(&dsn)
}).await?;
```

## Traits

### `CubridClient`

Async trait defining the interface for CUBRID database operations. Implement this trait to provide custom client behavior.

```rust
#[async_trait]
pub trait CubridClient: Send + Sync {
    async fn query(&mut self, sql: &str, params: &[CubridValue])
        -> Result<CubridQueryResult, CubridError>;

    async fn execute(&mut self, sql: &str, params: &[CubridValue])
        -> Result<u64, CubridError>;

    async fn commit(&mut self) -> Result<(), CubridError>;

    async fn rollback(&mut self) -> Result<(), CubridError>;

    async fn ping(&mut self) -> Result<String, CubridError>;
}
```

| Method | Returns | Description |
|--------|---------|-------------|
| `query` | `Result<CubridQueryResult, CubridError>` | Execute a query and return all rows |
| `execute` | `Result<u64, CubridError>` | Execute a statement, return affected row count |
| `commit` | `Result<(), CubridError>` | Commit the active transaction |
| `rollback` | `Result<(), CubridError>` | Roll back the active transaction |
| `ping` | `Result<String, CubridError>` | Ping the database server |

**Built-in implementation**: `cubrid_tokio::Client` implements `CubridClient`.

## Types

### `CubridProxy<C>`

SeaORM proxy backend that bridges SeaORM to CUBRID. Implements `ProxyDatabaseTrait`.

```rust
pub struct CubridProxy<C: CubridClient> { /* ... */ }

impl<C: CubridClient> CubridProxy<C> {
    pub fn new(client: C) -> Self;
}
```

### `CubridProxyError`

Error type wrapping CUBRID errors for SeaORM compatibility.

## Conversion Functions

### `sea_value_to_cubrid(value: &SeaValue) -> CubridValue`

Convert SeaQuery values into CUBRID protocol values.

| SeaQuery Type | CUBRID Type | Notes |
|---------------|-------------|-------|
| `Bool(Some(v))` | `Bool(v)` | |
| `TinyInt(Some(v))` | `Short(v as i16)` | Widened to i16 |
| `SmallInt(Some(v))` | `Short(v)` | |
| `Int(Some(v))` | `Int(v)` | |
| `BigInt(Some(v))` | `Long(v)` | |
| `TinyUnsigned(Some(v))` | `Int(v as i32)` | Widened |
| `SmallUnsigned(Some(v))` | `Int(v as i32)` | Widened |
| `Unsigned(Some(v))` | `Int(v)` or `String(v)` | String fallback for overflow |
| `BigUnsigned(Some(v))` | `Long(v)` or `String(v)` | String fallback for overflow |
| `Float(Some(v))` | `Float(v)` | |
| `Double(Some(v))` | `Double(v)` | |
| `String(Some(v))` | `String(v)` | |
| `Char(Some(v))` | `String(v)` | |
| `Bytes(Some(v))` | `Bytes(v)` | |
| Any `None` variant | `Null` | |
| Other | `String(v.to_string())` | Fallback |

### `cubrid_value_to_sea(value: &CubridValue, col_name: &str) -> (String, SeaValue)`

Convert CUBRID protocol values into SeaQuery values with the column name preserved.

| CUBRID Type | SeaQuery Type | Notes |
|-------------|---------------|-------|
| `Null` | `String(None)` | |
| `Bool(v)` | `Bool(Some(v))` | |
| `Short(v)` | `SmallInt(Some(v))` | |
| `Int(v)` | `Int(Some(v))` | |
| `Long(v)` | `BigInt(Some(v))` | |
| `Float(v)` | `Float(Some(v))` | |
| `Double(v)` | `Double(Some(v))` | |
| `String(v)` | `String(Some(v))` | |
| `Bytes(v)` | `Bytes(Some(v))` | |
| `Date{y,m,d}` | `String("YYYY-MM-DD")` | Formatted as string |
| `Time{h,m,s}` | `String("HH:MM:SS")` | Formatted as string |
| `Timestamp{...}` | `String("YYYY-MM-DD HH:MM:SS")` | |
| `Datetime{...ms}` | `String("YYYY-MM-DD HH:MM:SS.mmm")` | Millisecond precision |

## Error Handling

### `into_db_err(err: impl Into<CubridProxyError>) -> DbErr`

Convert CUBRID errors into SeaORM `DbErr` for seamless error propagation.

```rust
let result = client.query(&sql, &params).await.map_err(into_db_err)?;
```

## Backend Selection

`sea-orm-cubrid` uses `DbBackend::MySql` as the proxy backend because CUBRID's SQL syntax is closest to MySQL. SeaORM generates MySQL-compatible SQL which CUBRID accepts.

## Limitations

- No `RETURNING` clause — `last_insert_id` always returns `0` in `ProxyExecResult`
- No native JSON type support
- No native boolean — mapped through `Bool` value conversion
- Date/time values are returned as formatted strings, not native types
- Transaction `begin` uses raw `"BEGIN"` SQL (CUBRID interprets this correctly)
- DDL is auto-committed by CUBRID (not transactional)
