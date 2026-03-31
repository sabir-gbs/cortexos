//! Dependency graph validator for the CortexOS workspace.
//!
//! Validates that no crate has dependencies outside its allowed list
//! per spec 01, section 8.4.

use std::collections::HashMap;
use std::fs;

/// Canonical allowed dependencies per crate per spec 01 section 8.4.
fn allowed_deps() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m = HashMap::new();
    m.insert("cortex-core", vec![]);
    m.insert("cortex-config", vec!["cortex-core"]);
    m.insert("cortex-db", vec!["cortex-core", "cortex-config"]);
    m.insert(
        "cortex-observability",
        vec!["cortex-core", "cortex-config", "cortex-db"],
    );
    m.insert("cortex-sdk", vec!["cortex-core"]);
    m.insert(
        "cortex-auth",
        vec!["cortex-core", "cortex-config", "cortex-db"],
    );
    m.insert(
        "cortex-policy",
        vec!["cortex-core", "cortex-config", "cortex-db"],
    );
    m.insert(
        "cortex-settings",
        vec!["cortex-core", "cortex-config", "cortex-db"],
    );
    m.insert(
        "cortex-ai",
        vec!["cortex-core", "cortex-config", "cortex-db"],
    );
    m.insert(
        "cortex-files",
        vec!["cortex-core", "cortex-config", "cortex-db"],
    );
    m.insert(
        "cortex-search",
        vec!["cortex-core", "cortex-config", "cortex-db", "cortex-files"],
    );
    m.insert(
        "cortex-notify",
        vec!["cortex-core", "cortex-config", "cortex-db"],
    );
    m.insert(
        "cortex-wm",
        vec!["cortex-core", "cortex-config", "cortex-db"],
    );
    m.insert(
        "cortex-runtime",
        vec![
            "cortex-core",
            "cortex-config",
            "cortex-db",
            "cortex-policy",
            "cortex-sdk",
        ],
    );
    m.insert(
        "cortex-admin",
        vec![
            "cortex-core",
            "cortex-config",
            "cortex-db",
            "cortex-observability",
        ],
    );
    m.insert(
        "cortex-api",
        vec![
            "cortex-core",
            "cortex-config",
            "cortex-db",
            "cortex-auth",
            "cortex-policy",
            "cortex-settings",
            "cortex-ai",
            "cortex-files",
            "cortex-search",
            "cortex-notify",
            "cortex-observability",
            "cortex-runtime",
            "cortex-sdk",
            "cortex-admin",
            "cortex-wm",
        ],
    );
    m.insert(
        "cortex-server",
        vec![
            "cortex-api",
            "cortex-core",
            "cortex-config",
            "cortex-db",
            "cortex-auth",
            "cortex-notify",
            "cortex-wm",
        ],
    );
    m
}

fn main() {
    let repo_root = find_repo_root();
    let crates_dir = repo_root.join("crates");
    if !crates_dir.is_dir() {
        eprintln!("crates/ directory not found");
        std::process::exit(1);
    }

    let allowed = allowed_deps();
    let mut errors = 0u32;
    let mut checked = 0u32;

    for entry in fs::read_dir(&crates_dir).unwrap() {
        let entry = entry.unwrap();
        let manifest_path = entry.path().join("Cargo.toml");
        if !manifest_path.exists() {
            continue;
        }

        let crate_name = entry.file_name().to_string_lossy().into_owned();
        let content = fs::read_to_string(&manifest_path).unwrap();
        let manifest: toml::Value = content.parse::<toml::Value>().unwrap();
        let deps_table = manifest.get("dependencies");

        checked += 1;

        if let Some(deps) = deps_table {
            let deps = match deps {
                toml::Value::Table(table) => table
                    .keys()
                    .map(|k| k.to_string())
                    .filter(|k| k.starts_with("cortex-"))
                    .collect::<Vec<String>>(),
                _ => vec![],
            };

            let allowed_for_crate: Vec<&str> = allowed
                .get(crate_name.as_str())
                .cloned()
                .unwrap_or_default();

            for dep in &deps {
                if !allowed_for_crate.contains(&dep.as_str()) {
                    eprintln!(
                        "ERROR: {} has forbidden dependency on '{}'",
                        crate_name, dep
                    );
                    eprintln!("  Allowed: {:?}", allowed_for_crate);
                    errors += 1;
                }
            }
        }
    }

    if errors > 0 {
        eprintln!("\n{} dependency violations found", errors);
        std::process::exit(1);
    } else {
        println!("\n{} crates checked, 0 violations found", checked);
    }
}

fn find_repo_root() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.exists() {
            let content = fs::read_to_string(&candidate).unwrap();
            if content.contains("[workspace]") {
                return dir;
            }
        }
        dir = dir.parent().unwrap().to_path_buf();
    }
}
