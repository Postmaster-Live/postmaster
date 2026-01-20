# Installation Guide

Complete guide for installing and configuring the Webhook Operator.

## Quick Start

```bash
# 1. Install CRD
kubectl apply -f k8s/crd.yaml

# 2. Configure (update with your values first!)
vim k8s/config.yaml
kubectl apply -f k8s/config.yaml

# 3. Deploy operator
kubectl apply -f k8s/rbac.yaml
kubectl apply -f k8s/deployment.yaml

# 4. Verify
kubectl get pods -l app=webhook-operator
```

## Detailed Installation Steps

### Prerequisites

Before installing, ensure you have:

1. **Kubernetes Cluster** (v1.20 or higher)
   - `kubectl` configured to access your cluster
   - Cluster admin permissions

2. **Kafka Cluster** 
   - Bootstrap servers accessible from Kubernetes
   - SASL authentication configured
   - Service account with appropriate ACLs

3. **External Access**
   - LoadBalancer or Ingress capability
   - DNS entry for webhook endpoint (optional but recommended)

### Step 1: Install Custom Resource Definition

The CRD defines the `WebhookHandler` resource type:

```bash
kubectl apply -f k8s/crd.yaml
```

Verify CRD installation:

```bash
kubectl get crd webhookhandlers.webhooks.example.com
```

You should see:
```
NAME                                      CREATED AT
webhookhandlers.webhooks.example.com     2024-01-15T10:00:00Z
```

### Step 2: Configure Kafka Connection

#### Generate API Signing Key

First, generate a secure random key for API authentication:

```bash
openssl rand -hex 32
```

Save this key - you'll need it for making API requests.

#### Edit Configuration File

Open `k8s/config.yaml` and update the following values:

```yaml
# ConfigMap - Update bootstrap servers
apiVersion: v1
kind: ConfigMap
metadata:
  name: webhook-kafka-config
data:
  bootstrap_servers: "YOUR-KAFKA-BROKER-1:9092,YOUR-KAFKA-BROKER-2:9092"

---
# Secret - Update Kafka credentials
apiVersion: v1
kind: Secret
metadata:
  name: webhook-kafka-secret
type: Opaque
stringData:
  sasl_username: "YOUR-KAFKA-USERNAME"
  sasl_password: "YOUR-KAFKA-PASSWORD"
  sasl_mechanism: "SCRAM-SHA-512"  # or PLAIN, SCRAM-SHA-256

---
# Secret - Update API key
apiVersion: v1
kind: Secret
metadata:
  name: webhook-operator-api
type: Opaque
stringData:
  api_signing_key: "YOUR-GENERATED-API-KEY-FROM-ABOVE"
```

#### Apply Configuration

```bash
kubectl apply -f k8s/config.yaml
```

Verify:

```bash
kubectl get configmap webhook-kafka-config
kubectl get secret webhook-kafka-secret
kubectl get secret webhook-operator-api
```

### Step 3: Configure RBAC

Apply the RBAC configuration to grant the operator necessary permissions:

```bash
kubectl apply -f k8s/rbac.yaml
```

This creates:
- ServiceAccount: `webhook-operator`
- ClusterRole: `webhook-operator` (permissions for CRDs, ConfigMaps, Secrets)
- ClusterRoleBinding: Binds the role to the service account

Verify:

```bash
kubectl get serviceaccount webhook-operator
kubectl get clusterrole webhook-operator
kubectl get clusterrolebinding webhook-operator
```

### Step 4: Build and Push Docker Image

#### Option A: Build Locally

```bash
# Build
docker build -t your-registry.com/webhook-operator:v1.0.0 .

# Push
docker push your-registry.com/webhook-operator:v1.0.0
```

#### Option B: Use GitHub Actions / CI/CD

Set up automated builds in your CI/CD pipeline. Example GitHub Actions:

