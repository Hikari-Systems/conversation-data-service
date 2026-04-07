# conversation-data-service-rs

A Rust/actix-web microservice for storing and retrieving LLM conversation threads and messages. Drop-in replacement for the original TypeScript/Express service; all API endpoints, JSON config keys, and response shapes are identical.

---

## Features

- Thread and message CRUD backed by PostgreSQL
- `visibleToUserIds` array filtering for per-user thread visibility
- `toolCalls` (JSONB) and `toolResultCallId` for LLM tool call tracking
- Automatic schema migrations on startup
- Config layering: baked defaults → env vars
- 50 MB JSON body limit on message POST (supports large tool outputs)

---

## API Endpoints

### `GET /healthcheck`
Returns `200 OK` with body `OK`.

```bash
curl http://localhost:3001/healthcheck
```

---

### `GET /api/thread/:threadId`
Returns a single thread by ID. Returns `204 No Content` if not found.

```bash
curl http://localhost:3001/api/thread/550e8400-e29b-41d4-a716-446655440000
```

**Response `200 OK`**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "My conversation",
  "visibleToUserIds": ["user-abc", "user-def"],
  "botId": "gpt-4o",
  "createdAt": "2026-04-07T12:00:00Z",
  "updatedAt": "2026-04-07T12:00:00Z"
}
```

---

### `GET /api/thread/byUserId/:userId`
Returns all threads visible to the given user. A thread is visible if `visibleToUserIds` is `null` (public) or contains the user ID.

```bash
curl http://localhost:3001/api/thread/byUserId/user-abc
```

**Response `200 OK`** — array of thread objects, ordered by `createdAt` descending.

---

### `POST /api/thread`
Creates a new thread.

```bash
curl -X POST http://localhost:3001/api/thread \
  -H 'Content-Type: application/json' \
  -d '{
    "title": "My conversation",
    "visibleToUserIds": ["user-abc"],
    "botId": "gpt-4o"
  }'
```

**Body**
| Field | Type | Required | Description |
|---|---|---|---|
| `title` | string | yes | Thread title |
| `visibleToUserIds` | string[] | no | User IDs that can see this thread; `null` = public |
| `botId` | string | no | Associated bot/model identifier |

**Response `201 Created`** — created thread object.

---

### `GET /api/message/byThreadId/:threadId`
Returns all messages in a thread, ordered by `createdAt` ascending.

```bash
curl http://localhost:3001/api/message/byThreadId/550e8400-e29b-41d4-a716-446655440000
```

**Response `200 OK`**
```json
[
  {
    "id": "661f9511-f3ac-52e5-b827-557766551111",
    "threadId": "550e8400-e29b-41d4-a716-446655440000",
    "senderId": "user-abc",
    "content": "Hello, can you help me?",
    "toolCalls": null,
    "toolResultCallId": null,
    "createdAt": "2026-04-07T12:00:01Z",
    "updatedAt": "2026-04-07T12:00:01Z"
  }
]
```

---

### `GET /api/message/senderIdsByThreadId/:threadId`
Returns the distinct sender IDs of all messages in a thread.

```bash
curl http://localhost:3001/api/message/senderIdsByThreadId/550e8400-e29b-41d4-a716-446655440000
```

**Response `200 OK`**
```json
["user-abc", "claude-3-5-sonnet"]
```

---

### `POST /api/message`
Creates a new message. Accepts up to 50 MB of JSON (for large tool outputs).

```bash
curl -X POST http://localhost:3001/api/message \
  -H 'Content-Type: application/json' \
  -d '{
    "threadId": "550e8400-e29b-41d4-a716-446655440000",
    "senderId": "user-abc",
    "content": "Hello, can you help me?"
  }'
```

With tool calls:

```bash
curl -X POST http://localhost:3001/api/message \
  -H 'Content-Type: application/json' \
  -d '{
    "threadId": "550e8400-e29b-41d4-a716-446655440000",
    "senderId": "claude-3-5-sonnet",
    "content": "",
    "toolCalls": [
      { "id": "toolu_01Abc", "name": "get_weather", "arguments": "{\"city\":\"Tokyo\"}" }
    ]
  }'
