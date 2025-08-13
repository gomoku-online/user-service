use getset::CopyGetters;
use serde::Deserialize;

#[derive(CopyGetters, Debug, Deserialize)]
pub struct GrpcConfig {
    #[getset(get_copy = "pub with_prefix")]
    concurrency_limit_per_connection: u32,
}
