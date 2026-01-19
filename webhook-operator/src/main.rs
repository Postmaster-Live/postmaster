mod config;
mod controller;
mod crd;
mod filter;
mod handlers;
mod kafka;
mod signature;
mod state;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::controller::watch_handlers;
use crate::kafka::KafkaProducer;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "webhook_operator=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting webhook operator");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded: namespace={}, external_url={}", 
        config.namespace, config.external_url);

    // Initialize Kafka producer
    let kafka_producer = KafkaProducer::new(&config)?;
    tracing::info!("Kafka producer initialized");

    // Initialize shared state
    let handlers = Arc::new(RwLock::new(std::collections::HashMap::new()));
    let state = AppState {
        handlers: handlers.clone(),
        kafka_producer: Arc::new(kafka_producer),
        api_signing_key: config.api_signing_key.clone(),
        external_url: config.external_url.clone(),
        namespace: config.namespace.clone(),
    };

    // Start controller to watch WebhookHandler CRDs
    let client = kube::Client::try_default().await?;
    tokio::spawn(watch_handlers(
        client.clone(),
        config.namespace.clone(),
        handlers.clone(),
    ));

    // Build HTTP router
    let app = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/ready", get(handlers::health::ready))
        .route("/config", post(handlers::config::create_handler))
        .route("/handler/:uuid", post(handlers::webhook::handle_webhook))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(state));

    // Start server
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}