use futures::StreamExt;
use kube::{
    api::ListParams,
    runtime::{watcher, WatchStreamExt},
    Api, Client,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::crd::WebhookHandler;
use crate::state::HandlerConfig;

pub async fn watch_handlers(
    client: Client,
    namespace: String,
    handlers: Arc<RwLock<HashMap<Uuid, HandlerConfig>>>,
) {
    tracing::info!("Starting WebhookHandler watcher for namespace: {}", namespace);

    let api: Api<WebhookHandler> = Api::namespaced(client, &namespace);

    // Load existing handlers first
    match api.list(&ListParams::default()).await {
        Ok(list) => {
            let mut map = handlers.write().await;
            for handler in list.items {
                if let Some(uuid) = parse_uuid_from_name(&handler.metadata.name) {
                    let config = HandlerConfig {
                        topic: handler.spec.topic.clone(),
                        signature_key: handler.spec.signature_key.clone(),
                        filters: handler.spec.filters.clone(),
                        routes: handler.spec.routes.clone(),
                    };
                    map.insert(uuid, config);
                    tracing::info!("Loaded existing handler: {} -> {}", uuid, handler.spec.topic);
                }
            }
            tracing::info!("Loaded {} existing handlers", map.len());
        }
        Err(e) => {
            tracing::error!("Failed to list existing handlers: {}", e);
        }
    }

    // Watch for changes
    let stream = watcher(api, watcher::Config::default()).applied_objects();
    futures::pin_mut!(stream);

    while let Some(result) = stream.next().await {
        match result {
            Ok(handler) => {
                if let Some(uuid) = parse_uuid_from_name(&handler.metadata.name) {
                    let config = HandlerConfig {
                        topic: handler.spec.topic.clone(),
                        signature_key: handler.spec.signature_key.clone(),
                        filters: handler.spec.filters.clone(),
                        routes: handler.spec.routes.clone(),
                    };
                    handlers.write().await.insert(uuid, config);
                    tracing::info!("Handler updated: {} -> {}", uuid, handler.spec.topic);
                }
            }
            Err(e) => {
                tracing::error!("Watch error: {}", e);
            }
        }
    }

    tracing::warn!("Handler watcher stream ended");
}

fn parse_uuid_from_name(name: &Option<String>) -> Option<Uuid> {
    name.as_ref()?
        .strip_prefix("handler-")
        .and_then(|s| Uuid::parse_str(s).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid_from_name() {
        let uuid = Uuid::new_v4();
        let name = format!("handler-{}", uuid);
        assert_eq!(parse_uuid_from_name(&Some(name)), Some(uuid));
    }

    #[test]
    fn test_parse_uuid_from_name_invalid() {
        assert_eq!(parse_uuid_from_name(&Some("invalid".to_string())), None);
        assert_eq!(parse_uuid_from_name(&Some("handler-invalid".to_string())), None);
        assert_eq!(parse_uuid_from_name(&None), None);
    }
}