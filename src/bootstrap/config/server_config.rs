use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(CopyGetters, Getters, Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[getset(get_copy = "pub with_prefix")]
    port: u16,
    #[getset(get = "pub with_prefix")]
    host: String,
}