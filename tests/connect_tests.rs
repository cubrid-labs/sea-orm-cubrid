use async_trait::async_trait;
use cubrid_protocol::value::Value as CubridValue;
use sea_orm_cubrid::{connect, connect_with_factory, CubridClient};

#[derive(Debug, Default)]
struct ConnectMockClient;

#[async_trait]
impl CubridClient for ConnectMockClient {
    async fn query(
        &mut self,
        _sql: &str,
        _params: &[CubridValue],
    ) -> Result<cubrid_tokio::QueryResult, cubrid_tokio::Error> {
        Ok(cubrid_tokio::QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            total_count: 0,
        })
    }

    async fn execute(
        &mut self,
        _sql: &str,
        _params: &[CubridValue],
    ) -> Result<u64, cubrid_tokio::Error> {
        Ok(0)
    }

    async fn commit(&mut self) -> Result<(), cubrid_tokio::Error> {
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), cubrid_tokio::Error> {
        Ok(())
    }

    async fn ping(&mut self) -> Result<String, cubrid_tokio::Error> {
        Ok("ok".to_owned())
    }
}

#[tokio::test]
async fn connect_with_factory_builds_database_connection() {
    let db = connect_with_factory::<ConnectMockClient, _, _>("ignored", |_dsn| async {
        Ok(ConnectMockClient)
    })
    .await
    .expect("connection should succeed");

    db.ping().await.expect("db ping should succeed");
}

#[tokio::test]
async fn connect_with_factory_maps_driver_error() {
    let err = connect_with_factory::<ConnectMockClient, _, _>("ignored", |_dsn| async {
        Err(cubrid_tokio::Error::ConnectionClosed)
    })
    .await
    .expect_err("connection should fail");
    assert!(err.to_string().contains("Connection Error"));
}

#[tokio::test]
async fn connect_invalid_dsn_returns_db_err() {
    let err = connect("not-a-valid-dsn")
        .await
        .expect_err("invalid dsn should fail");
    assert!(err.to_string().contains("invalid DSN"));
}
