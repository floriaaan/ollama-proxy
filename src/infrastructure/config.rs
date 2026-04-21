use anyhow::{bail, Result};

use crate::interfaces::cli::{CliArgs, LogLevel};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub token: String,
    pub port: u16,
    pub log_level: LogLevel,
}

impl AppConfig {
    pub fn from_cli(args: CliArgs) -> Result<Self> {
        if args.host.trim().is_empty() {
            bail!("--host must not be empty");
        }

        if args.token.trim().is_empty() {
            bail!("--token must not be empty");
        }

        Ok(Self {
            host: args.host.trim_end_matches('/').to_string(),
            token: args.token,
            port: args.port,
            log_level: args.log_level,
        })
    }
}
