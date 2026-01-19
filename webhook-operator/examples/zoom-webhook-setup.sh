#!/bin/bash
set -e

# Zoom Webhook Handler Setup Example
# This demonstrates filtering and routing for Zoom meeting events

WEBHOOK_URL="${WEBHOOK_URL:-https://webhooks.example.com}"
API_KEY="${API_KEY:-}"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ -z "$API_KEY" ]; then
    echo "Error: API_KEY environment variable is required"
    echo "Usage: API_KEY=your-key ./zoom-webhook-setup.sh"
    exit 1
fi

echo -e "${YELLOW}Creating Zoom webhook handler with filtering and routing...${NC}"
echo ""

# Configuration:
# - Default topic: zoom.events.default
# - Filter out blocked account IDs
# - Filter to only allow specific event types
# - Route events to account-specific topics

BODY=$(cat <<'EOF'
{
  "topic": "zoom.events.default",
  "signature_key": "whsec_zoom_secret_key_12345",
  "filters": [
    {
      "path": "$.payload.account_id",
      "operator": "not_in",
      "value": ["blocked_acc_1", "blocked_acc_2", "test_account"]
    },
    {
      "path": "$.event",
      "operator": "in",
      "value": [
        "meeting.started",
        "meeting.ended",
        "meeting.participant_joined",
        "meeting.participant_left"
      ]
    }
  ],
  "routes": [
    {
      "path": "$.payload.account_id",
      "mapping": [
        {
          "value": "enterprise_acc_123",
          "topic": "zoom.enterprise.account-123"
        },
        {
          "value": "enterprise_acc_456",
          "topic": "zoom.enterprise.account-456"
        },
        {
          "value": "startup_acc_789",
          "topic": "zoom.startup.account-789"
        }
      ]
    }
  ]
}
EOF
)

# Generate timestamp
TIMESTAMP=$(date +%s)

# Generate signature
MESSAGE="${TIMESTAMP}.${BODY}"
SIGNATURE=$(echo -n "${MESSAGE}" | openssl dgst -sha256 -hmac "${API_KEY}" | awk '{print $2}')

echo "Endpoint: ${WEBHOOK_URL}/config"
echo ""

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
    echo -e "${GREEN}✓ Zoom webhook handler created successfully${NC}"
    echo ""
    echo "Response:"
    echo "$BODY_RESPONSE" | jq '.'
    echo ""
    
    WEBHOOK_ENDPOINT=$(echo "$BODY_RESPONSE" | jq -r '.webhook_url')
    HANDLER_ID=$(echo "$BODY_RESPONSE" | jq -r '.handler_id')
    
    echo -e "${GREEN}Webhook URL: ${WEBHOOK_ENDPOINT}${NC}"
    echo ""
    
    echo "Configuration Summary:"
    echo "  Default Topic: zoom.events.default"
    echo "  Blocked Accounts: blocked_acc_1, blocked_acc_2, test_account"
    echo "  Allowed Events: meeting.started, meeting.ended, meeting.participant_joined, meeting.participant_left"
    echo ""
    echo "  Routing Rules:"
    echo "    enterprise_acc_123 → zoom.enterprise.account-123"
    echo "    enterprise_acc_456 → zoom.enterprise.account-456"
    echo "    startup_acc_789 → zoom.startup.account-789"
    echo "    (others) → zoom.events.default"
    echo ""
    
    # Save configuration
    echo "$BODY_RESPONSE" | jq '.' > "zoom-handler-${HANDLER_ID}.json"
    echo -e "${YELLOW}Configuration saved to: zoom-handler-${HANDLER_ID}.json${NC}"
    
    echo ""
    echo "Next steps:"
    echo "  1. Configure this URL in your Zoom App settings"
    echo "  2. Test with: ./zoom-test-webhook.sh ${WEBHOOK_ENDPOINT}"
else
    echo "✗ Failed to create handler (HTTP ${HTTP_CODE})"
    echo "$BODY_RESPONSE" | jq '.' || echo "$BODY_RESPONSE"
    exit 1
fi