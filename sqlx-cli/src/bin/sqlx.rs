use clap::Parser;
use console::style;
use sqlx_cli::{Opt, SqlxConfig};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    SqlxConfig::read()
        .ok()
        .ok_or(anyhow::anyhow!("Error reading sqlx-config.file"))
        .unwrap();

    // no special handling here
    if let Err(error) = sqlx_cli::run(Opt::parse()).await {
        println!("{} {}", style("error:").bold().red(), error);
        std::process::exit(1);
    }
}
