#!/bin/bash
set -e

# Configuration
WEBHOOK_URL="${WEBHOOK_URL:-https://webhooks.example.com}"
API_KEY="${API_KEY:-}"
TOPIC="${TOPIC:-}"
SIGNATURE_KEY="${SIGNATURE_KEY:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

if [ -z "$API_KEY" ]; then
    echo -e "${RED}Error: API_KEY environment variable is required${NC}"
    echo "Usage: API_KEY=your-key TOPIC=your-topic ./create-handler.sh"
    exit 1
fi

if [ -z "$TOPIC" ]; then
    echo -e "${RED}Error: TOPIC environment variable is required${NC}"
    echo "Usage: API_KEY=your-key TOPIC=your-topic ./create-handler.sh"
    exit 1
fi

# Build request body
if [ -n "$SIGNATURE_KEY" ]; then
    BODY=$(cat <<EOF
{"topic":"${TOPIC}","signature_key":"${SIGNATURE_KEY}"}
EOF
)
else
    BODY=$(cat <<EOF
{"topic":"${TOPIC}"}
EOF
)
fi

# Generate timestamp
TIMESTAMP=$(date +%s)

# Generate signature
MESSAGE="${TIMESTAMP}.${BODY}"
SIGNATURE=$(echo -n "${MESSAGE}" | openssl dgst -sha256 -hmac "${API_KEY}" | awk '{print $2}')

echo -e "${YELLOW}Creating webhook handler...${NC}"
echo "Endpoint: ${WEBHOOK_URL}/config"
echo "Topic: ${TOPIC}"

# Make request
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${WEBHOOK_URL}/config" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Signature: sha256=${SIGNATURE}" \
  -d "${BODY}")

# Extract HTTP status code
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY_RESPONSE=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" -eq 200 ]; then
    echo -e "${GREEN}✓ Handler created successfully${NC}"
    echo ""
    echo "Response:"
    echo "$BODY_RESPONSE" | jq '.'
    echo ""
    
    # Extract webhook URL
    WEBHOOK_ENDPOINT=$(echo "$BODY_RESPONSE" | jq -r '.webhook_url')
    echo -e "${GREEN}Webhook URL: ${WEBHOOK_ENDPOINT}${NC}"
    
    # Save to file for easy reference
    HANDLER_ID=$(echo "$BODY_RESPONSE" | jq -r '.handler_id')
    echo "$BODY_RESPONSE" | jq '.' > "handler-${HANDLER_ID}.json"
    echo -e "${YELLOW}Configuration saved to: handler-${HANDLER_ID}.json${NC}"
else
    echo -e "${RED}✗ Failed to create handler (HTTP ${HTTP_CODE})${NC}"
    echo "$BODY_RESPONSE" | jq '.' || echo "$BODY_RESPONSE"
    exit 1
fi