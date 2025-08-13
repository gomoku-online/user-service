use crate::bootstrap::config::grpc_config::GrpcConfig;
use crate::bootstrap::config::oracle_config::OracleConfig;
use config::{Config, File, FileFormat};
use getset::Getters;
use regex::Regex;
use serde::Deserialize;
use std::{env, fs};
use anyhow::Result;
use crate::bootstrap::config::server_config::ServerConfig;

#[derive(Getters, Debug, Deserialize)]
pub struct AppConfig {
    #[getset(get = "pub with_prefix")]
    server: ServerConfig,
    #[getset(get = "pub with_prefix")]
    oracle: OracleConfig,
    #[getset(get = "pub with_prefix")]
    grpc: GrpcConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "local".into());

        let base_yaml = fs::read_to_string("config/application.yml")?;
        let base_yaml = Self::substitute_env_vars(&base_yaml);

        let mode_path = format!("config/application-{}.yml", run_mode);
        let mode_yaml = fs::read_to_string(&mode_path).ok();
        let mode_yaml = mode_yaml.map(|yml| Self::substitute_env_vars(&yml));

        let mut builder =
            Config::builder().add_source(File::from_str(&base_yaml, FileFormat::Yaml));

        if let Some(mode_yaml) = mode_yaml {
            builder = builder.add_source(File::from_str(&mode_yaml, FileFormat::Yaml));
        }

        let config = builder.build()?.try_deserialize()?;
        Ok(config)
    }

    fn substitute_env_vars(content: &str) -> String {
        let re = Regex::new(r"\$\{(\w+)(:([^}]*))?\}").unwrap();
        re.replace_all(content, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let default = caps.get(3).map_or("", |m| m.as_str());

            std::env::var(var_name).unwrap_or_else(|_| default.to_string())
        })
        .to_string()
    }
}
