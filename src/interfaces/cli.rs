use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Info,
    Debug,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "ollama-proxy",
    version,
    about = "Local authenticated Ollama HTTP proxy"
)]
pub struct CliArgs {
    #[arg(long)]
    pub host: String,

    #[arg(long)]
    pub token: String,

    #[arg(long, default_value_t = 11434)]
    pub port: u16,

    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,
}

impl CliArgs {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

pub fn init_logging(level: &str) -> Result<()> {
    let filter = EnvFilter::builder().parse_lossy(format!(
        "ollama_proxy={},rocket=off,rocket_http=off",
        level
    ));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .compact()
        .try_init()
        .map_err(|err| anyhow!(err.to_string()))?;

    Ok(())
}
