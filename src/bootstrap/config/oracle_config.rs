use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Getters, CopyGetters, Debug, Deserialize)]
pub struct OracleConfig {
    #[getset(get = "pub with_prefix")]
    url: String,
    #[getset(get_copy = "pub with_prefix")]
    migration_enabled: bool,
    #[getset(get_copy = "pub with_prefix")]
    min_idle: u32,
    #[getset(get_copy = "pub with_prefix")]
    max_conn: u32,
}