use axum::{
    extract::Extension,
    http::{HeaderMap, StatusCode},
    Json,
};
use kube::{api::PostParams, Api, Client};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crd::{WebhookHandler, WebhookHandlerSpec, WebhookHandlerStatus, Filter, Route};
use crate::signature::verify_signature;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ConfigRequest {
    topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<Vec<Filter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    routes: Option<Vec<Route>>,
}

#[derive(Serialize)]
pub struct ConfigResponse {
    handler_id: Uuid,
    webhook_url: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

pub async fn create_handler(
    Extension(state): Extension<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Extract and verify signature
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

    // Verify signature
    let is_valid = verify_signature(&state.api_signing_key, timestamp, &body, signature)
        .map_err(|e| {
            tracing::error!("Signature verification error: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid signature".to_string(),
                }),
            )
        })?;

    if !is_valid {
        tracing::warn!("Invalid signature for /config request");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid signature".to_string(),
            }),
        ));
    }

    // Parse request body
    let req: ConfigRequest = serde_json::from_str(&body).map_err(|e| {
        tracing::error!("Failed to parse request body: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid request body: {}", e),
            }),
        )
    })?;

    // Generate UUID for handler
    let handler_id = Uuid::new_v4();
    let handler_name = format!("handler-{}", handler_id);

    tracing::info!("Creating webhook handler: {} for topic: {}", handler_id, req.topic);

    // Create Kubernetes client
    let client = Client::try_default().await.map_err(|e| {
        tracing::error!("Failed to create Kubernetes client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
            }),
        )
    })?;

    let api: Api<WebhookHandler> = Api::namespaced(client, &state.namespace);

    // Create WebhookHandler resource
    let webhook_url = format!("{}/handler/{}", state.external_url, handler_id);
    
    let handler = WebhookHandler {
        metadata: kube::api::ObjectMeta {
            name: Some(handler_name.clone()),
            namespace: Some(state.namespace.clone()),
            ..Default::default()
        },
        spec: WebhookHandlerSpec {
            topic: req.topic.clone(),
            signature_key: req.signature_key,
            filters: req.filters,
            routes: req.routes,
        },
        status: Some(WebhookHandlerStatus {
            handler_url: Some(webhook_url.clone()),
            ready: true,
        }),
    };

    api.create(&PostParams::default(), &handler)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create WebhookHandler CRD: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to create handler: {}", e),
                }),
            )
        })?;

    tracing::info!("Successfully created handler: {}", handler_id);

    Ok(Json(ConfigResponse {
        handler_id,
        webhook_url,
    }))
}