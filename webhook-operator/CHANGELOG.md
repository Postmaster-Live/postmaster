# Changelog

All notable changes to the Webhook Operator will be documented in this file.

## [2.0.0] - 2026-01-18

### Added

#### Filtering Feature
- **Event filtering** - Discard unwanted events before they reach Kafka
- Support for multiple filter operators:
  - `equals` / `not_equals` - Exact matching
  - `in` / `not_in` - Array membership
  - `contains` / `not_contains` - Substring matching
- Filters applied to webhook payload using JSONPath expressions
- All filters must pass for event to be processed
- Detailed logging when events are filtered out

#### Routing Feature
- **Dynamic topic routing** - Route events to different Kafka topics based on payload content
- Route based on any field in the webhook payload using JSONPath
- First-match routing logic with fallback to default topic
- Support for multiple routing rules per handler
- Account-specific topic routing (e.g., route Zoom events by account_id)
- Event-type based routing (e.g., separate topics for different webhook events)

#### New Modules
- `src/filter.rs` - Filtering and routing logic with comprehensive tests
- JSONPath extraction for nested field access
- Value matching across different types (string, number, boolean)

#### Documentation
- `FILTERING_AND_ROUTING.md` - Complete guide with examples
- Example scripts:
  - `examples/zoom-webhook-setup.sh` - Zoom meeting events with filtering and routing
  - `examples/zoom-test-webhook.sh` - Test suite for Zoom webhooks
- Real-world use cases:
  - Zoom meeting events (filter blocked accounts, route by account_id)
  - Stripe webhooks (filter test mode, route by event type)
  - GitHub webhooks (filter repositories, route by action)

#### API Changes
- `/config` endpoint now accepts optional `filters` and `routes` fields
- WebhookHandler CRD updated with new fields:
  - `spec.filters` - Array of filter rules
  - `spec.routes` - Array of routing rules
  - `spec.topic` - Now serves as default topic when no route matches

### Changed
- Handler configuration now includes filtering and routing rules
- Controller loads filters and routes into handler config
- Webhook processing order: Signature → Filters → Routing → Kafka
- Response message now includes target topic name

### Performance
- Filter evaluation: O(n) where n = number of filters
- Routing evaluation: O(m × k) where m = routes, k = mappings
- Negligible impact for typical use cases (< 1ms per webhook)

## [1.0.0] - 2026-01-15

### Initial Release

#### Core Features
- Kubernetes operator with Custom Resource Definitions
- Dynamic webhook handler provisioning via REST API
- HMAC-SHA256 signature verification
- Kafka producer with SASL authentication
- Multi-replica high availability deployment
- Health and readiness probes
- Structured logging with tracing

#### Security
- API signing key authentication for /config endpoint
- Optional per-handler webhook signature verification
- Replay attack prevention (5-minute timestamp window)
- Kubernetes RBAC integration

#### Infrastructure
- Docker multi-stage build for minimal image size
- ConfigMap for Kafka connection settings
- Secrets for sensitive credentials
- LoadBalancer service for external access
- Horizontal pod autoscaling support

#### Documentation
- Complete installation guide
- Local development setup with Docker Compose
- Example scripts for handler creation and testing
- Comprehensive README with troubleshooting

#### Developer Tools
- Makefile for common tasks
- Example curl commands
- Local Kafka setup with docker-compose
- Unit tests for signature verification and routing

---

## Upgrade Guide

### From 1.0.0 to 2.0.0

**Breaking Changes:** None - 2.0.0 is fully backward compatible

**New Optional Features:**

1. **Update CRD** (required for new features):
```bash
kubectl apply -f k8s/crd.yaml
```

2. **Existing handlers continue to work** without changes

3. **To use filtering/routing**, create new handlers with:
```json
{
  "topic": "default-topic",
  "filters": [ /* optional */ ],
  "routes": [ /* optional */ ]
}
```

**Migration:**

No migration needed. Existing handlers without filters/routes work exactly as before.

To add filtering/routing to existing handlers:
```bash
# Delete old handler
kubectl delete webhookhandler handler-<uuid>

# Create new handler with filters/routes
curl -X POST .../config -d '{"topic":"...","filters":[...],"routes":[...]}'
```

**Testing:**

Verify your upgrade:
```bash
# Test that operator started successfully
kubectl get pods -l app=webhook-operator

# Verify CRD has new fields
kubectl explain webhookhandler.spec.filters
kubectl explain webhookhandler.spec.routes

# Test creating a handler with filters
API_KEY="your-key" TOPIC="test" ./examples/zoom-webhook-setup.sh
```