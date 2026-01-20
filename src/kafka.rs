use anyhow::{Context, Result};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

use crate::config::Config;

pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(config: &Config) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.kafka_bootstrap_servers)
            .set("security.protocol", "SASL_SSL")
            .set("sasl.mechanism", &config.kafka_sasl_mechanism)
            .set("sasl.username", &config.kafka_sasl_username)
            .set("sasl.password", &config.kafka_sasl_password)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .create()
            .context("Failed to create Kafka producer")?;

        Ok(KafkaProducer { producer })
    }

    pub async fn send(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &str,
    ) -> Result<()> {
        let mut record = FutureRecord::to(topic).payload(payload);
        
        if let Some(k) = key {
            record = record.key(k);
        }

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| anyhow::anyhow!("Failed to send to Kafka: {}", e))?;

        tracing::debug!("Message sent to Kafka topic: {}", topic);
        Ok(())
    }
}