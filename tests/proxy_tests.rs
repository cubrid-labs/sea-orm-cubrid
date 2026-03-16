use std::sync::Arc;

use async_trait::async_trait;
use cubrid_protocol::{constants::DataType, types::ColumnMetaData, value::Value as CubridValue};
use sea_orm::{DbBackend, ProxyDatabaseTrait, Statement, Values};
use sea_orm_cubrid::{CubridClient, CubridProxy};
use tokio::sync::Mutex;

#[derive(Debug, Default)]
struct MockClient {
    last_sql: Option<String>,
    last_params: Vec<CubridValue>,
    begin_count: usize,
    commit_count: usize,
    rollback_count: usize,
    ping_count: usize,
    query_rows: Vec<Vec<CubridValue>>,
    query_columns: Vec<ColumnMetaData>,
    affected_rows: u64,
    fail: bool,
}

#[async_trait]
impl CubridClient for MockClient {
    async fn query(
        &mut self,
        sql: &str,
        params: &[CubridValue],
    ) -> Result<cubrid_tokio::QueryResult, cubrid_tokio::Error> {
        self.last_sql = Some(sql.to_owned());
        self.last_params = params.to_vec();

        if self.fail {
            return Err(cubrid_tokio::Error::ConnectionClosed);
        }

        Ok(cubrid_tokio::QueryResult {
            columns: self.query_columns.clone(),
            rows: self.query_rows.clone(),
            total_count: self.query_rows.len() as i32,
        })
    }

    async fn execute(
        &mut self,
        sql: &str,
        params: &[CubridValue],
    ) -> Result<u64, cubrid_tokio::Error> {
        self.last_sql = Some(sql.to_owned());
        self.last_params = params.to_vec();

        if self.fail {
            return Err(cubrid_tokio::Error::ConnectionClosed);
        }

        if sql == "BEGIN" {
            self.begin_count += 1;
            return Ok(0);
        }

        Ok(self.affected_rows)
    }

    async fn commit(&mut self) -> Result<(), cubrid_tokio::Error> {
        self.commit_count += 1;
        if self.fail {
            return Err(cubrid_tokio::Error::ConnectionClosed);
        }
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), cubrid_tokio::Error> {
        self.rollback_count += 1;
        if self.fail {
            return Err(cubrid_tokio::Error::ConnectionClosed);
        }
        Ok(())
    }

    async fn ping(&mut self) -> Result<String, cubrid_tokio::Error> {
        self.ping_count += 1;
        if self.fail {
            return Err(cubrid_tokio::Error::ConnectionClosed);
        }
        Ok("11.4".to_owned())
    }
}

fn make_statement(sql: &str, values: Vec<sea_orm::Value>) -> Statement {
    Statement {
        sql: sql.to_owned(),
        values: Some(Values(values)),
        db_backend: DbBackend::MySql,
    }
}

fn empty_statement(sql: &str) -> Statement {
    Statement {
        sql: sql.to_owned(),
        values: None,
        db_backend: DbBackend::MySql,
    }
}

fn col(name: &str, data_type: DataType) -> ColumnMetaData {
    ColumnMetaData {
        column_type: data_type,
        scale: 0,
        precision: 0,
        name: name.to_owned(),
        real_name: name.to_owned(),
        table_name: "t".to_owned(),
        is_nullable: true,
        default_value: String::new(),
        is_auto_increment: false,
        is_unique_key: false,
        is_primary_key: false,
        is_foreign_key: false,
    }
}

#[tokio::test]
async fn query_maps_rows_and_parameters() {
    let mock = MockClient {
        query_columns: vec![col("id", DataType::Int), col("name", DataType::String)],
        query_rows: vec![vec![
            CubridValue::Int(1),
            CubridValue::String("alpha".to_owned()),
        ]],
        ..Default::default()
    };

    let proxy = CubridProxy::new(mock);
    let stmt = make_statement(
        "SELECT id, name FROM user WHERE id = ?",
        vec![sea_orm::Value::Int(Some(1))],
    );

    let rows = proxy.query(stmt).await.expect("query should pass");
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].values.get("id"),
        Some(&sea_orm::Value::Int(Some(1)))
    );
    assert_eq!(
        rows[0].values.get("name"),
        Some(&sea_orm::Value::String(Some(Box::new("alpha".to_owned()))))
    );
}

