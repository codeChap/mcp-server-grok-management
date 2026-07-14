# mcp-server-grok-management

Rust MCP server for **xAI Management API** — API keys + **billing**.

## Build / test

```bash
cargo build --release
cargo test
./tests/test-stdio.sh
```

Binary: `target/release/mcp-server-grok-management`

## Config

`~/.config/mcp-server-grok-management/config.toml` — `management_key` (+ optional `team_id`).

## Architecture (no god modules)

```
src/
  main.rs              # stdio entry
  config.rs            # TOML
  util.rs              # JSON field / cents / timestamps
  helpers.rs           # ok/err, require_confirm, with_team_fmt
  api/                 # HTTP client — one concern per file
    mod.rs             # transport + validate + team_id
    keys.rs            # key/model/endpoint endpoints
    billing.rs         # billing endpoints
    audit.rs           # audit endpoints
  params/              # JsonSchema param structs by domain
    keys.rs | billing.rs | audit.rs | common.rs
  format/              # pure Value → String (no I/O)
    keys.rs
    billing/           # money, contact, invoices, prepaid, usage, audit
  tools/               # MCP #[tool] — composed routers
    mod.rs             # GrokManagementServer + tool_router() = keys+billing+audit
    keys.rs            # router_keys
    billing.rs         # router_billing
    audit.rs           # router_audit
```

`tools/mod.rs` combines domain routers with `ToolRouter` `+` (rmcp):

```rust
Self::router_keys() + Self::router_billing() + Self::router_audit()
```

## Billing agent tips

- Prefer **`billing_overview`** for “how much am I spending?”
- Soft limit / top-up need **BillingWrite** + confirm gates
- Amounts are **USD cents**
