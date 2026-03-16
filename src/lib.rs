#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

//! SeaORM proxy backend for the CUBRID database.

use std::future::Future;
use std::sync::Arc;

use sea_orm::{Database, DatabaseConnection, DbBackend, DbErr, ProxyDatabaseTrait};

pub mod convert;
pub mod error;
pub mod proxy;

pub use convert::{cubrid_value_to_sea, sea_value_to_cubrid};
pub use error::{into_db_err, CubridProxyError};
pub use proxy::{CubridClient, CubridProxy};

/// Connect a SeaORM [`DatabaseConnection`] to CUBRID using a DSN.
#[cfg(not(tarpaulin_include))]
pub async fn connect(dsn: &str) -> Result<DatabaseConnection, DbErr> {
    let client = cubrid_tokio::Client::connect(dsn)
        .await
        .map_err(into_db_err)?;
    connect_with_client(client).await
}

/// Connect using a custom async client factory.
///
/// The factory receives an owned `String` DSN and must return a `CubridClient`.
pub async fn connect_with_factory<C, F, Fut>(
    dsn: &str,
    factory: F,
) -> Result<DatabaseConnection, DbErr>
where
    C: CubridClient + 'static,
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<C, cubrid_tokio::Error>>,
{
    let client = factory(dsn.to_owned()).await.map_err(into_db_err)?;
    connect_with_client(client).await
}

/// Connect using an already-initialized CUBRID client.
pub async fn connect_with_client<C>(client: C) -> Result<DatabaseConnection, DbErr>
where
    C: CubridClient + 'static,
{
    let proxy = CubridProxy::new(client);
    let proxy: Arc<Box<dyn ProxyDatabaseTrait>> = Arc::new(Box::new(proxy));
    Database::connect_proxy(DbBackend::MySql, proxy).await
}
