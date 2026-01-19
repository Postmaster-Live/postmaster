use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::crd::{Filter, FilterValue, Route};

/// Evaluates all filters against the JSON payload
/// Returns true if the event should be processed (passes all filters)
/// Returns false if the event should be discarded (fails any filter)
pub fn should_process_event(payload: &Value, filters: &[Filter]) -> Result<bool> {
    for filter in filters {
        if !evaluate_filter(payload, filter)? {
            tracing::debug!("Event filtered out by rule: path={}, operator={}", 
                filter.path, filter.operator);
            return Ok(false);
        }
    }
    Ok(true)
}

/// Evaluates a single filter against the payload
fn evaluate_filter(payload: &Value, filter: &Filter) -> Result<bool> {
    let extracted_value = extract_json_path(payload, &filter.path)?;
    
    match filter.operator.as_str() {
        "equals" => Ok(match_equals(&extracted_value, &filter.value)),
        "not_equals" => Ok(!match_equals(&extracted_value, &filter.value)),
        "in" => Ok(match_in(&extracted_value, &filter.value)),
        "not_in" => Ok(!match_in(&extracted_value, &filter.value)),
        "contains" => Ok(match_contains(&extracted_value, &filter.value)),
        "not_contains" => Ok(!match_contains(&extracted_value, &filter.value)),
        _ => Err(anyhow!("Unknown filter operator: {}", filter.operator)),
    }
}

/// Determines the target topic based on routing rules
/// Returns the matched topic or None if no rule matches (use default topic)
pub fn route_to_topic(payload: &Value, routes: &[Route]) -> Result<Option<String>> {
    for route in routes {
        let extracted_value = extract_json_path(payload, &route.path)?;
        
        // Try to match against each mapping
        for mapping in &route.mapping {
            if value_matches_string(&extracted_value, &mapping.value) {
                tracing::debug!("Event routed to topic '{}' based on path={}, value={}", 
                    mapping.topic, route.path, mapping.value);
                return Ok(Some(mapping.topic.clone()));
            }
        }
    }
    
    Ok(None) // No route matched, use default topic
}

/// Extracts a value from JSON using a simplified JSONPath-like syntax
/// Supports: $.field, $.field.nested, $.array[0]
fn extract_json_path(payload: &Value, path: &str) -> Result<Value> {
    let path = path.strip_prefix("$.").unwrap_or(path);
    let parts: Vec<&str> = path.split('.').collect();
    
    let mut current = payload;
    
    for part in parts {
        // Handle array indexing like "field[0]"
        if let Some((field, index_str)) = part.split_once('[') {
            let index_str = index_str.trim_end_matches(']');
            let index: usize = index_str.parse()
                .map_err(|_| anyhow!("Invalid array index: {}", index_str))?;
            
            current = current.get(field)
                .ok_or_else(|| anyhow!("Field not found: {}", field))?;
            
            current = current.get(index)
                .ok_or_else(|| anyhow!("Array index out of bounds: {}", index))?;
        } else {
            current = current.get(part)
                .ok_or_else(|| anyhow!("Field not found: {}", part))?;
        }
    }
    
    Ok(current.clone())
}

fn match_equals(extracted: &Value, filter_value: &FilterValue) -> bool {
    match filter_value {
        FilterValue::String(s) => {
            if let Some(extracted_str) = extracted.as_str() {
                extracted_str == s
            } else {
                false
            }
        }
        FilterValue::Number(n) => {
            if let Some(extracted_num) = extracted.as_i64() {
                extracted_num == *n
            } else {
                false
            }
        }
        _ => false,
    }
}

