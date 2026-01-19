# Webhook Operator for Kubernetes

A Rust-based Kubernetes operator that handles webhooks and forwards them to Kafka topics. Each tenant gets a unique webhook endpoint backed by a Custom Resource Definition (CRD).

## Features

- 🚀 Dynamic webhook handler provisioning via REST API
- 🔐 HMAC-SHA256 signature verification for webhooks
- 🎯 **Advanced filtering** - Discard unwanted events before they reach Kafka
- 🔀 **Smart routing** - Route events to different topics based on payload content
- 📨 Forwards webhook payloads and headers to Kafka
- ☸️ Native Kubernetes operator with CRD support
- 🔄 High availability with multi-replica deployment
- 🔍 Health and readiness probes
- 📊 Structured logging with tracing

## Architecture

```
Client -> POST /config (signed) -> Operator creates WebhookHandler CRD
                                    -> Returns /handler/<UUID>

Webhook -> POST /handler/<UUID> -> Verify signature (optional)
                                 -> Apply filters (optional)
                                 -> Determine topic via routing (optional)
                                 -> Send to Kafka topic
```

## Prerequisites

### For Customers Installing the Operator

1. **Kafka Cluster** with:
   - Bootstrap servers accessible from Kubernetes
   - SASL authentication configured
   - Topics created per tenant (or auto-create enabled)

2. **Kubernetes Cluster** (v1.20+)

3. **Required Permissions**:
   - Create CRDs
   - Create ConfigMaps and Secrets
   - Deploy applications

## Installation

### 1. Install CRD

```bash
kubectl apply -f k8s/crd.yaml
```

### 2. Configure Kafka Connection

Edit `k8s/config.yaml` and update:

```yaml
# ConfigMap: Update bootstrap servers
data:
  bootstrap_servers: "your-kafka-broker:9092"

# Secret: Update credentials
stringData:
  sasl_username: "your-username"
  sasl_password: "your-password"
  sasl_mechanism: "SCRAM-SHA-512"  # or PLAIN
```

Generate API signing key:
```bash
openssl rand -hex 32
```

Update the `api_signing_key` in `k8s/config.yaml`.

Apply configuration:
```bash
kubectl apply -f k8s/config.yaml
```

### 3. Deploy Operator

```bash
# Create RBAC resources
kubectl apply -f k8s/rbac.yaml

# Update image in deployment.yaml with your registry
# Then deploy
kubectl apply -f k8s/deployment.yaml
```

### 4. Verify Installation

```bash
# Check operator pods
kubectl get pods -l app=webhook-operator

# Check CRD installation
kubectl get crd webhookhandlers.webhooks.example.com

# Check service
kubectl get svc webhook-operator
```

## Building the Docker Image

```bash
# Build image
docker build -t your-registry/webhook-operator:latest .

# Push to registry
docker push your-registry/webhook-operator:latest
```

## Usage

### Create a Webhook Handler

**Simple handler (no filtering or routing):**

```bash
#!/bin/bash
API_KEY="your-api-signing-key"
TIMESTAMP=$(date +%s)
BODY='{"topic":"tenant-a.webhooks","signature_key":"whsec_tenant_secret"}'

# Generate signature
SIGNATURE=$(echo -n "${TIMESTAMP}.${BODY}" | \
  openssl dgst -sha256 -hmac "${API_KEY}" | \
  awk '{print $2}')

# Make request
curl -X POST https://webhooks.example.com/config \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Signature: sha256=${SIGNATURE}" \
  -d "${BODY}"
```

**Advanced handler with filtering and routing:**

For complex scenarios like Zoom webhooks where you need to:
- Filter out specific account IDs or event types
- Route events to different topics based on account

See [FILTERING_AND_ROUTING.md](FILTERING_AND_ROUTING.md) for detailed examples, or use the provided script:

```bash
API_KEY="your-key" ./examples/zoom-webhook-setup.sh
```

Response:
```json
{
  "handler_id": "789e4567-e89b-12d3-a456-426614174001",
  "webhook_url": "https://webhooks.example.com/handler/789e4567-e89b-12d3-a456-426614174001"
}
```

### Send Webhook Data

If signature verification is configured:

```bash
#!/bin/bash
WEBHOOK_SECRET="whsec_tenant_secret"
TIMESTAMP=$(date +%s)
PAYLOAD='{"event":"payment.succeeded","amount":1000}'

SIGNATURE=$(echo -n "${TIMESTAMP}.${PAYLOAD}" | \
  openssl dgst -sha256 -hmac "${WEBHOOK_SECRET}" | \
  awk '{print $2}')

curl -X POST https://webhooks.example.com/handler/789e4567-e89b-12d3-a456-426614174001 \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Signature: sha256=${SIGNATURE}" \
  -d "${PAYLOAD}"
```

Without signature verification:

```bash
curl -X POST https://webhooks.example.com/handler/789e4567-e89b-12d3-a456-426614174001 \
  -H "Content-Type: application/json" \
  -d '{"event":"payment.succeeded","amount":1000}'
```

### List Webhook Handlers

```bash
kubectl get webhookhandlers
# or short form:
kubectl get wh
```

