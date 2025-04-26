use clap::Parser;
use dotenvy;
use tokio;
use tracing::{self, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub mod routes;
pub mod views;

#[derive(Parser)]
struct Cli {
    #[arg(short = 'b', long = "bind-address", env = "BIND_ADDRESS", default_value_t = String::from("127.0.0.1"))]
    bind_address: String,

    #[arg(short, long = "port", env = "PORT", default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize dotenvy
    dotenvy::dotenv().ok();

    // Set up tracing with the default format subscriber
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug,info", env!("CARGO_CRATE_NAME")).into()),
        )
        .init();

    let args = Cli::parse();

    // serve the app
    let app = routes::create_router();
    let addr = format!("{}:{}", args.bind_address, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("started listener on {}", &addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
