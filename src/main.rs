mod application;
mod domain;
mod infrastructure;
mod interfaces;

use std::sync::Arc;

use anyhow::Context;
use application::use_cases::ForwardProxyRequestUseCase;
use infrastructure::{
    config::AppConfig, http_client::ReqwestProxyGateway, logger::RequestLogger,
};
use interfaces::{cli::CliArgs, http::build_rocket};

fn print_startup(config: &AppConfig) {
    println!("ollama-proxy v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("remote  : {}", config.host);
    println!("local   : http://127.0.0.1:{}", config.port);
    println!("auth    : bearer token");
    println!("log     : {}", config.log_level.as_str());
    println!();
    println!("ready.");
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse_args();
    let config = AppConfig::from_cli(args)?;

    interfaces::cli::init_logging(config.log_level.as_str())?;

    let gateway =
        ReqwestProxyGateway::new(config.host.clone(), config.token.clone(), config.timeout)
            .context("failed to initialize upstream HTTP client")?;
    let use_case = Arc::new(ForwardProxyRequestUseCase::new(Arc::new(gateway)));
    let logger = Arc::new(RequestLogger::new());

    print_startup(&config);

    let rocket = build_rocket(config.port, use_case, logger, config.log_level);
    rocket.launch().await.context("failed to launch server")?;

    Ok(())
}
