use axum::{extract::Extension, http::StatusCode, Json};
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<HealthDetails>,
}

#[derive(Serialize)]
pub struct HealthDetails {
    kafka: String,
    kubernetes: String,
    handlers_loaded: usize,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        details: None,
    })
}

pub async fn ready(
    Extension(state): Extension<AppState>,
) -> (StatusCode, Json<HealthResponse>) {
    // Check multiple health indicators
    let kafka_status: String;
    let k8s_status: String;
    let mut is_ready = true;

    // 1. Check Kafka connection by attempting to get metadata
    match check_kafka_connection(&state).await {
        Ok(true) => kafka_status = "connected".to_string(),
        Ok(false) => {
            kafka_status = "unreachable".to_string();
            is_ready = false;
        }
        Err(e) => {
            kafka_status = format!("error: {}", e);
            is_ready = false;
        }
    }

    // 2. Check Kubernetes API connection
    match check_kubernetes_connection().await {
        Ok(true) => k8s_status = "connected".to_string(),
        Ok(false) => {
            k8s_status = "unreachable".to_string();
            is_ready = false;
        }
        Err(e) => {
            k8s_status = format!("error: {}", e);
            is_ready = false;
        }
    }

    // 3. Check that handlers are loaded
    let handlers_count = state.handlers.read().await.len();
    
    // If we have no handlers loaded but Kafka and K8s are OK, still consider ready
    // (it's valid to have zero handlers configured)

    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = HealthResponse {
        status: if is_ready { "ready" } else { "not_ready" }.to_string(),
        details: Some(HealthDetails {
            kafka: kafka_status,
            kubernetes: k8s_status,
            handlers_loaded: handlers_count,
        }),
    };

    (status_code, Json(response))
}

async fn check_kafka_connection(state: &AppState) -> Result<bool, String> {
    // Test Kafka connection by sending to a test topic with timeout
    // We use a very short timeout since this is a health check
    let test_result = timeout(
        Duration::from_secs(2),
        state.kafka_producer.send(
            "__health_check__", // Use a special topic or check metadata
            None,
            r#"{"type":"health_check"}"#,
        ),
    )
    .await;

    match test_result {
        Ok(Ok(_)) => Ok(true),
        Ok(Err(e)) => {
            // If the error is just that the topic doesn't exist, Kafka is still reachable
            let err_msg = e.to_string();
            if err_msg.contains("UnknownTopicOrPartition") {
                Ok(true) // Kafka is reachable, just topic doesn't exist
            } else {
                tracing::warn!("Kafka health check failed: {}", e);
                Ok(false)
            }
        }
        Err(_) => {
            tracing::warn!("Kafka health check timeout");
            Ok(false)
        }
    }
}

async fn check_kubernetes_connection() -> Result<bool, String> {
    // Try to connect to Kubernetes API
    match timeout(Duration::from_secs(2), kube::Client::try_default()).await {
        Ok(Ok(_client)) => Ok(true),
        Ok(Err(e)) => {
            tracing::warn!("Kubernetes API connection failed: {}", e);
            Ok(false)
        }
        Err(_) => {
            tracing::warn!("Kubernetes API connection timeout");
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_always_succeeds() {
        let response = health().await;
        assert_eq!(response.0.status, "healthy");
    }
}