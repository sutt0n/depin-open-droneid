use anyhow::Context;
use serde::{Deserialize, Serialize};

use std::path::Path;

use crate::app::AppConfig;

use super::db::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub db: DbConfig,
    #[serde(default)]
    pub app: AppConfig,
}

pub struct EnvOverride {
    pub db_con: String,
}

impl Config {
    pub fn from_path(
        path: Option<impl AsRef<Path>>,
        EnvOverride { db_con }: EnvOverride,
    ) -> anyhow::Result<Self> {
        let mut config: Config = if let Some(path) = path {
            let config_file = std::fs::read_to_string(path).context("Couldn't read config file")?;

            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?
        } else {
            Default::default()
        };

        config.db.pg_con = db_con;

        Ok(config)
    }
}
