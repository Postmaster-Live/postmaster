# Local Development Guide

Guide for developing and testing the webhook operator locally.

## Prerequisites

### System Dependencies

**macOS:**
```bash
brew install cmake openssl@3 pkg-config
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y cmake libssl-dev libsasl2-dev pkg-config build-essential
```

**Fedora/RHEL:**
```bash
sudo dnf install cmake openssl-devel cyrus-sasl-devel pkgconfig gcc
```

### Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rustfmt clippy
```

### Kubernetes Tools

1. **Install kind (Kubernetes in Docker):**
```bash
# macOS
brew install kind

# Linux
curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-linux-amd64
chmod +x ./kind
sudo mv ./kind /usr/local/bin/kind
```

2. **Install kubectl:**
```bash
# macOS
brew install kubectl

# Linux
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl
sudo mv kubectl /usr/local/bin/
```

## Local Kafka Setup

### Option 1: Docker Compose (Recommended)

Start local Kafka cluster:

```bash
docker-compose up -d
```

This starts:
- Zookeeper on port 2181
- Kafka broker on port 9092
- Kafka UI on port 8090

Access Kafka UI: http://localhost:8090

Create test topics:
```bash
docker exec -it kafka kafka-topics --create \
  --bootstrap-server localhost:9092 \
  --topic test-topic \
  --partitions 3 \
  --replication-factor 1
```

### Option 2: Local Kafka Installation

Download and run Kafka manually if you prefer not to use Docker.

## Local Kubernetes Cluster

### Create kind Cluster

```bash
cat <<EOF | kind create cluster --config=-
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
name: webhook-dev
nodes:
- role: control-plane
  extraPortMappings:
  - containerPort: 30080
    hostPort: 8080
    protocol: TCP
EOF
```

### Install CRD

```bash
kubectl apply -f k8s/crd.yaml
```

### Create Test ConfigMap and Secrets

```bash
kubectl create configmap webhook-kafka-config \
  --from-literal=bootstrap_servers=host.docker.internal:9092

kubectl create secret generic webhook-kafka-secret \
  --from-literal=sasl_username=admin \
  --from-literal=sasl_password=admin-secret \
  --from-literal=sasl_mechanism=PLAIN

kubectl create secret generic webhook-operator-api \
  --from-literal=api_signing_key=test-key-12345
```

Note: `host.docker.internal` allows kind containers to reach your host machine where Kafka is running.

## Running the Operator Locally

### Method 1: Run Outside Cluster (Fastest for Development)

```bash
# Set environment variables
export KAFKA_BOOTSTRAP_SERVERS="localhost:9092"
export KAFKA_SASL_USERNAME="admin"
export KAFKA_SASL_PASSWORD="admin-secret"
export KAFKA_SASL_MECHANISM="PLAIN"
export API_SIGNING_KEY="test-key-12345"
export EXTERNAL_URL="http://localhost:8080"
export NAMESPACE="default"
export RUST_LOG="webhook_operator=debug,tower_http=debug"

# Run
cargo run
```

The operator will:
- Connect to your local Kafka
- Watch the kind cluster for WebhookHandler CRDs
- Listen on http://localhost:8080

### Method 2: Run in kind Cluster

Build and load image into kind:

```bash
# Build image
docker build -t webhook-operator:dev .

# Load into kind
kind load docker-image webhook-operator:dev --name webhook-dev

# Update deployment to use local image
# Edit k8s/deployment.yaml:
#   image: webhook-operator:dev
#   imagePullPolicy: Never

# Deploy
kubectl apply -f k8s/rbac.yaml
kubectl apply -f k8s/deployment.yaml

# Expose service
kubectl port-forward svc/webhook-operator 8080:80
```

## Development Workflow

### 1. Make Code Changes

Edit files in `src/`

### 2. Run Tests

```bash
# Unit tests
cargo test

# With logging
RUST_LOG=debug cargo test -- --nocapture

# Specific test
cargo test test_verify_signature
```

### 3. Format and Lint

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

### 4. Test Locally

```bash
# Run operator
cargo run

# In another terminal, create a handler
API_KEY="test-key-12345" \
TOPIC="test-topic" \
WEBHOOK_URL="http://localhost:8080" \
./examples/create-handler.sh

# Send a webhook
HANDLER_URL="http://localhost:8080/handler/YOUR-UUID-HERE" \
PAYLOAD='{"test":"data"}' \
./examples/send-webhook.sh
```

### 5. Verify in Kafka

```bash
# Watch messages
docker exec -it kafka kafka-console-consumer \
  --bootstrap-server localhost:9092 \
  --topic test-topic \
  --from-beginning
```

## Testing Different Scenarios

### Test Signature Verification

```bash
# Create handler WITH signature key
API_KEY="test-key-12345" \
TOPIC="secure-topic" \
SIGNATURE_KEY="whsec_my_secret" \
WEBHOOK_URL="http://localhost:8080" \
./examples/create-handler.sh

# Send webhook WITH signature (should succeed)
HANDLER_URL="http://localhost:8080/handler/YOUR-UUID" \
SIGNATURE_KEY="whsec_my_secret" \
PAYLOAD='{"event":"test"}' \
./examples/send-webhook.sh

# Send webhook WITHOUT signature (should fail)
curl -X POST http://localhost:8080/handler/YOUR-UUID \
  -H "Content-Type: application/json" \
  -d '{"event":"test"}'
