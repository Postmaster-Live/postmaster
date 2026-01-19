#!/bin/bash
set -e

# Configuration
HANDLER_URL="${HANDLER_URL:-}"
SIGNATURE_KEY="${SIGNATURE_KEY:-}"
PAYLOAD="${PAYLOAD:-}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ -z "$HANDLER_URL" ]; then
    echo -e "${RED}Error: HANDLER_URL environment variable is required${NC}"
    echo "Usage: HANDLER_URL=https://webhooks.example.com/handler/xxx PAYLOAD='{...}' ./send-webhook.sh"
    exit 1
fi

if [ -z "$PAYLOAD" ]; then
    echo -e "${YELLOW}No PAYLOAD specified, using default example${NC}"
    PAYLOAD='{"event":"test.event","data":{"id":123,"status":"success"}}'
fi

echo -e "${YELLOW}Sending webhook...${NC}"
echo "URL: ${HANDLER_URL}"
echo "Payload: ${PAYLOAD}"

# If signature key is provided, sign the request
if [ -n "$SIGNATURE_KEY" ]; then
    TIMESTAMP=$(date +%s)
    MESSAGE="${TIMESTAMP}.${PAYLOAD}"
    SIGNATURE=$(echo -n "${MESSAGE}" | openssl dgst -sha256 -hmac "${SIGNATURE_KEY}" | awk '{print $2}')
    
    echo "Signature: sha256=${SIGNATURE}"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${HANDLER_URL}" \
      -H "Content-Type: application/json" \
      -H "X-Timestamp: ${TIMESTAMP}" \
      -H "X-Signature: sha256=${SIGNATURE}" \
      -d "${PAYLOAD}")
else
    echo -e "${YELLOW}No SIGNATURE_KEY provided, sending unsigned request${NC}"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${HANDLER_URL}" \
      -H "Content-Type: application/json" \
      -d "${PAYLOAD}")
fi

# Extract HTTP status code
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY_RESPONSE=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" -eq 200 ]; then
    echo -e "${GREEN}✓ Webhook sent successfully${NC}"
    echo ""
    echo "Response:"
    echo "$BODY_RESPONSE" | jq '.'
else
    echo -e "${RED}✗ Webhook failed (HTTP ${HTTP_CODE})${NC}"
    echo "$BODY_RESPONSE" | jq '.' || echo "$BODY_RESPONSE"
    exit 1
fi