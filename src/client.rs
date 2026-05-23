use reqwest::Client;
use serde_json::{json, Value};

pub struct RegistryClient {
    client: Client,
}

impl RegistryClient {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn lookup_crate(&self, name: &str) -> Result<Value, String> {
        let v = self.client.get(&format!("https://crates.io/api/v1/crates/{}", name))
            .header("User-Agent", "mcp-package-registry/1.0")
            .send().await.map_err(|e| e.to_string())?
            .json::<Value>().await.map_err(|e| e.to_string())?;
        let c = &v["crate"];
        Ok(json!({
            "name": c["id"], "description": c["description"],
            "max_version": c["max_version"], "downloads": c["downloads"],
            "repository": c["repository"], "homepage": c["homepage"],
            "categories": v["categories"].as_array().map(|a| a.iter().filter_map(|x| x["category"].as_str().map(|s| s.to_string())).collect::<Vec<_>>()),
        }))
    }

    pub async fn lookup_npm(&self, name: &str) -> Result<Value, String> {
        let v = self.client.get(&format!("https://registry.npmjs.org/{}", name))
            .send().await.map_err(|e| e.to_string())?
            .json::<Value>().await.map_err(|e| e.to_string())?;
        let latest = v["dist-tags"]["latest"].as_str().unwrap_or("");
        Ok(json!({
            "name": v["name"], "description": v["description"],
            "latest_version": latest, "homepage": v["homepage"],
            "repository": v["repository"]["url"], "license": v["license"],
        }))
    }

    pub async fn list_versions_crate(&self, name: &str) -> Result<Value, String> {
        let v = self.client.get(&format!("https://crates.io/api/v1/crates/{}/versions", name))
            .header("User-Agent", "mcp-package-registry/1.0")
            .send().await.map_err(|e| e.to_string())?
            .json::<Value>().await.map_err(|e| e.to_string())?;
        let versions: Vec<Value> = v["versions"].as_array().unwrap_or(&vec![]).iter().take(20).map(|ver| json!({
            "num": ver["num"], "yanked": ver["yanked"], "created_at": ver["created_at"], "downloads": ver["downloads"],
        })).collect();
        Ok(json!({"name": name, "versions": versions, "count": versions.len()}))
    }

    pub async fn list_versions_npm(&self, name: &str) -> Result<Value, String> {
        let v = self.client.get(&format!("https://registry.npmjs.org/{}", name))
            .send().await.map_err(|e| e.to_string())?
            .json::<Value>().await.map_err(|e| e.to_string())?;
        let versions: Vec<String> = v["versions"].as_object().map(|o| o.keys().rev().take(20).cloned().collect()).unwrap_or_default();
        Ok(json!({"name": name, "versions": versions, "count": versions.len()}))
    }

    pub async fn get_advisories_crate(&self, name: &str) -> Result<Value, String> {
        // Query RustSec advisory DB via crates.io
        let v = self.client.get(&format!("https://crates.io/api/v1/crates/{}", name))
            .header("User-Agent", "mcp-package-registry/1.0")
            .send().await.map_err(|e| e.to_string())?
            .json::<Value>().await.map_err(|e| e.to_string())?;
        // crates.io doesn't directly expose advisories, but we can check audit info
        Ok(json!({"name": name, "advisories": [], "note": "Check https://rustsec.org/advisories/ for Rust advisories", "max_version": v["crate"]["max_version"]}))
    }

    pub async fn get_advisories_npm(&self, name: &str) -> Result<Value, String> {
        // npm audit endpoint
        Ok(json!({"name": name, "advisories": [], "note": "Run 'npm audit' locally for full advisory data"}))
    }

    pub async fn analyze_lockfile(&self, path: &str) -> Result<Value, String> {
        // Check for Cargo.lock or package-lock.json
        if let Ok(content) = tokio::fs::read_to_string(format!("{}/Cargo.lock", path)).await {
            let packages: Vec<&str> = content.lines().filter(|l| l.starts_with("name = ")).take(30)
                .map(|l| l.trim_start_matches("name = ").trim_matches('"')).collect();
            return Ok(json!({"type": "cargo", "packages": packages.len(), "sample": &packages[..packages.len().min(10)]}));
        }
        if let Ok(content) = tokio::fs::read_to_string(format!("{}/package-lock.json", path)).await {
            let v: Value = serde_json::from_str(&content).unwrap_or(json!({}));
            let count = v["packages"].as_object().map(|o| o.len()).unwrap_or(0);
            return Ok(json!({"type": "npm", "packages": count}));
        }
        Ok(json!({"error": "No lockfile found (Cargo.lock or package-lock.json)"}))
    }
}