```

### Test Invalid Signature

```bash
# Wrong signature key (should fail)
HANDLER_URL="http://localhost:8080/handler/YOUR-UUID" \
SIGNATURE_KEY="wrong-key" \
PAYLOAD='{"event":"test"}' \
./examples/send-webhook.sh
```

### Test Replay Attack Prevention

```bash
# Old timestamp (should fail)
OLD_TIMESTAMP=$(($(date +%s) - 400))  # 400 seconds ago
PAYLOAD='{"test":"data"}'
SIGNATURE=$(echo -n "${OLD_TIMESTAMP}.${PAYLOAD}" | \
  openssl dgst -sha256 -hmac "whsec_my_secret" | awk '{print $2}')

curl -X POST http://localhost:8080/handler/YOUR-UUID \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${OLD_TIMESTAMP}" \
  -H "X-Signature: sha256=${SIGNATURE}" \
  -d "${PAYLOAD}"
```

### Test Handler Not Found

```bash
# Random UUID that doesn't exist
curl -X POST http://localhost:8080/handler/00000000-0000-0000-0000-000000000000 \
  -H "Content-Type: application/json" \
  -d '{"test":"data"}'
```

## Debugging

### Enable Debug Logging

```bash
export RUST_LOG="webhook_operator=trace,rdkafka=debug,tower_http=trace"
cargo run
```

### Use rust-lldb or rust-gdb

```bash
# Build with debug symbols
cargo build

# Debug
rust-lldb target/debug/webhook-operator
# or
rust-gdb target/debug/webhook-operator
```

### Attach to Running Process

```bash
# In kind cluster
kubectl get pods -l app=webhook-operator
kubectl exec -it webhook-operator-xxxxx-yyyyy -- /bin/sh

# Check logs
kubectl logs -f webhook-operator-xxxxx-yyyyy
```

## Performance Testing

### Load Testing with Apache Bench

```bash
# Create handler first
HANDLER_UUID="your-handler-uuid"

# Simple load test (100 requests, 10 concurrent)
ab -n 100 -c 10 -p payload.json -T application/json \
  http://localhost:8080/handler/${HANDLER_UUID}
```

### Using k6

```javascript
// loadtest.js
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 10,
  duration: '30s',
};

export default function() {
  const url = 'http://localhost:8080/handler/YOUR-UUID';
  const payload = JSON.stringify({
    event: 'load.test',
    timestamp: Date.now()
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  let res = http.post(url, payload, params);
  check(res, {
    'is status 200': (r) => r.status === 200,
  });
}
```

Run:
```bash
k6 run loadtest.js
```

## Kafka Monitoring

### View Topics

```bash
docker exec -it kafka kafka-topics --list \
  --bootstrap-server localhost:9092
```

### Check Consumer Lag

```bash
docker exec -it kafka kafka-consumer-groups --list \
  --bootstrap-server localhost:9092
```

### View Messages

```bash
# All messages from beginning
docker exec -it kafka kafka-console-consumer \
  --bootstrap-server localhost:9092 \
  --topic test-topic \
  --from-beginning

# Only new messages
docker exec -it kafka kafka-console-consumer \
  --bootstrap-server localhost:9092 \
  --topic test-topic
```

## Clean Up

### Stop Local Services

```bash
# Stop Kafka
docker-compose down

# Delete kind cluster
kind delete cluster --name webhook-dev
```

### Clean Build Artifacts

```bash
cargo clean
```

## IDE Setup

### VS Code

Install extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Kubernetes
- Docker

Example `.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug webhook-operator",
      "cargo": {
        "args": ["build", "--bin=webhook-operator"]
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "KAFKA_BOOTSTRAP_SERVERS": "localhost:9092",
        "KAFKA_SASL_USERNAME": "admin",
        "KAFKA_SASL_PASSWORD": "admin-secret",
        "KAFKA_SASL_MECHANISM": "PLAIN",
        "API_SIGNING_KEY": "test-key-12345",
        "EXTERNAL_URL": "http://localhost:8080",
        "NAMESPACE": "default",
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

### IntelliJ IDEA / CLion

Install plugins:
- Rust
- Kubernetes

Configure run configuration with environment variables listed above.

## Common Issues

### "Address already in use"

Port 8080 is already taken:
```bash
# Find what's using it
lsof -i :8080

# Kill the process or change port
export EXTERNAL_URL="http://localhost:8081"
```

### Cannot connect to Kafka from kind

Make sure you're using `host.docker.internal:9092` instead of `localhost:9092` in the ConfigMap.

### CRD changes not recognized

Regenerate and reapply:
```bash
kubectl delete crd webhookhandlers.webhooks.example.com
kubectl apply -f k8s/crd.yaml
```

### Tests failing

Check that local Kafka is running:
```bash
docker-compose ps
```

## Tips

1. **Hot Reload**: Use `cargo watch -x run` for auto-reload on file changes
2. **Faster Builds**: Use `cargo build --release` for optimized builds
3. **Parallel Tests**: `cargo test -- --test-threads=1` to avoid test conflicts
4. **Test Coverage**: Install `cargo-tarpaulin` for coverage reports

## Next Steps

Once comfortable with local development:
1. Review the [README](README.md) for full usage
2. See [INSTALL](INSTALL.md) for production deployment
3. Write integration tests
4. Add monitoring and metrics