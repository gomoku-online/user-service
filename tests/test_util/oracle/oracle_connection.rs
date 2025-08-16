use anyhow::Result;
use diesel::Connection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_logger::LoggingConnection;
use diesel_oci::OciConnection;
use crate::test_util::oracle::oracle_test_container::{get_oracle_database_system_url, get_oracle_database_url};

const TEST_CONNECTION_MIN_IDLE: u32 = 2;
const TEST_CONNECTION_MAX_SIZE: u32 = 10;

pub async fn create_oracle_connection_pool() -> Pool<ConnectionManager<OciConnection>> {
    let database_url = get_oracle_database_url().await;
    let connection_manager: ConnectionManager<OciConnection> =
        ConnectionManager::<OciConnection>::new(database_url);

    Pool::builder()
        .min_idle(Some(TEST_CONNECTION_MIN_IDLE))
        .max_size(TEST_CONNECTION_MAX_SIZE)
        .build(connection_manager)
        .map_err(|e| e.to_string())
        .expect("오라클 커넥션 풀 생성에 실패했습니다.")
}

pub async fn create_oracle_system_connection() -> Result<LoggingConnection<OciConnection>> {
    let database_system_url = get_oracle_database_system_url().await;

    let connection = <LoggingConnection<OciConnection> as Connection>::establish(&database_system_url)?;

    Ok(connection)
}

pub async fn create_oracle_connection() -> Result<LoggingConnection<OciConnection>> {
    let database_url = get_oracle_database_url().await;

    let connection = <LoggingConnection<OciConnection> as Connection>::establish(&database_url)?;

    Ok(connection)
}
