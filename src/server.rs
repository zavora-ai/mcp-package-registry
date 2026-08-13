use adk_mcp_sdk::{HealthCheck, HealthStatus};
use crate::client::RegistryClient;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LookupPackageInput { pub name: String, /// "crates" or "npm" (default: auto-detect)
    pub registry: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListVersionsInput { pub name: String, pub registry: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetChangelogInput { pub name: String, pub registry: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAdvisoriesInput { pub name: String, pub registry: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CheckCompatibilityInput { pub name: String, pub from_version: String, pub to_version: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeLockfileInput { pub path: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProposeUpgradePlanInput { pub path: String }

fn detect_registry(name: &str, hint: Option<&str>) -> &'static str {
    if let Some(h) = hint { return if h.contains("npm") { "npm" } else { "crates" }; }
    if name.starts_with('@') || name.contains('/') { "npm" } else { "crates" }
}

#[derive(Clone)]
pub struct PackageRegistryServer {
    pub client: Arc<RegistryClient>,
}

#[tool_router]
impl PackageRegistryServer {
    #[tool(description = "Look up a package — get description, latest version, downloads, repository. Works with crates.io and npm.")]
    async fn lookup_package(&self, Parameters(i): Parameters<LookupPackageInput>) -> String {
        let reg = detect_registry(&i.name, i.registry.as_deref());
        let result = if reg == "npm" { self.client.lookup_npm(&i.name).await } else { self.client.lookup_crate(&i.name).await };
        match result {
            Ok(v) => serde_json::to_string_pretty(&json!({"registry": reg, "package": v})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List available versions of a package (most recent first).")]
    async fn list_versions(&self, Parameters(i): Parameters<ListVersionsInput>) -> String {
        let reg = detect_registry(&i.name, i.registry.as_deref());
        let result = if reg == "npm" { self.client.list_versions_npm(&i.name).await } else { self.client.list_versions_crate(&i.name).await };
        match result {
            Ok(v) => serde_json::to_string_pretty(&json!({"registry": reg, "data": v})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get changelog or release notes for a package version.")]
    async fn get_changelog(&self, Parameters(i): Parameters<GetChangelogInput>) -> String {
        let reg = detect_registry(&i.name, i.registry.as_deref());
        // Point to the package's repository CHANGELOG
        let pkg = if reg == "npm" { self.client.lookup_npm(&i.name).await } else { self.client.lookup_crate(&i.name).await };
        match pkg {
            Ok(v) => {
                let repo = v["repository"].as_str().unwrap_or("unknown");
                serde_json::to_string_pretty(&json!({"name": i.name, "registry": reg, "repository": repo, "changelog_url": format!("{}/blob/main/CHANGELOG.md", repo.trim_end_matches(".git"))})).unwrap()
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Check for known security advisories for a package.")]
    async fn get_advisories(&self, Parameters(i): Parameters<GetAdvisoriesInput>) -> String {
        let reg = detect_registry(&i.name, i.registry.as_deref());
        let result = if reg == "npm" { self.client.get_advisories_npm(&i.name).await } else { self.client.get_advisories_crate(&i.name).await };
        match result {
            Ok(v) => serde_json::to_string_pretty(&json!({"registry": reg, "data": v})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Check if upgrading between two versions is compatible (semver analysis).")]
    async fn check_compatibility(&self, Parameters(i): Parameters<CheckCompatibilityInput>) -> String {
        let from_parts: Vec<u64> = i.from_version.split('.').filter_map(|s| s.parse().ok()).collect();
        let to_parts: Vec<u64> = i.to_version.split('.').filter_map(|s| s.parse().ok()).collect();
        let breaking = from_parts.first() != to_parts.first() && from_parts.first() != Some(&0);
        let minor_bump = from_parts.get(1) != to_parts.get(1);
        serde_json::to_string_pretty(&json!({
            "name": i.name, "from": i.from_version, "to": i.to_version,
            "breaking_change": breaking, "minor_bump": minor_bump,
            "compatible": !breaking,
            "recommendation": if breaking { "Review breaking changes before upgrading" } else { "Safe to upgrade" },
        })).unwrap()
    }

    #[tool(description = "Analyze a lockfile (Cargo.lock or package-lock.json) — count packages, detect outdated.")]
    async fn analyze_lockfile(&self, Parameters(i): Parameters<AnalyzeLockfileInput>) -> String {
        match self.client.analyze_lockfile(&i.path).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Propose an upgrade plan for outdated dependencies in a project.")]
    async fn propose_upgrade_plan(&self, Parameters(i): Parameters<ProposeUpgradePlanInput>) -> String {
        // Read Cargo.toml and check each dep against latest
        let cargo_path = format!("{}/Cargo.toml", i.path);
        let content = match tokio::fs::read_to_string(&cargo_path).await {
            Ok(c) => c,
            Err(_) => return json!({"error": "No Cargo.toml found"}).to_string(),
        };
        let mut deps = Vec::new();
        let mut in_deps = false;
        for line in content.lines() {
            if line.starts_with("[dependencies]") { in_deps = true; continue; }
            if line.starts_with('[') && !line.contains("dependencies") { in_deps = false; continue; }
            if in_deps && line.contains('=') && !line.trim().starts_with('#') {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                let name = parts[0].trim();
                deps.push(name.to_string());
            }
        }
        let mut plan = Vec::new();
        for dep in deps.iter().take(10) {
            if let Ok(info) = self.client.lookup_crate(dep).await {
                plan.push(json!({"name": dep, "latest": info["max_version"]}));
            }
        }
        serde_json::to_string_pretty(&json!({"path": i.path, "dependencies_checked": plan.len(), "plan": plan})).unwrap()
    }
}

#[async_trait::async_trait]
impl HealthCheck for PackageRegistryServer {
    async fn check_health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            message: Some("operational".into()),
            latency_ms: Some(1),
        }
    }
}

adk_mcp_sdk::mcp_2026_server! {
    server: PackageRegistryServer,
    task_tools: ["analyze_lockfile"],
    approval_tools: [],
    cache_ttl_ms: 60_000,
}
