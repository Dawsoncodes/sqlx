use serde::{Deserialize, Serialize};
use std::fs::{self, File};

use crate::opt::ConnectOpts;

/// Main struct for the sqlx-config.json
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlxConfig {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub env_path: Option<String>,
}

impl Default for SqlxConfig {
    fn default() -> Self {
        SqlxConfig {
            schema: "https://raw.githubusercontent.com/Dawsoncodes/sqlx/main/sqlx-cli/sqlx-config.schema.json"
                .to_string(),
            env_path: Some(".env".to_string())
        }
    }
}

// SqlxConfig::create
impl SqlxConfig {
    pub fn create() -> anyhow::Result<SqlxConfig> {
        let default = SqlxConfig::default();

        serde_json::to_writer_pretty(File::create("sqlx-config.json")?, &default)?;

        Ok(default)
    }
}

impl SqlxConfig {
    pub fn read() -> anyhow::Result<SqlxConfig> {
        let file = fs::read_to_string("sqlx-config.json");
        match file {
            Ok(f) => {
                let config: SqlxConfig = serde_json::from_str(&f).unwrap();

                Ok(config)
            }
            Err(_) => {
                // Create the file
                let default = SqlxConfig::default();

                serde_json::to_writer_pretty(File::create("sqlx-config.json")?, &default)?;

                Ok(default)
            }
        }
    }

    pub fn get_database_url() -> Option<String> {
        let sqlx_config = SqlxConfig::read().unwrap_or_default();

        if let Some(env_path) = sqlx_config.env_path {
            dotenvy::from_filename(env_path).ok();

            let url = dotenvy::var("DATABASE_URL").ok();

            if let Some(url) = url {
                return Some(url);
            }
        }

        None
    }
}

/// If the `DATABASE_URL` was available in the `env_path` specified in the `sqlx-config.json` file,
/// then it will be returned, otherwise it will check the command line option `--database-url`.
pub fn get_database_url(connect_opts: &ConnectOpts) -> String {
    let db_url_from_config = SqlxConfig::get_database_url();

    if let Some(db_url_from_config) = db_url_from_config {
        db_url_from_config
    } else {
        connect_opts.required_db_url().unwrap().to_string()
    }
}