#[tokio::test]
async fn execute_maps_rows_affected() {
    let mock = MockClient {
        affected_rows: 3,
        ..Default::default()
    };

    let proxy = CubridProxy::new(mock);
    let stmt = make_statement(
        "UPDATE user SET active = ?",
        vec![sea_orm::Value::Bool(Some(true))],
    );

    let result = proxy.execute(stmt).await.expect("execute should pass");
    assert_eq!(result.rows_affected, 3);
    assert_eq!(result.last_insert_id, 0);
}

#[tokio::test]
async fn query_error_converts_to_db_err() {
    let mock = MockClient {
        fail: true,
        ..Default::default()
    };
    let proxy = CubridProxy::new(mock);

    let err = proxy
        .query(empty_statement("SELECT 1"))
        .await
        .expect_err("query should fail");
    assert!(err.to_string().contains("Connection Error"));
}

#[tokio::test]
async fn execute_error_converts_to_db_err() {
    let mock = MockClient {
        fail: true,
        ..Default::default()
    };
    let proxy = CubridProxy::new(mock);

    let err = proxy
        .execute(empty_statement("UPDATE t SET x = 1"))
        .await
        .expect_err("execute should fail");
    assert!(err.to_string().contains("Connection Error"));
}

#[tokio::test]
async fn transaction_and_ping_methods_delegate() {
    let proxy = CubridProxy::new(MockClient::default());
    proxy.begin().await;
    proxy.commit().await;
    proxy.rollback().await;
    proxy.start_rollback();
    proxy.ping().await.expect("ping should pass");
    let debug_text = format!("{proxy:?}");
    assert!(debug_text.contains("CubridProxy"));
}

#[tokio::test]
async fn query_handles_empty_results() {
    let proxy = CubridProxy::new(MockClient::default());
    let rows = proxy
        .query(empty_statement("SELECT id FROM user"))
        .await
        .expect("query should pass");
    assert!(rows.is_empty());
}

#[tokio::test]
async fn query_handles_column_mismatch() {
    let mock = MockClient {
        query_columns: vec![col("only_col", DataType::Int)],
        query_rows: vec![vec![
            CubridValue::Int(10),
            CubridValue::String("extra".to_owned()),
        ]],
        ..Default::default()
    };
    let proxy = CubridProxy::new(mock);

    let rows = proxy
        .query(empty_statement("SELECT * FROM user"))
        .await
        .expect("query should pass");

    assert_eq!(rows.len(), 1);
    assert!(rows[0].values.contains_key("only_col"));
    assert!(rows[0].values.contains_key("column"));
}

#[tokio::test]
async fn state_tracking_mock_receives_sql_and_params() {
    let mock = Arc::new(Mutex::new(MockClient::default()));
    let proxy = CubridProxy::new(SharedMockClient(mock.clone()));

    let stmt = make_statement(
        "DELETE FROM user WHERE id = ?",
        vec![sea_orm::Value::Int(Some(5))],
    );
    proxy.execute(stmt).await.expect("execute should pass");

    let guard = mock.lock().await;
    assert_eq!(
        guard.last_sql.as_deref(),
        Some("DELETE FROM user WHERE id = ?")
    );
    assert_eq!(guard.last_params, vec![CubridValue::Int(5)]);
}

#[derive(Debug)]
struct SharedMockClient(Arc<Mutex<MockClient>>);

#[async_trait]
impl CubridClient for SharedMockClient {
    async fn query(
        &mut self,
        sql: &str,
        params: &[CubridValue],
    ) -> Result<cubrid_tokio::QueryResult, cubrid_tokio::Error> {
        self.0.lock().await.query(sql, params).await
    }