```yaml
# .github/workflows/build.yml
name: Build and Push
on:
  push:
    tags:
      - 'v*'
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Build and push
      run: |
        docker build -t ${{ secrets.REGISTRY }}/webhook-operator:${GITHUB_REF#refs/tags/} .
        docker push ${{ secrets.REGISTRY }}/webhook-operator:${GITHUB_REF#refs/tags/}
```

### Step 5: Update Deployment Configuration

Edit `k8s/deployment.yaml` and update:

1. **Image name**:
```yaml
spec:
  containers:
  - name: operator
    image: your-registry.com/webhook-operator:v1.0.0  # Update this
```

2. **External URL** (where your webhooks will be accessible):
```yaml
env:
- name: EXTERNAL_URL
  value: "https://webhooks.yourdomain.com"  # Update this
```

3. **Resource limits** (adjust based on your expected load):
```yaml
resources:
  requests:
    memory: "128Mi"
    cpu: "100m"
  limits:
    memory: "512Mi"
    cpu: "500m"
```

4. **Replica count** (for high availability):
```yaml
spec:
  replicas: 3  # Adjust as needed
```

### Step 6: Deploy Operator

```bash
kubectl apply -f k8s/deployment.yaml
```

### Step 7: Verify Deployment

#### Check Pods

```bash
kubectl get pods -l app=webhook-operator
```

Expected output (all pods Running):
```
NAME                               READY   STATUS    RESTARTS   AGE
webhook-operator-xxxxx-yyyyy       1/1     Running   0          1m
webhook-operator-xxxxx-zzzzz       1/1     Running   0          1m
webhook-operator-xxxxx-wwwww       1/1     Running   0          1m
```

#### Check Logs

```bash
kubectl logs -l app=webhook-operator --tail=50
```

Look for:
```
INFO webhook_operator: Starting webhook operator
INFO webhook_operator: Configuration loaded: namespace=default, external_url=https://...
INFO webhook_operator: Kafka producer initialized
INFO webhook_operator: Starting WebhookHandler watcher for namespace: default
INFO webhook_operator: Loaded 0 existing handlers
INFO webhook_operator: Listening on 0.0.0.0:8080
```

#### Check Service

```bash
kubectl get svc webhook-operator
```

Expected output:
```
NAME               TYPE           CLUSTER-IP      EXTERNAL-IP      PORT(S)        AGE
webhook-operator   LoadBalancer   10.96.xxx.xxx   x.x.x.x          80:xxxxx/TCP   1m
```

Note the `EXTERNAL-IP` - this is where your webhooks will be accessible.

#### Test Health Endpoint

```bash
EXTERNAL_IP=$(kubectl get svc webhook-operator -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
curl http://${EXTERNAL_IP}/health
```

Expected response:
```json
{"status":"healthy"}
```

### Step 8: Configure External Access (Optional)

For production, you'll want a proper domain name:

#### Option A: Using Ingress

Create an Ingress resource:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: webhook-operator
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - webhooks.yourdomain.com
    secretName: webhook-tls
  rules:
  - host: webhooks.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: webhook-operator
            port:
              number: 80
```

Apply:
```bash
kubectl apply -f ingress.yaml
```

#### Option B: Update LoadBalancer with DNS

Point your DNS A record to the LoadBalancer IP:

```
webhooks.yourdomain.com.  IN  A  x.x.x.x
```

### Step 9: Test the Installation

#### Create a Test Handler

```bash
# Set environment variables
export API_KEY="your-api-signing-key"
export TOPIC="test-topic"
export WEBHOOK_URL="https://webhooks.yourdomain.com"

# Run the example script
chmod +x examples/create-handler.sh
./examples/create-handler.sh
```

Expected output:
```
✓ Handler created successfully

