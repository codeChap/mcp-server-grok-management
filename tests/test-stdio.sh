#!/usr/bin/env bash
# Smoke test for the stdio-based xAI Management MCP server.
# Requires ~/.config/mcp-server-grok-management/config.toml with a valid management_key.
# Run from the project root or from tests/.

set -euo pipefail

cd "$(dirname "$0")/.." || exit 1

echo "Testing mcp-server-grok-management over stdio"
echo "(reads key from ~/.config/mcp-server-grok-management/config.toml)"
echo ""

# Pipe a sequence of JSON-RPC frames to one server process:
#  1. initialize / initialized handshake
#  2. tools/list
#  3. validate_key
#  4. list_models (read-only, cheaper than listing keys)
cargo run --quiet <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"validate_key","arguments":{}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"list_models","arguments":{}}}
EOF
