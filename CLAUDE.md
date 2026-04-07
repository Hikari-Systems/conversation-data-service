# CLAUDE.md — conversation-data-service-rs

Guidance for AI assistants working on this codebase.

---

## What this service does

Stores and retrieves LLM conversation threads and messages for replay and history functionality. Exposes a REST API over PostgreSQL. No S3, no file I/O, no external services beyond the database. This is a Rust/actix-web rewrite of the original TypeScript/Express/Knex service and must remain a drop-in replacement.

---

## Codebase map

```
src/
  main.rs            — startup: config load, pool build, migrations, server bind
  config.rs          — AppConfig and sub-structs; serde_json loader + env var override walker
  db.rs              — PgPool builder (SSL mode, pool sizing)
  models/
    thread.rs        — Thread struct (FromRow + Serialize), DB query functions
    message.rs       — Message struct (FromRow + Serialize), DB query functions
  routes/
    mod.rs           — registers all routes on ServiceConfig; specific routes before wildcards
    thread.rs        — GET /api/thread/:id, GET /api/thread/byUserId/:id, POST /api/thread
    message.rs       — GET /api/message/byThreadId/:id, GET .../senderIdsByThreadId/:id, POST /api/message
static/
  index.html         — root page (embedded via include_str! at compile time)
migrations/
  *.sql              — sqlx forward-only migrations, run automatically on startup
config.json          — default config values baked into the image
```

---

## Critical implementation decisions

### Config loading (`config.rs`)
**Do not use the `config` crate.** It normalises JSON keys to lowercase, which breaks `camelCase` fields like `caCertFile`. The custom loader reads `config.json` with `serde_json::Value`, applies env var overrides by walking the key tree with exact-case segment matching, then deserialises in one pass.

Config priority (lowest → highest):
1. `config.json` in the working directory (or `CONFIG_PATH` env var)
2. Env vars with `__` separator, exact camelCase path segments (e.g. `db__host=postgres`)

### Route registration (`routes/mod.rs`)
Routes are registered directly on `web::ServiceConfig`, **never inside `web::scope("")`**. An empty scope matches all paths and swallows 404s.

Route order matters: `GET /api/thread/byUserId/{userId}` must be registered **before** `GET /api/thread/{threadId}`. If the wildcard comes first, actix-web routes `byUserId` as a thread ID and the specific handler is never reached.

### AppState (`main.rs`)
`AppState` is a **concrete struct** with a `PgPool` field — not a trait object. `web::Data<AppState>` extraction in handlers is straightforward. All handlers take `state: web::Data<AppState>` as their only shared-state parameter.

### sqlx queries (`models/`)
Use **runtime queries** (`query_as::<_, Row>(sql).bind(value)`) not compile-time macros (`query_as!`). The macro requires `DATABASE_URL` at compile time, which breaks Docker builds without a live database.

### JSONB field (`models/message.rs`)
`tool_calls` is a nullable `JSONB` column stored as `Option<serde_json::Value>` in the struct. The `#[sqlx(json)]` attribute on the field instructs sqlx's `FromRow` derive to decode the raw JSON bytes directly into `serde_json::Value`. For INSERT binds, wrap with `input.tool_calls.as_ref().map(sqlx::types::Json)` to produce `Option<Json<&Value>>`.

### PostgreSQL array query
The `get_all_by_user_id` query uses:
```sql
WHERE visible_to_user_ids IS NULL OR $1 = ANY(visible_to_user_ids)
```
`$1` is bound as a `&str` (Postgres `text`), which is compatible with `varchar(100)[]` in PostgreSQL's implicit cast rules.

### Static file serving
`static/index.html` is embedded via `include_str!("../static/index.html")` and served from a handler. Do not use `actix-files` — the relative path `./static` resolves relative to the process CWD, which is unreliable inside Docker.

### TLS / OpenSSL
sqlx is configured with `runtime-tokio-rustls`. This eliminates `openssl`/`pkg-config` from the build entirely. The runtime image (`debian:bookworm-slim`) provides only `libc`, `libm`, `libgcc_s` — all that is needed.

### 50 MB JSON limit
`POST /api/message` accepts large tool outputs (LLM tool result content). The `web::JsonConfig::default().limit(50 * 1024 * 1024)` is registered globally on the App — it applies to all JSON routes but only matters for message POST in practice.

---

## Docker build notes

Two-stage build:
1. **`builder`** (`rust:1-bookworm`) — stub `src/main.rs` (`fn main() {}`) is compiled first with `cargo build --release --locked` to cache all dependency compilation in a layer. The stub artifacts are deleted, real source is overlaid, and `cargo build --release --locked` runs again to compile only the service code.
2. **`runtime`** (`debian:bookworm-slim`) — copies binary, migrations, static, and config.json. Runs as unprivileged `appuser` (UID 1000).

No ImageMagick, no native deps beyond `ca-certificates`. Cold first build takes ~25–35 min; subsequent builds with the dep-cache layer hit are ~2 min.

**Do not use alpine/musl.** Proc-macro crates require the dynamic linker at build time.

**cargo-chef is not worth it** for this single-crate service. The stub `main.rs` pattern achieves the same caching.

---

## Adding new endpoints

1. Add the DB query function to the relevant `models/` file.
2. Add the handler function to the relevant `routes/` file.
3. Register the route in `routes/mod.rs` — remember to place specific paths before wildcard paths.

## Common gotchas

- Config key names are **case-sensitive**. `db__host` not `db__Host`.
- The `role` column was dropped in migration `20260215155700_tools`. The `Message` struct does not have a `role` field. Do not add one without a new migration.
- `unrouted` model functions (`get`, `get_all`, `delete` on message; `get_all` on thread) are marked `#[allow(dead_code)]` — they are intentionally present for future endpoints.
- Database migrations run automatically on startup. The `_sqlx_migrations` table is idempotent — safe to run against an already-migrated database.
