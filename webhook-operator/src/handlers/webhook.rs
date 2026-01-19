use axum::{
    extract::{Extension, Path},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::filter::{route_to_topic, should_process_event};
use crate::signature::verify_signature;
use crate::state::AppState;

#[derive(Serialize)]
pub struct WebhookResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
pub struct KafkaMessage {
    headers: serde_json::Value,
    body: serde_json::Value,
    received_at: String,
}

pub async fn handle_webhook(
    Extension(state): Extension<AppState>,
    Path(uuid): Path<Uuid>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<WebhookResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::debug!("Received webhook for handler: {}", uuid);

    // Look up handler configuration
    let handlers = state.handlers.read().await;
    let handler_config = handlers.get(&uuid).ok_or_else(|| {
        tracing::warn!("Handler not found: {}", uuid);
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Handler not found".to_string(),
            }),
        )
    })?;

    let default_topic = handler_config.topic.clone();
    let signature_key = handler_config.signature_key.clone();
    let filters = handler_config.filters.clone();
    let routes = handler_config.routes.clone();
    drop(handlers); // Release lock

    // Verify signature if configured
    if let Some(key) = signature_key {
        let signature = headers
            .get("x-signature")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Missing X-Signature header".to_string(),
                    }),
                )
            })?;

        let timestamp = headers
            .get("x-timestamp")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Missing X-Timestamp header".to_string(),
                    }),
                )
            })?;

        let is_valid = verify_signature(&key, timestamp, &body, signature).map_err(|e| {
            tracing::error!("Signature verification error for handler {}: {}", uuid, e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid signature".to_string(),
                }),
            )
        })?;

        if !is_valid {
            tracing::warn!("Invalid signature for handler: {}", uuid);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid signature".to_string(),
                }),
            ));
        }
    }

    // Parse body as JSON (or store as string if not valid JSON)
    let body_json: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|_| json!({ "raw": body }));

    // Apply filters if configured
    if let Some(filter_rules) = filters {
        match should_process_event(&body_json, &filter_rules) {
            Ok(should_process) => {
                if !should_process {
                    tracing::info!("Event filtered out for handler: {}", uuid);
                    return Ok(Json(WebhookResponse {
                        success: true,
                        message: "Event filtered, not sent to Kafka".to_string(),
                    }));
                }
            }
            Err(e) => {
                tracing::error!("Filter evaluation error for handler {}: {}", uuid, e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Filter error: {}", e),
                    }),
                ));
            }
        }
    }

    // Determine target topic using routing rules
    let target_topic = if let Some(route_rules) = routes {
        match route_to_topic(&body_json, &route_rules) {
            Ok(Some(routed_topic)) => routed_topic,
            Ok(None) => default_topic, // No route matched, use default
            Err(e) => {
                tracing::error!("Routing error for handler {}: {}", uuid, e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Routing error: {}", e),
                    }),
                ));
            }
        }
    } else {
        default_topic
    };

    // Convert headers to JSON
    let headers_json: serde_json::Value = headers
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_string(),
                v.to_str().unwrap_or("").to_string(),
            )
        })
        .collect::<serde_json::Map<_, _>>()
        .into();

    // Create Kafka message
    let kafka_message = KafkaMessage {
        headers: headers_json,
        body: body_json,
        received_at: chrono::Utc::now().to_rfc3339(),
    };

    let kafka_payload = serde_json::to_string(&kafka_message).map_err(|e| {
        tracing::error!("Failed to serialize Kafka message: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to process message".to_string(),
            }),
        )
    })?;

    // Send to Kafka
    state
        .kafka_producer
        .send(&target_topic, Some(&uuid.to_string()), &kafka_payload)
        .await
        .map_err(|e| {
            tracing::error!("Failed to send to Kafka for handler {}: {}", uuid, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to send to Kafka".to_string(),
                }),
            )
        })?;

    tracing::info!(
        "Successfully processed webhook for handler: {} -> topic: {}",
        uuid,
        target_topic
    );

    Ok(Json(WebhookResponse {
        success: true,
        message: format!("Webhook sent to topic: {}", target_topic),
    }))
}