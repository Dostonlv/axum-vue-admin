use clap::Parser;
use core::str::FromStr;
use std::path::PathBuf;
use std::fs;
use tower_http::trace::TraceLayer;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use common::logging::init_logging;
use config::state;
use axum_vue_admin::common;
use axum_vue_admin::config;
use axum_vue_admin::routes;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing::info;
use utoipa::OpenApi;

use crate::common::function::subscribe_to_policy_updates;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    generate_config: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if cli.generate_config {
        let config_content = AppConfig::get_default_config();
        match fs::write("config.toml", config_content.trim()) {
            Ok(_) => println!("✅ Default 'config.toml' generated."),
            Err(e) => eprintln!("❌ Failed to generate config file: {}", e),
        }
        return;
    }
    let config_path = cli.config.unwrap_or_else(|| PathBuf::from("config.toml"));

    println!("Loading config from: {:?}", config_path);

    let config = match AppConfig::load_config(&config_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("❌ Cannot read config file {:?}. {}", config_path, err);
            eprintln!("💡 Run with '-g' to generate a default config file.");
            return;
        }
    };

    // `_guard` must stay alive for the whole of main
    let _guard = init_logging(&config.log);

    // debug!("http_proxy Env: {}", env::var("http_proxy").unwrap_or("No http_proxy".to_string()));
    // debug!("https_proxy Env: {}", env::var("https_proxy").unwrap_or("No https_proxy".to_string()));

    let app_state = state::AppState::new(&config)
        .await
        .expect("Failed to initialize state");

    // Spawn the background task that listens for policy updates
    let state_for_subscriber = app_state.clone();
    tokio::spawn(async move {
        subscribe_to_policy_updates(state_for_subscriber).await;
    });

    
    let (router, api) = OpenApiRouter::with_openapi(config::openapi::ApiDoc::openapi())
        .nest("/api", routes::api_router(app_state.clone()))
        .layer(TraceLayer::new_for_http())
        // .layer(middleware::from_fn_with_state(state.clone(), handle_audit_log_middleware))
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));

    let ipaddr = Ipv4Addr::from_str(&config.server.host.as_str());
    let ipaddr = match ipaddr {
        Ok(ipaddr) => ipaddr,
        Err(_) => {
            eprintln!("Invalid listen address");
            return;
        }
    };
    let addr = SocketAddr::new(IpAddr::V4(ipaddr), config.server.port);
    info!("Listening on {}", addr);
    // Run server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
