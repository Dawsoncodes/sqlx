use serde::{Deserialize, Serialize};
use std::fs;

/// Main struct for the sqlx-config.json
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlxConfig {
    pub env_path: String,
}

impl Default for SqlxConfig {
    fn default() -> Self {
        SqlxConfig {
            env_path: ".env".to_string(),
        }
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
            Err(e) => {
                // Create the file
                let default = SqlxConfig::default();

                let json = serde_json::to_string(&default)?;

                fs::write("sqlx-config.json", json)?;

                Ok(SqlxConfig::default())
            }
        }
    }
}
