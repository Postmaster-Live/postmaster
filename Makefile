.PHONY: help build test docker-build docker-push deploy clean install-crd delete-crd

# Variables
IMAGE_NAME ?= webhook-operator
IMAGE_TAG ?= latest
REGISTRY ?= your-registry
FULL_IMAGE = $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)
NAMESPACE ?= default

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the Rust binary locally
	cargo build --release

test: ## Run tests
	cargo test

docker-build: ## Build Docker image
	docker build -t $(FULL_IMAGE) .

docker-push: docker-build ## Build and push Docker image
	docker push $(FULL_IMAGE)

install-crd: ## Install the CRD
	kubectl apply -f k8s/crd.yaml

delete-crd: ## Delete the CRD (WARNING: deletes all handlers)
	kubectl delete -f k8s/crd.yaml

install-config: ## Install ConfigMap and Secrets
	@echo "WARNING: Update k8s/config.yaml with your Kafka credentials first!"
	kubectl apply -f k8s/config.yaml

deploy: install-crd ## Deploy the operator
	kubectl apply -f k8s/rbac.yaml
	kubectl apply -f k8s/deployment.yaml

undeploy: ## Remove the operator deployment
	kubectl delete -f k8s/deployment.yaml
	kubectl delete -f k8s/rbac.yaml

full-install: install-config deploy ## Full installation (config + operator)

clean: ## Clean build artifacts
	cargo clean
	rm -rf target/

logs: ## View operator logs
	kubectl logs -l app=webhook-operator -f --namespace=$(NAMESPACE)

status: ## Check operator status
	@echo "=== Pods ==="
	kubectl get pods -l app=webhook-operator --namespace=$(NAMESPACE)
	@echo ""
	@echo "=== Service ==="
	kubectl get svc webhook-operator --namespace=$(NAMESPACE)
	@echo ""
	@echo "=== Webhook Handlers ==="
	kubectl get webhookhandlers --namespace=$(NAMESPACE)

generate-api-key: ## Generate a new API signing key
	@openssl rand -hex 32

example-create-handler: ## Show example curl command to create handler
	@echo "# Generate timestamp and signature"
	@echo 'TIMESTAMP=$$(date +%s)'
	@echo 'BODY='"'"'{"topic":"my-topic","signature_key":"whsec_my_secret"}'"'"''
	@echo 'SIGNATURE=$$(echo -n "$${TIMESTAMP}.$${BODY}" | openssl dgst -sha256 -hmac "YOUR_API_KEY" | awk '"'"'{print $$2}'"'"')'
	@echo ""
	@echo "# Create handler"
	@echo 'curl -X POST https://webhooks.example.com/config \'
	@echo '  -H "Content-Type: application/json" \'
	@echo '  -H "X-Timestamp: $${TIMESTAMP}" \'
	@echo '  -H "X-Signature: sha256=$${SIGNATURE}" \'
	@echo '  -d "$${BODY}"'

fmt: ## Format code
	cargo fmt

lint: ## Run clippy
	cargo clippy -- -D warnings

dev-setup: ## Setup development environment
	@echo "Installing Rust components..."
	rustup component add rustfmt clippy
	@echo ""
	@echo "Installing system dependencies (Ubuntu/Debian)..."
	@echo "Run: sudo apt-get install cmake libssl-dev libsasl2-dev pkg-config"