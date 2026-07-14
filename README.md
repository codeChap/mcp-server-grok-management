# mcp-server-grok-management

MCP server for the **xAI Management API** (`management-api.x.ai`) — team **API keys**
and **billing**, over stdio. Pure Rust binary.

Auth uses a **management key** (not an inference API key) from
[console.x.ai](https://console.x.ai) → **Settings → Management Keys**.

For day-to-day spend questions, start with **`billing_overview`**.

## Tools (23)

### Billing (primary)

| Tool | Access | Notes |
|------|--------|--------|
| `billing_overview` | read | Dashboard: period spend, headroom, limits, prepaid, PMs, contact |
| `get_invoice_preview` | read | This month’s postpaid spend + product breakdown |
| `get_spending_limits` | read | Soft/hard limits in USD |
| `set_spending_limit` | write | Soft limit in **cents**; `confirm="YES-WRITE"` |
| `get_prepaid_balance` | read | Credit ledger |
| `top_up` | write | Charge default PM; `confirm="TOP-UP"` |
| `list_payment_methods` | read | Cards / Stripe Link (add cards in console) |
| `set_default_payment_method` | write | `confirm="YES-WRITE"` |
| `get_billing_info` | read | Name / email / address |
| `set_billing_info` | write | `confirm="YES-WRITE"` |
| `list_invoices` | read | Optional year/month/since/invoice_ids filters |
| `get_invoice` | read | One invoice + line items |
| `get_usage` | read | Analytics (usd sum, group_by, timezone) |

### API keys

`validate_key`, `list_api_keys`, `create_api_key`, `update_api_key`,
`delete_api_key` (`DELETE`), `rotate_api_key` (`ROTATE`), `check_propagation`,
`list_models`, `list_endpoints`

### Audit

`list_audit_events`

## Safety gates

| Tool | Required `confirm` |
|------|--------------------|
| `delete_api_key` | `"DELETE"` |
| `rotate_api_key` | `"ROTATE"` |
| `set_spending_limit` | `"YES-WRITE"` |
| `set_billing_info` | `"YES-WRITE"` |
| `set_default_payment_method` | `"YES-WRITE"` |
| `top_up` | `"TOP-UP"` (**charges card**) |

Money fields are **USD cents** (e.g. `10000` = $100.00).

Write billing tools need **BillingWrite** (or full access) on the management key.
`validate_key` reports whether BillingWrite is listed on the key’s ACLs.

## Configuration

`~/.config/mcp-server-grok-management/config.toml` (`chmod 600`):

```toml
management_key = "xai-..."   # console.x.ai → Settings → Management Keys
# team_id = "uuid"           # optional — auto from validate
# base_url = "https://management-api.x.ai"
```

## Build

```bash
cargo build --release   # target/release/mcp-server-grok-management
cargo test
./tests/test-stdio.sh
```

## Architecture

No god modules: **api/** (HTTP by domain), **format/** (pure text), **params/** (schemas),
**tools/** (MCP handlers composed with `router_keys + router_billing + router_audit`).
See `CLAUDE.md`.

## MCP clients

Already wired for **Grok** (`~/.grok/config.toml`) and **Claude Code**
(`claude mcp add -s user …`). Restart the client session if tools don’t appear.

## Scope notes

- Creating **management keys** themselves is console-only.
- Adding **payment methods** is console-only; this server lists them and can set default.
- Collections / Computer Use / Conversations / Files resources are not in the public Management REST surface (only appear as management-key ACL labels).

## Docs

- [Using Management API](https://docs.x.ai/developers/management-api-guide)
- [Billing reference](https://docs.x.ai/developers/rest-api-reference/management/billing)
