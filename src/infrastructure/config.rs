use std::time::Duration;

use anyhow::{bail, Result};

use crate::domain::LogLevel;
use crate::interfaces::cli::CliArgs;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub token: String,
    pub port: u16,
    pub timeout: Option<Duration>,
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

        let timeout = if args.timeout < 0 {
            None
        } else {
            Some(Duration::from_secs(args.timeout as u64))
        };

        Ok(Self {
            host: args.host.trim_end_matches('/').to_string(),
            token: args.token,
            port: args.port,
            timeout,
            log_level: args.log_level,
        })
    }
}
