use clap::Parser;
use dotenvy;
use mash_todo::{db, routes, state::AppState, todos::TodoSqliteDao};
use tokio;
use tracing::{self, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Parser)]
struct Cli {
    #[arg(short = 'b', long = "bind-address", env = "BIND_ADDRESS", default_value_t = String::from("127.0.0.1"))]
    bind_address: String,

    #[arg(short, long = "port", env = "PORT", default_value_t = 3000)]
    port: u16,

    #[arg(short = 'd', long = "database-url", env = "DATABASE_URL", default_value_t = String::from("sqlite://db/app.db"))]
    database_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize dotenvy
    dotenvy::dotenv().ok();

    // Set up tracing with the default format subscriber
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("{}=debug,info", env!("CARGO_CRATE_NAME")).into()
        }))
        .init();

    let args = Cli::parse();

    // database
    let pool = db::create_pool(&args.database_url).await?;

    // construct app dependenciess
    let app_state = AppState::new(TodoSqliteDao::new(pool));

    // serve the app
    let app = routes::create_router(app_state);
    let addr = format!("{}:{}", args.bind_address, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("started listener on {}", &addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
