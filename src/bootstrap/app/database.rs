use crate::bootstrap::config::oracle_config::OracleConfig;
use anyhow::{Context, Result};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_oci::OciConnection;

pub fn create_oracle_pool(config: &OracleConfig) -> Result<Pool<ConnectionManager<OciConnection>>> {
    let database_url = config.get_url();
    let min_idle = config.get_min_idle();
    let max_size = config.get_max_conn();

    let manager = ConnectionManager::<OciConnection>::new(database_url);

    Pool::builder()
        .min_idle(Some(min_idle))
        .max_size(max_size)
        .build(manager)
        .context("오라클 커넥션 풀을 생성 실패하였습니다.")
}
