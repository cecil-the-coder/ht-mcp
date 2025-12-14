# HT-MCP Pre-built Binaries

This directory contains pre-built binaries for the ht-mcp server.

## Available Binaries

### Linux AMD64

- **File**: `ht-mcp-linux-amd64`
- **Platform**: Linux x86_64
- **Size**: ~8.1 MB
- **Type**: ELF 64-bit LSB pie executable
- **Checksum**: See `ht-mcp-linux-amd64.sha256`

## Features

This build includes:
- ✅ All 6 MCP tools (create_session, send_keys, take_snapshot, **take_screenshot**, execute_command, list_sessions, close_session)
- ✅ **Full-color screenshot support** with ANSI 256-color palette + RGB
- ✅ System font discovery (uses system monospace fonts)
- ✅ PNG image generation with base64 encoding
- ✅ WebSocket-based live terminal preview

## Verification

To verify the binary integrity:

```bash
sha256sum -c ht-mcp-linux-amd64.sha256
```

## Usage

```bash
# Make executable (if not already)
chmod +x ht-mcp-linux-amd64

# Run the server
./ht-mcp-linux-amd64
```

## Requirements

- Linux kernel 3.2.0+
- System fonts (DejaVu Sans Mono, Liberation Mono, Consolas, or similar monospace fonts)
- fontconfig library

## Build Information

- **Built with**: Docker multi-stage build
- **Rust version**: 1.85
- **Build type**: Release (optimized, stripped)
- **Base image**: Debian Bullseye Slim

## Screenshot Tool Usage

The new `ht_take_screenshot` tool captures full-color PNG screenshots:

```json
{
  "method": "tools/call",
  "params": {
    "name": "ht_take_screenshot",
    "arguments": {
      "sessionId": "your-session-id"
    }
  }
}
```

Returns a base64-encoded PNG image with full ANSI color support.