Output:
```
NAME                                      TOPIC                 URL                                              READY   AGE
handler-789e4567-e89b-12d3-a456-426614   tenant-a.webhooks     https://webhooks.example.com/handler/789e...   true    5m
```

### View Handler Details

```bash
kubectl describe webhookhandler handler-789e4567-e89b-12d3-a456-426614174001
```

### Delete a Handler

```bash
kubectl delete webhookhandler handler-789e4567-e89b-12d3-a456-426614174001
```

## Kafka Message Format

Messages sent to Kafka have the following structure:

```json
{
  "headers": {
    "content-type": "application/json",
    "x-timestamp": "1640000000",
    "user-agent": "curl/7.68.0"
  },
  "body": {
    "event": "payment.succeeded",
    "amount": 1000
  },
  "received_at": "2024-01-15T10:30:45.123Z"
}
```

- **Key**: Handler UUID
- **Topic**: Configured topic for the handler
- **Partition**: Determined by Kafka (based on key)

## Security Considerations

### API Signing Key

- Store securely in Kubernetes Secret
- Rotate periodically
- Use strong random keys (32+ bytes)
- Never commit to version control

### Webhook Signature Keys

- Optional but recommended for production
- Each tenant can have their own key
- Prevents replay attacks (5-minute window)
- Uses HMAC-SHA256

### Network Security

- Use TLS/HTTPS for all external endpoints
- Consider mutual TLS for Kafka connections
- Use NetworkPolicies to restrict pod communication
- Enable Pod Security Standards

## Monitoring

### Logs

```bash
# View operator logs
kubectl logs -l app=webhook-operator -f

# View logs for specific pod
kubectl logs webhook-operator-xxxxx-yyyyy
```

### Metrics Endpoints

- `/health` - Liveness probe
- `/ready` - Readiness probe

### Common Log Messages

- `INFO: Loaded N existing handlers` - Controller initialized
- `INFO: Handler updated: <uuid> -> <topic>` - New handler created
- `INFO: Successfully processed webhook` - Webhook forwarded to Kafka
- `WARN: Invalid signature for handler` - Signature verification failed
- `ERROR: Failed to send to Kafka` - Kafka connection issue

## Troubleshooting

### Operator Not Starting

Check pod events:
```bash
kubectl describe pod webhook-operator-xxxxx-yyyyy
```

Common issues:
- Missing ConfigMap/Secret
- Invalid Kafka credentials
- RBAC permissions not applied

### Handler Not Found (404)

1. Check if CRD exists:
```bash
kubectl get webhookhandler handler-<uuid>
```

2. Check operator logs for watcher errors

3. Verify handler was created successfully

### Signature Verification Failing

1. Ensure timestamp is current (within 5 minutes)
2. Verify signature format: `sha256=<hex>`
3. Check signature key matches
4. Ensure message format: `{timestamp}.{body}`

### Kafka Connection Issues

1. Test connectivity from pod:
```bash
kubectl exec -it webhook-operator-xxxxx-yyyyy -- sh
# Try connecting to Kafka bootstrap servers
```

2. Verify SASL credentials
3. Check Kafka ACLs for the service account
4. Review Kafka broker logs

## Development

### Local Development

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies (Ubuntu/Debian)
sudo apt-get install cmake libssl-dev libsasl2-dev pkg-config

# Run tests
cargo test

# Run locally (requires kubeconfig)
export KAFKA_BOOTSTRAP_SERVERS="localhost:9092"
export KAFKA_SASL_USERNAME="admin"
export KAFKA_SASL_PASSWORD="admin-secret"
export API_SIGNING_KEY="test-key-123"
export EXTERNAL_URL="http://localhost:8080"
export NAMESPACE="default"

cargo run
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires Kubernetes cluster)
cargo test --test integration
```

### Code Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration loading
├── crd.rs               # WebhookHandler CRD definition
├── controller.rs        # Watch CRDs and maintain state
├── kafka.rs             # Kafka producer client
├── signature.rs         # HMAC signature verification
├── state.rs             # Shared application state
└── handlers/
    ├── config.rs        # POST /config endpoint
    ├── webhook.rs       # POST /handler/<uuid> endpoint
    └── health.rs        # Health check endpoints
```

## Configuration Reference

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `KAFKA_BOOTSTRAP_SERVERS` | Yes | - | Kafka bootstrap servers (comma-separated) |
| `KAFKA_SASL_USERNAME` | Yes | - | SASL username for Kafka authentication |
| `KAFKA_SASL_PASSWORD` | Yes | - | SASL password for Kafka authentication |
| `KAFKA_SASL_MECHANISM` | No | `SCRAM-SHA-512` | SASL mechanism (PLAIN, SCRAM-SHA-256, SCRAM-SHA-512) |
| `API_SIGNING_KEY` | Yes | - | Secret key for signing /config requests |
| `EXTERNAL_URL` | No | `http://localhost:8080` | External URL where webhooks are accessible |
| `NAMESPACE` | No | `default` | Kubernetes namespace to watch for CRDs |
| `RUST_LOG` | No | `info` | Log level (trace, debug, info, warn, error) |

## License

MIT

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## Support

For issues and questions:
- GitHub Issues: [your-repo-url]/issues
- Email: support@example.com