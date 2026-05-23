# Package Registry MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-package-registry.svg)](https://crates.io/crates/mcp-package-registry)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

Let your AI agents check dependencies. This MCP server queries crates.io and npm to look up packages, check versions, find advisories, analyze lockfiles, and propose upgrade plans.

## What It Does

When your agent adds a dependency or reviews a PR that bumps versions, it can verify the package exists, check for security advisories, and confirm the upgrade is compatible — all without leaving the conversation.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-package-registry/main/docs/architecture.svg" alt="Package Registry MCP Architecture" width="700"/>
</p>

## Tools (7)

| Tool | What It Does | When To Use |
|------|-------------|-------------|
| `lookup_package` | Get package info (version, downloads, repo) | "What's the latest version of tokio?" |
| `list_versions` | Show available versions | "What versions of serde are available?" |
| `get_changelog` | Find the changelog URL | "Where's the changelog for this package?" |
| `get_advisories` | Check for security advisories | "Are there any vulnerabilities in this dep?" |
| `check_compatibility` | Semver analysis between versions | "Is upgrading from 1.38 to 1.43 safe?" |
| `analyze_lockfile` | Count and inspect locked dependencies | "How many deps does this project have?" |
| `propose_upgrade_plan` | Check each dep against latest version | "What needs upgrading?" |

## Verified Output

Tested against live crates.io and npm:

```
> lookup_package(name: "tokio")

{ "registry": "crates", "package": { "name": "tokio", "max_version": "1.52.3", "downloads": 689370719 } }

> lookup_package(name: "express", registry: "npm")

{ "registry": "npm", "package": { "name": "express", "latest_version": "5.2.1", "license": "MIT" } }

> list_versions(name: "serde")

{ "registry": "crates", "data": { "count": 20, "versions": [{ "num": "1.0.228" }, { "num": "1.0.227" }, ...] } }

> check_compatibility(name: "tokio", from_version: "1.38.0", to_version: "1.43.0")

{ "breaking_change": false, "compatible": true, "recommendation": "Safe to upgrade" }

> analyze_lockfile(path: "/my-project")

{ "type": "cargo", "packages": 30 }

> propose_upgrade_plan(path: "/my-project")

{ "dependencies_checked": 10, "plan": [
  { "name": "tokio", "latest": "1.52.3" },
  { "name": "serde", "latest": "1.0.228" },
  { "name": "reqwest", "latest": "0.12.15" }
]}
```

## Supported Registries

| Registry | Auto-detect | Packages |
|----------|-------------|----------|
| **crates.io** | `Cargo.toml` / plain name | Rust crates |
| **npm** | `@scope/name` or `registry: "npm"` | JavaScript/TypeScript |

## Installation

### 1. Build

```bash
git clone https://github.com/zavora-ai/mcp-package-registry
cd mcp-package-registry
cargo build --release
```

### 2. Add to your MCP client

No API key needed — crates.io and npm are public registries.

**Claude Desktop / Kiro / Cursor / Windsurf:**
```json
{
  "mcpServers": {
    "packages": {
      "command": "/path/to/mcp-package-registry"
    }
  }
}
```

### 3. Use it

Ask your agent:
- "What's the latest version of tokio?"
- "Is it safe to upgrade serde from 1.0.200 to 1.0.228?"
- "Check for security advisories on this dependency"
- "How many packages are in our lockfile?"
- "What dependencies need upgrading?"

## MCP Server Manifest

```toml
server_id = "mcp_package_registry"
display_name = "Package Registry MCP"
version = "1.0.0"
domain = "developer"
risk_level = "low"
writes_allowed = "none"
transports = ["stdio"]
governance_gates = []
```

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.
