use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub kafka_bootstrap_servers: String,
    pub kafka_sasl_username: String,
    pub kafka_sasl_password: String,
    pub kafka_sasl_mechanism: String,
    pub api_signing_key: String,
    pub external_url: String,
    pub namespace: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            kafka_bootstrap_servers: env::var("KAFKA_BOOTSTRAP_SERVERS")
                .context("KAFKA_BOOTSTRAP_SERVERS must be set")?,
            kafka_sasl_username: env::var("KAFKA_SASL_USERNAME")
                .context("KAFKA_SASL_USERNAME must be set")?,
            kafka_sasl_password: env::var("KAFKA_SASL_PASSWORD")
                .context("KAFKA_SASL_PASSWORD must be set")?,
            kafka_sasl_mechanism: env::var("KAFKA_SASL_MECHANISM")
                .unwrap_or_else(|_| "SCRAM-SHA-512".to_string()),
            api_signing_key: env::var("API_SIGNING_KEY")
                .context("API_SIGNING_KEY must be set")?,
            external_url: env::var("EXTERNAL_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            namespace: env::var("NAMESPACE")
                .unwrap_or_else(|_| "default".to_string()),
        })
    }
}