use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::kafka::KafkaProducer;
use crate::crd::{Filter, Route};

#[derive(Clone)]
pub struct AppState {
    pub handlers: Arc<RwLock<HashMap<Uuid, HandlerConfig>>>,
    pub kafka_producer: Arc<KafkaProducer>,
    pub api_signing_key: String,
    pub external_url: String,
    pub namespace: String,
}

#[derive(Clone, Debug)]
pub struct HandlerConfig {
    pub topic: String,
    pub signature_key: Option<String>,
    pub filters: Option<Vec<Filter>>,
    pub routes: Option<Vec<Route>>,
}