```

With a tool result:

```bash
curl -X POST http://localhost:3001/api/message \
  -H 'Content-Type: application/json' \
  -d '{
    "threadId": "550e8400-e29b-41d4-a716-446655440000",
    "senderId": "user-abc",
    "content": "{\"temperature\": 22, \"condition\": \"sunny\"}",
    "toolResultCallId": "toolu_01Abc"
  }'
```

**Body**
| Field | Type | Required | Description |
|---|---|---|---|
| `threadId` | UUID | yes | Thread this message belongs to |
| `senderId` | string | yes | User UUID or bot identifier |
| `content` | string | yes | Message body (markdown or JSON for tool results) |
| `toolCalls` | object[] | no | `[{id, name, arguments}]` — LLM tool call requests |
| `toolResultCallId` | string | no | Links this message to a prior tool call by its `id` |

**Response `201 Created`** — created message object.

---

## Configuration

Config is loaded from `config.json` (or `CONFIG_PATH` env var), then overridden by environment variables using `__` as the path separator with **exact camelCase key names**.

### `config.json` reference

```json
{
  "server": {
    "port": 3000
  },
  "log": {
    "level": "debug"
  },
  "db": {
    "database": "conversation-data-service",
    "host": "conversation-data-service-db",
    "port": "5432",
    "username": "conversation-data-service",
    "password": "conversation-data-service",
    "ssl": {
      "enabled": false,
      "verify": true,
      "caCertFile": ""
    },
    "minpool": 0,
    "maxpool": 10
  }
}
```

### Environment variable examples

```bash
db__host=postgres
db__password=secret
db__ssl__enabled=true
db__ssl__caCertFile=/certs/ca.pem
log__level=info
server__port=8080
```

---

## Docker / Deployment

### Build and run locally

```bash
docker compose up --build
```

Service listens on `http://localhost:3001` (mapped from container port 3000).

### docker-compose.yml overview

```yaml
services:
  app:
    build: .
    ports:
      - "3001:3000"
    environment:
      - db__host=conversation-data-service-db
      - db__password=conversation-data-service
    depends_on:
      conversation-data-service-db:
        condition: service_healthy

  conversation-data-service-db:
    image: postgres:18
```

### Multi-stage Dockerfile

1. **`builder`** (`rust:1-bookworm`) — stub `main.rs` compiled first to cache dependencies; real source compiled second. No native deps required beyond the Rust toolchain.
2. **`runtime`** (`debian:bookworm-slim`) — binary, migrations, static file, and config only. Runs as unprivileged `appuser`.

Cold build time: ~25–35 min. Subsequent builds with dep-cache layer: ~2 min.

---

## Development

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+

### Run locally

```bash
# start postgres (or use docker compose up conversation-data-service-db)
db__host=localhost cargo run
```

### Compile check (no database needed)

```bash
cargo check
```

### End-to-end test sequence

```bash
BASE=http://localhost:3001

# Create a thread
THREAD=$(curl -s -X POST $BASE/api/thread \
  -H 'Content-Type: application/json' \
  -d '{"title":"Test thread","visibleToUserIds":["user-abc"]}')
echo $THREAD
THREAD_ID=$(echo $THREAD | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)

# Fetch it back
curl -s $BASE/api/thread/$THREAD_ID | python3 -m json.tool

# Check visibility
curl -s $BASE/api/thread/byUserId/user-abc | python3 -m json.tool

# Post a user message
curl -s -X POST $BASE/api/message \
  -H 'Content-Type: application/json' \
  -d "{\"threadId\":\"$THREAD_ID\",\"senderId\":\"user-abc\",\"content\":\"Hello!\"}" \
  | python3 -m json.tool

# Post an assistant message with a tool call
curl -s -X POST $BASE/api/message \
  -H 'Content-Type: application/json' \
  -d "{\"threadId\":\"$THREAD_ID\",\"senderId\":\"claude-3-5-sonnet\",\"content\":\"\",\"toolCalls\":[{\"id\":\"toolu_01Abc\",\"name\":\"get_weather\",\"arguments\":\"{}\"}]}" \
  | python3 -m json.tool

# Fetch all messages
curl -s $BASE/api/message/byThreadId/$THREAD_ID | python3 -m json.tool

# Fetch sender IDs
curl -s $BASE/api/message/senderIdsByThreadId/$THREAD_ID | python3 -m json.tool
```