fn match_in(extracted: &Value, filter_value: &FilterValue) -> bool {
    match filter_value {
        FilterValue::StringArray(arr) => {
            if let Some(extracted_str) = extracted.as_str() {
                arr.contains(&extracted_str.to_string())
            } else {
                false
            }
        }
        FilterValue::NumberArray(arr) => {
            if let Some(extracted_num) = extracted.as_i64() {
                arr.contains(&extracted_num)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn match_contains(extracted: &Value, filter_value: &FilterValue) -> bool {
    match filter_value {
        FilterValue::String(s) => {
            if let Some(extracted_str) = extracted.as_str() {
                extracted_str.contains(s)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn value_matches_string(value: &Value, target: &str) -> bool {
    if let Some(s) = value.as_str() {
        s == target
    } else if let Some(n) = value.as_i64() {
        n.to_string() == target
    } else if let Some(b) = value.as_bool() {
        b.to_string() == target
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_json_path_simple() {
        let payload = json!({
            "event": "meeting.started",
            "payload": {
                "account_id": "acc123"
            }
        });

        let result = extract_json_path(&payload, "$.event").unwrap();
        assert_eq!(result, json!("meeting.started"));

        let result = extract_json_path(&payload, "$.payload.account_id").unwrap();
        assert_eq!(result, json!("acc123"));
    }

    #[test]
    fn test_filter_equals() {
        let payload = json!({
            "payload": {
                "account_id": "acc123"
            }
        });

        let filter = Filter {
            path: "$.payload.account_id".to_string(),
            operator: "equals".to_string(),
            value: FilterValue::String("acc123".to_string()),
        };

        assert!(evaluate_filter(&payload, &filter).unwrap());

        let filter_no_match = Filter {
            path: "$.payload.account_id".to_string(),
            operator: "equals".to_string(),
            value: FilterValue::String("acc999".to_string()),
        };

        assert!(!evaluate_filter(&payload, &filter_no_match).unwrap());
    }

    #[test]
    fn test_filter_not_in() {
        let payload = json!({
            "payload": {
                "account_id": "acc123"
            }
        });

        let filter = Filter {
            path: "$.payload.account_id".to_string(),
            operator: "not_in".to_string(),
            value: FilterValue::StringArray(vec![
                "blocked1".to_string(),
                "blocked2".to_string(),
            ]),
        };

        assert!(evaluate_filter(&payload, &filter).unwrap());

        let filter_blocked = Filter {
            path: "$.payload.account_id".to_string(),
            operator: "not_in".to_string(),
            value: FilterValue::StringArray(vec![
                "acc123".to_string(),
                "blocked2".to_string(),
            ]),
        };

        assert!(!evaluate_filter(&payload, &filter_blocked).unwrap());
    }

    #[test]
    fn test_filter_event_type() {
        let payload = json!({
            "event": "meeting.started"
        });

        // Only allow meeting.started and meeting.ended
        let filter = Filter {
            path: "$.event".to_string(),
            operator: "in".to_string(),
            value: FilterValue::StringArray(vec![
                "meeting.started".to_string(),
                "meeting.ended".to_string(),
            ]),
        };

        assert!(evaluate_filter(&payload, &filter).unwrap());
    }

    #[test]
    fn test_route_to_topic() {
        let payload = json!({
            "payload": {
                "account_id": "acc123"
            }
        });

        let routes = vec![Route {
            path: "$.payload.account_id".to_string(),
            mapping: vec![
                crate::crd::RouteMapping {
                    value: "acc123".to_string(),
                    topic: "zoom-acc123".to_string(),
                },
                crate::crd::RouteMapping {
                    value: "acc456".to_string(),
                    topic: "zoom-acc456".to_string(),
                },
            ],
        }];

        let result = route_to_topic(&payload, &routes).unwrap();
        assert_eq!(result, Some("zoom-acc123".to_string()));
    }

    #[test]
    fn test_route_no_match() {
        let payload = json!({
            "payload": {
                "account_id": "acc999"
            }
        });

        let routes = vec![Route {
            path: "$.payload.account_id".to_string(),
            mapping: vec![
                crate::crd::RouteMapping {
                    value: "acc123".to_string(),
                    topic: "zoom-acc123".to_string(),
                },
            ],
        }];

        let result = route_to_topic(&payload, &routes).unwrap();
        assert_eq!(result, None); // Should use default topic
    }

    #[test]
    fn test_complex_zoom_scenario() {
        let payload = json!({
            "event": "meeting.started",
            "payload": {
                "account_id": "zoom_acc_123",
                "object": {
                    "id": "123456789"
                }
            }
        });

        // Filter: Discard events from blocked accounts
        let filters = vec![Filter {
            path: "$.payload.account_id".to_string(),
            operator: "not_in".to_string(),
            value: FilterValue::StringArray(vec![
                "blocked_acc_1".to_string(),
                "blocked_acc_2".to_string(),
            ]),
        }];

        assert!(should_process_event(&payload, &filters).unwrap());

        // Route: Send to account-specific topic
        let routes = vec![Route {
            path: "$.payload.account_id".to_string(),
            mapping: vec![
                crate::crd::RouteMapping {
                    value: "zoom_acc_123".to_string(),
                    topic: "zoom.account-123.events".to_string(),
                },
            ],
        }];

        let topic = route_to_topic(&payload, &routes).unwrap();
        assert_eq!(topic, Some("zoom.account-123.events".to_string()));
    }
}