Response:
{
  "handler_id": "789e4567-e89b-12d3-a456-426614174001",
  "webhook_url": "https://webhooks.yourdomain.com/handler/789e4567-e89b-12d3-a456-426614174001"
}
```

#### Verify Handler in Kubernetes

```bash
kubectl get webhookhandlers
```

Expected output:
```
NAME                                      TOPIC         URL                                              READY   AGE
handler-789e4567-e89b-12d3-a456-426614   test-topic    https://webhooks.yourdomain.com/handler/789e...  true    1m
```

#### Send a Test Webhook

```bash
export HANDLER_URL="https://webhooks.yourdomain.com/handler/789e4567-e89b-12d3-a456-426614174001"
export PAYLOAD='{"event":"test","data":"hello world"}'

chmod +x examples/send-webhook.sh
./examples/send-webhook.sh
```

#### Verify in Kafka

Check that the message arrived in Kafka:

```bash
# Using kafka-console-consumer
kafka-console-consumer --bootstrap-server your-kafka:9092 \
  --topic test-topic \
  --from-beginning
```

You should see your webhook payload in Kafka.

## Kafka ACL Setup

The Kafka service account needs the following ACLs:

```bash
# Allow WRITE to all webhook topics
kafka-acls --bootstrap-server your-kafka:9092 \
  --add --allow-principal User:webhook-service-account \
  --operation Write \
  --topic '*'

# Allow DESCRIBE topics
kafka-acls --bootstrap-server your-kafka:9092 \
  --add --allow-principal User:webhook-service-account \
  --operation Describe \
  --topic '*'
```

For better security, limit to specific topic patterns:

```bash
kafka-acls --bootstrap-server your-kafka:9092 \
  --add --allow-principal User:webhook-service-account \
  --operation Write \
  --topic 'webhooks.*' \
  --resource-pattern-type prefixed
```

## Upgrading

To upgrade to a new version:

```bash
# Build new image
docker build -t your-registry.com/webhook-operator:v2.0.0 .
docker push your-registry.com/webhook-operator:v2.0.0

# Update deployment
kubectl set image deployment/webhook-operator \
  operator=your-registry.com/webhook-operator:v2.0.0

# Watch rollout
kubectl rollout status deployment/webhook-operator
```

## Uninstalling

To completely remove the operator:

```bash
# Delete deployment
kubectl delete -f k8s/deployment.yaml
kubectl delete -f k8s/rbac.yaml

# Delete configuration (WARNING: includes secrets)
kubectl delete -f k8s/config.yaml

# Delete CRD (WARNING: deletes all handlers)
kubectl delete -f k8s/crd.yaml
```

## Troubleshooting Installation

### Pods Not Starting

**Check events:**
```bash
kubectl describe pod webhook-operator-xxxxx-yyyyy
```

**Common causes:**
- Image pull errors (check image name and registry credentials)
- Missing ConfigMap/Secret
- Insufficient resources

### Cannot Connect to Kafka

**Check logs:**
```bash
kubectl logs webhook-operator-xxxxx-yyyyy | grep -i kafka
```

**Common causes:**
- Wrong bootstrap servers
- Invalid credentials
- Network policy blocking egress
- Firewall rules

**Test connectivity:**
```bash
kubectl run kafka-test --rm -it --image=confluentinc/cp-kafka:7.5.0 -- bash
# Inside pod:
kafka-broker-api-versions --bootstrap-server your-kafka:9092
```

### Handler Creation Fails

**Check operator logs:**
```bash
kubectl logs -l app=webhook-operator --tail=100
```

**Common causes:**
- Invalid signature
- RBAC permissions missing
- Namespace mismatch

### LoadBalancer Pending

If `EXTERNAL-IP` shows `<pending>`:

```bash
kubectl describe svc webhook-operator
```

**Possible issues:**
- Cloud provider doesn't support LoadBalancer
- Quota exceeded
- Region doesn't have available IPs

**Solution:**
Use NodePort or Ingress instead of LoadBalancer.

## Next Steps

After successful installation:

1. Set up monitoring (Prometheus/Grafana)
2. Configure log aggregation
3. Set up alerts for failures
4. Document your API key management process
5. Create runbooks for common operations

See [README.md](README.md) for usage documentation.