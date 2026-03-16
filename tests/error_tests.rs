use sea_orm::DbErr;
use sea_orm_cubrid::into_db_err;

#[test]
fn into_db_err_maps_connection_closed() {
    let err = into_db_err(cubrid_tokio::Error::ConnectionClosed);
    assert!(matches!(err, DbErr::Conn(_)));
    assert!(err.to_string().contains("Connection Error"));
}

#[test]
fn into_db_err_preserves_message() {
    let source = cubrid_tokio::Error::InvalidDsn("bad".to_owned());
    let err = into_db_err(source);
    let rendered = err.to_string();
    assert!(rendered.contains("invalid DSN: bad"));
}
