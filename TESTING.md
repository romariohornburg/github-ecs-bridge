# Local Testing Guide

## Prerequisites

- Rust 1.82+ (`rustup update stable`)
- `curl` and `jq` installed

---

## 1. Starting the Service

The service requires three mandatory environment variables. For local testing, Elasticsearch values can be dummy values — ES send failures are logged and do not affect the debug endpoint.

```bash
cd github-webhook-ingester

WEBHOOK_SECRET=my-secret \
ELASTICSEARCH_URL=http://localhost:9200 \
ELASTICSEARCH_API_KEY=dummy \
cargo run
```

Expected output:

```
INFO github_webhook_ingester: github-webhook-ingester listening on 0.0.0.0:3001
```

---

## 2. Available Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/health` | Health check |
| `POST` | `/webhook` | Real webhook (validates HMAC, sends to ES) |
| `POST` | `/debug/webhook` | Local debug (NO HMAC, returns ECS as JSON) |

---

## 3. Health Check

```bash
curl -s http://localhost:3001/health
```

Response: `ok`

---

## 4. Debug Endpoint

`POST /debug/webhook` accepts the same headers and payload as a real webhook, but:
- **Does NOT** validate the HMAC signature
- **Does NOT** send anything to Elasticsearch
- **Returns** the generated ECS document as JSON

### Response Format

```json
{
  "event_type": "push",
  "ecs": {
    "@timestamp": "...",
    "event": { "action": "git.push", "category": ["source"], "..." },
    "github": { "actor": "joao", "repo": "my-org/repo", "..." },
    "..."
  }
}
```

---

## 5. Examples by Event Type

### push

```bash
curl -s -X POST http://localhost:3001/debug/webhook \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: push" \
  -d '{
    "ref": "refs/heads/main",
    "before": "abc123",
    "after": "def456",
    "forced": false,
    "head_commit": {"id": "def456"},
    "commits": [{"id": "def456", "message": "fix: fix bug"}],
    "repository": {"id": 1, "full_name": "my-org/repo", "private": false},
    "organization": {"login": "my-org", "id": 99},
    "sender": {"login": "joao", "id": 42}
  }' | jq .
```

---

### pull_request — opened

```bash
curl -s -X POST http://localhost:3001/debug/webhook \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: pull_request" \
  -d '{
    "action": "opened",
    "pull_request": {
      "number": 7,
      "node_id": "PR_kwDOA",
      "title": "feat: new feature",
      "html_url": "https://github.com/my-org/repo/pull/7",
      "merged": false,
      "state": "open",
      "base": {"ref": "main"},
      "head": {"ref": "feature/new"}
    },
    "repository": {"id": 1, "full_name": "my-org/repo"},
    "organization": {"login": "my-org", "id": 99},
    "sender": {"login": "joao", "id": 42}
  }' | jq .
```

---

### repository — created

```bash
curl -s -X POST http://localhost:3001/debug/webhook \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: repository" \
  -d '{
    "action": "created",
    "repository": {
      "id": 999,
      "full_name": "my-org/new-repo",
      "private": false,
      "visibility": "public",
      "html_url": "https://github.com/my-org/new-repo"
    },
    "organization": {"login": "my-org", "id": 99},
    "sender": {"login": "joao", "id": 42}
  }' | jq .
```

---

## 6. Testing the Real Webhook with HMAC

To test `/webhook` (with signature validation), you need to calculate the `HMAC-SHA256` of the body using the same `WEBHOOK_SECRET` configured.

### Helper Script

Create a file named `test-webhook.sh`:

```bash
#!/usr/bin/env bash
# Usage: ./test-webhook.sh <event-type> <json-file>

SECRET="${WEBHOOK_SECRET:-my-secret}"
EVENT="$1"
BODY=$(cat "$2")

SIG=$(echo -n "$BODY" | openssl dgst -sha256 -hmac "$SECRET" | awk '{print $2}')

echo "→ Sending event: $EVENT"
echo "→ Signature: sha256=$SIG"

curl -s -o /dev/null -w "HTTP %{http_code}\n" \
  -X POST http://localhost:3001/webhook \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: $EVENT" \
  -H "X-Hub-Signature-256: sha256=$SIG" \
  -H "X-GitHub-Delivery: $(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid)" \
  -d "$BODY"
```

```bash
chmod +x test-webhook.sh
```

### Usage

```bash
# Save a payload to a file
cat > /tmp/push.json << 'EOF'
{
  "ref": "refs/heads/main",
  "head_commit": {"id": "abc123"},
  "repository": {"id": 1, "full_name": "my-org/repo"},
  "organization": {"login": "my-org", "id": 99},
  "sender": {"login": "joao", "id": 42}
}
EOF

./test-webhook.sh push /tmp/push.json
```

Expected response: `HTTP 200`
