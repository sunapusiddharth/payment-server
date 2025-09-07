#!/bin/bash
# scripts/rollback.sh

set -e

echo "🚀 Starting rollback to previous version..."

# Get current and previous versions
CURRENT_VERSION=$(kubectl get deployment payment-system -o jsonpath='{.metadata.annotations.version}')
PREVIOUS_VERSION=$(kubectl get configmap payment-versions -o jsonpath='{.data.previous_version}')

if [ -z "$PREVIOUS_VERSION" ]; then
  echo "❌ No previous version found!"
  exit 1
fi

echo "Current version: $CURRENT_VERSION"
echo "Rolling back to: $PREVIOUS_VERSION"

# Update deployment
kubectl set image deployment/payment-system payment-system=payment-system:$PREVIOUS_VERSION
kubectl rollout status deployment/payment-system --timeout=300s

# Update version annotation
kubectl annotate deployment payment-system version=$PREVIOUS_VERSION --overwrite

# Verify
kubectl get pods -l app=payment-system

echo "✅ Rollback completed successfully!"

# Notify team
curl -X POST -H 'Content-type: application/json' \
  --data '{"text":"🚨 Payment System Rolled Back to '$PREVIOUS_VERSION'"}' \
  $SLACK_WEBHOOK_URL