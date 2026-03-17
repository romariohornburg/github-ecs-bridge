# github-webhook-ingester

A Rust-based service that receives GitHub webhooks and transforms them into the Elastic Common Schema (ECS) format, equivalent to what the native Elasticsearch Security + GitHub Enterprise integration would produce when consuming the GitHub Audit Log API — without requiring a GitHub Enterprise account.

Events are indexed into the `logs-github.audit-default` data stream, making them compatible with Elastic Security's detection rules, dashboards, and alerts that expect the `github.audit` dataset.

---

## How it Works

```
GitHub Org Webhook
       │
       ▼
POST /webhook
       │
  HMAC-SHA256 Validation (X-Hub-Signature-256)
       │
  Transformation → ECS (event.dataset: github.audit)
       │
       ├─── 200 OK ──► GitHub (immediate)
       │
  tokio::spawn
       │
       ▼
POST /_bulk ──► Elasticsearch
               logs-github.audit-default
```

The ACK to GitHub is returned immediately before writing to Elasticsearch, ensuring that GitHub's 10-second timeout is never hit. Failures in Elasticsearch are logged.

---

## Prerequisites

- Rust 1.82+
- Elasticsearch 8.x (or higher) accessible by the service
- Webhook configured in the GitHub organization or repository pointing to this service

---

## Configuration

Copy `.env.example` to `.env` and fill in the variables:

```bash
cp .env.example .env
```

| Variable | Required | Default | Description |
|---|---|---|---|
| `WEBHOOK_SECRET` | ✅ | — | Secret configured in the GitHub webhook |
| `ELASTICSEARCH_URL` | ✅ | — | ES base URL (e.g., `https://localhost:9200`) |
| `ELASTICSEARCH_API_KEY` | ✅ | — | ES API Key (the `encoded` field returned by `POST /_security/api_key`) |
| `ELASTICSEARCH_INDEX` | — | `logs-github.audit-default` | Target data stream or index |
| `LISTEN_ADDR` | — | `0.0.0.0:3001` | TCP bind address |
| `RUST_LOG` | — | `info` | Log level |

---

## Running

### Local

```bash
cargo run
```

### Docker

```bash
docker build -t github-webhook-ingester .
docker run --env-file .env -p 3001:3001 github-webhook-ingester
```

### Docker Compose

```bash
docker compose up -d
```

The service starts on port 3001. Configure `.env` with `WEBHOOK_SECRET`, `ELASTICSEARCH_URL`, and `ELASTICSEARCH_API_KEY` before starting.

---

## Endpoints

| Method | Path | Description |
|---|---|---|
| `POST` | `/webhook` | Receives GitHub webhooks (validated) |
| `GET` | `/health` | Health check — returns `ok` |
| `POST` | `/debug/webhook` | Local debug endpoint (NO signature validation) |

---

## Configuring the Webhook in GitHub

1. Go to **Settings → Webhooks → Add webhook** in the organization or repository.
2. **Payload URL:** Public URL of this service + `/webhook` (e.g., `https://your-domain.com/webhook`).
3. **Content type:** `application/json`
4. **Secret:** Same value as `WEBHOOK_SECRET`.
5. **SSL verification:** Enabled (recommended).
6. **Events:** Select desired events or "Send me everything".

For local development, use [smee.io](https://smee.io) or [ngrok](https://ngrok.com) to expose the service:

```bash
# With ngrok
ngrok http 3001

# With smee
npx smee-client --url https://smee.io/YOUR_CHANNEL --target http://localhost:3001/webhook
```

---

## Generated ECS Format

Each webhook is converted into a document with this structure:

```json
{
  "@timestamp": "2024-01-15T12:00:00.000Z",
  "event": {
    "action": "repo.create",
    "category": ["configuration"],
    "type": ["creation"],
    "kind": "event",
    "dataset": "github.audit",
    "created": "2024-01-15T12:00:00.000Z"
  },
  "user": {
    "name": "monalisa",
    "id": "12345"
  },
  "organization": {
    "name": "my-org",
    "id": "67890"
  },
  "github": {
    "action": "repo.create",
    "actor": "monalisa",
    "actor_id": 12345,
    "org": "my-org",
    "org_id": 67890,
    "repo": "my-org/new-repo",
    "repo_id": 111,
    "created_at": 1705316400000,
    "data": {
      "name": "new-repo",
      "private": false,
      "visibility": "public"
    }
  },
  "tags": ["github-webhook"],
  "data_stream": {
    "type": "logs",
    "dataset": "github.audit",
    "namespace": "default"
  }
}
```

> **Note:** Fields `source.ip`, `related.ip`, and `github.actor_ip` are **absent** in events originated via webhook. The GitHub Enterprise Audit Log API includes the actor's IP, but webhook payloads do not carry this information. This is the only structural difference compared to the pull-based integration.

---

## Security

- **HMAC-SHA256 Validation** on all requests to `/webhook` using the `X-Hub-Signature-256` header.
- Constant-time comparison (`hmac::Mac::verify_slice`) used to prevent timing side-channel attacks.
- Requests without a valid signature return `401 Unauthorized`.
- **Warning on /debug/webhook**: This endpoint does NOT validate signatures. It is intended for local development and testing only. If running in production, ensure this path is blocked by your reverse proxy (e.g., Nginx, Caddy) or firewall.
- It is recommended to run behind a reverse proxy with TLS termination.

---

## Stack

- **[Axum 0.8](https://github.com/tokio-rs/axum)** — HTTP server
- **[Tokio](https://tokio.rs)** — Asynchronous runtime
- **[reqwest 0.12](https://github.com/seanmonstar/reqwest)** — HTTP client for Elasticsearch
- **[hmac](https://github.com/RustCrypto/MACs) + [sha2](https://github.com/RustCrypto/hashes)** — Webhook signature validation
- **[serde_json](https://github.com/serde-rs/json)** — ECS serialization
- **[chrono](https://github.com/chronotope/chrono)** — ISO 8601 / epoch ms timestamps

---

## License

This project is licensed under the [GPL-3.0 License](LICENSE).
