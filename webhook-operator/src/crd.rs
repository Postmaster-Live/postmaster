use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "webhooks.example.com",
    version = "v1",
    kind = "WebhookHandler",
    namespaced,
    status = "WebhookHandlerStatus"
)]
#[kube(printcolumn = r#"{"name":"Topic", "type":"string", "jsonPath":".spec.topic"}"#)]
#[kube(printcolumn = r#"{"name":"URL", "type":"string", "jsonPath":".status.handlerUrl"}"#)]
#[kube(printcolumn = r#"{"name":"Ready", "type":"boolean", "jsonPath":".status.ready"}"#)]
pub struct WebhookHandlerSpec {
    /// Default topic for events that don't match any routing rules
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_key: Option<String>,
    /// Optional filters to discard events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<Filter>>,
    /// Optional routing rules to send events to different topics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routes: Option<Vec<Route>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct Filter {
    /// JSONPath expression to extract value (e.g., "$.payload.account_id")
    pub path: String,
    /// Operator: "equals", "not_equals", "in", "not_in", "contains", "not_contains"
    pub operator: String,
    /// Value(s) to compare against
    pub value: FilterValue,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(untagged)]
pub enum FilterValue {
    String(String),
    StringArray(Vec<String>),
    Number(i64),
    NumberArray(Vec<i64>),
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct Route {
    /// JSONPath expression to extract value for routing (e.g., "$.payload.account_id")
    pub path: String,
    /// Mapping of values to topics
    pub mapping: Vec<RouteMapping>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct RouteMapping {
    /// Value to match (e.g., "account123")
    pub value: String,
    /// Topic to route to when matched
    pub topic: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
pub struct WebhookHandlerStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler_url: Option<String>,
    #[serde(default)]
    pub ready: bool,
}