//! Proxy implementation that bridges SeaORM to `cubrid-tokio`.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use cubrid_protocol::value::Value as CubridValue;
use sea_orm::{DbErr, ProxyDatabaseTrait, ProxyExecResult, ProxyRow, Statement};
use tokio::sync::Mutex;

use crate::convert::{cubrid_value_to_sea, sea_value_to_cubrid};
use crate::error::into_db_err;

type CubridError = cubrid_tokio::Error;
type CubridQueryResult = cubrid_tokio::QueryResult;

/// Async client behavior required by [`CubridProxy`].
#[async_trait]
pub trait CubridClient: Send + Sync {
    /// Execute a query and return all rows.
    async fn query(
        &mut self,
        sql: &str,
        params: &[CubridValue],
    ) -> Result<CubridQueryResult, CubridError>;

    /// Execute a statement and return affected row count.
    async fn execute(&mut self, sql: &str, params: &[CubridValue]) -> Result<u64, CubridError>;

    /// Commit the active transaction.
    async fn commit(&mut self) -> Result<(), CubridError>;

    /// Roll back the active transaction.
    async fn rollback(&mut self) -> Result<(), CubridError>;

    /// Ping the database server.
    async fn ping(&mut self) -> Result<String, CubridError>;
}

#[cfg(not(tarpaulin_include))]
#[async_trait]
impl CubridClient for cubrid_tokio::Client {
    async fn query(
        &mut self,
        sql: &str,
        params: &[CubridValue],
    ) -> Result<CubridQueryResult, CubridError> {
        cubrid_tokio::Client::query(self, sql, params).await
    }
    async fn execute(&mut self, sql: &str, params: &[CubridValue]) -> Result<u64, CubridError> {
        cubrid_tokio::Client::execute(self, sql, params).await
    }
    async fn commit(&mut self) -> Result<(), CubridError> {
        cubrid_tokio::Client::commit(self).await
    }
    async fn rollback(&mut self) -> Result<(), CubridError> {
        cubrid_tokio::Client::rollback(self).await
    }
    async fn ping(&mut self) -> Result<String, CubridError> {
        cubrid_tokio::Client::ping(self).await
    }
}

/// SeaORM proxy backend for CUBRID.
pub struct CubridProxy<C>
where
    C: CubridClient,
{
    client: Arc<Mutex<C>>,
}

impl<C> fmt::Debug for CubridProxy<C>
where
    C: CubridClient,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CubridProxy").finish_non_exhaustive()
    }
}

impl<C> CubridProxy<C>
where
    C: CubridClient,
{
    /// Construct a proxy backend from an initialized CUBRID client.
    pub fn new(client: C) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    fn convert_statement(statement: &Statement) -> (String, Vec<CubridValue>) {
        let params: Vec<CubridValue> = match &statement.values {
            Some(values) => values.iter().map(sea_value_to_cubrid).collect(),
            None => Vec::new(),
        };
        (statement.sql.clone(), params)
    }

    fn map_rows(result: cubrid_tokio::QueryResult) -> Vec<ProxyRow> {
        let mut proxy_rows = Vec::with_capacity(result.rows.len());
        for row in &result.rows {
            let mut values = BTreeMap::new();
            for (index, value) in row.iter().enumerate() {
                let col_name = if index < result.columns.len() {
                    result.columns[index].name.as_str()
                } else {
                    "column"
                };
                let (col_name, sea_val) = cubrid_value_to_sea(value, col_name);
                values.insert(col_name, sea_val);
            }
            proxy_rows.push(ProxyRow { values });
        }
        proxy_rows
    }
}

#[async_trait]
impl<C> ProxyDatabaseTrait for CubridProxy<C>
where
    C: CubridClient,
{
    async fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let (sql, params) = Self::convert_statement(&statement);
        let mut client = self.client.lock().await;
        let result = client.query(&sql, &params).await.map_err(into_db_err)?;
        Ok(Self::map_rows(result))
    }

    async fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let (sql, params) = Self::convert_statement(&statement);
        let mut client = self.client.lock().await;
        let rows_affected = client.execute(&sql, &params).await.map_err(into_db_err)?;
        Ok(ProxyExecResult { last_insert_id: 0, rows_affected })
    }

    async fn begin(&self) {
        let mut client = self.client.lock().await;
        let _ = client.execute("BEGIN", &[]).await;
    }

    async fn commit(&self) {
        let mut client = self.client.lock().await;
        let _ = client.commit().await;
    }

    async fn rollback(&self) {
        let mut client = self.client.lock().await;
        let _ = client.rollback().await;
    }

    fn start_rollback(&self) {}

    async fn ping(&self) -> Result<(), DbErr> {
        let mut client = self.client.lock().await;
        client.ping().await.map_err(into_db_err)?;
        Ok(())
    }
}