    async fn execute(
        &mut self,
        sql: &str,
        params: &[CubridValue],
    ) -> Result<u64, cubrid_tokio::Error> {
        self.0.lock().await.execute(sql, params).await
    }

    async fn commit(&mut self) -> Result<(), cubrid_tokio::Error> {
        self.0.lock().await.commit().await
    }

    async fn rollback(&mut self) -> Result<(), cubrid_tokio::Error> {
        self.0.lock().await.rollback().await
    }

    async fn ping(&mut self) -> Result<String, cubrid_tokio::Error> {
        self.0.lock().await.ping().await
    }
}

#[tokio::test]
async fn begin_commit_rollback_via_shared_mock() {
    let mock = Arc::new(Mutex::new(MockClient::default()));
    let proxy = CubridProxy::new(SharedMockClient(mock.clone()));

    proxy.begin().await;
    {
        let guard = mock.lock().await;
        assert_eq!(guard.begin_count, 1);
        assert_eq!(guard.last_sql.as_deref(), Some("BEGIN"));
    }

    proxy.commit().await;
    {
        let guard = mock.lock().await;
        assert_eq!(guard.commit_count, 1);
    }

    proxy.rollback().await;
    {
        let guard = mock.lock().await;
        assert_eq!(guard.rollback_count, 1);
    }
}

#[tokio::test]
async fn ping_via_shared_mock() {
    let mock = Arc::new(Mutex::new(MockClient::default()));
    let proxy = CubridProxy::new(SharedMockClient(mock.clone()));

    proxy.ping().await.expect("ping should pass");
    {
        let guard = mock.lock().await;
        assert_eq!(guard.ping_count, 1);
    }
}

#[tokio::test]
async fn ping_error_via_proxy() {
    let mock = MockClient {
        fail: true,
        ..Default::default()
    };
    let proxy = CubridProxy::new(mock);
    let err = proxy.ping().await.expect_err("ping should fail");
    assert!(err.to_string().contains("Connection Error"));
}

#[tokio::test]
async fn execute_returns_correct_affected_rows() {
    let mock = MockClient {
        affected_rows: 42,
        ..Default::default()
    };
    let proxy = CubridProxy::new(mock);
    let stmt = make_statement(
        "INSERT INTO t VALUES (?)",
        vec![sea_orm::Value::String(Some(Box::new("val".to_owned())))],
    );
    let result = proxy.execute(stmt).await.expect("execute should pass");
    assert_eq!(result.rows_affected, 42);
    assert_eq!(result.last_insert_id, 0);
}

#[tokio::test]
async fn query_with_multiple_rows() {
    let mock = MockClient {
        query_columns: vec![
            col("id", DataType::Int),
            col("score", DataType::Float),
        ],
        query_rows: vec![
            vec![CubridValue::Int(1), CubridValue::Float(9.5)],
            vec![CubridValue::Int(2), CubridValue::Float(8.0)],
            vec![CubridValue::Int(3), CubridValue::Float(7.3)],
        ],
        ..Default::default()
    };
    let proxy = CubridProxy::new(mock);
    let rows = proxy
        .query(make_statement("SELECT id, score FROM t", vec![]))
        .await
        .expect("query should pass");
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[0].values.get("id"),
        Some(&sea_orm::Value::Int(Some(1)))
    );
    assert_eq!(
        rows[2].values.get("score"),
        Some(&sea_orm::Value::Float(Some(7.3)))
    );
}

#[tokio::test]
async fn convert_statement_with_no_values() {
    let proxy = CubridProxy::new(MockClient::default());
    let rows = proxy
        .query(empty_statement("SELECT 1"))
        .await
        .expect("query should pass");
    assert!(rows.is_empty());
}

#[tokio::test]
async fn execute_with_no_values() {
    let mock = MockClient {
        affected_rows: 1,
        ..Default::default()
    };
    let proxy = CubridProxy::new(mock);
    let result = proxy
        .execute(empty_statement("DELETE FROM t"))
        .await
        .expect("execute should pass");
    assert_eq!(result.rows_affected, 1);
}
