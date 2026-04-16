use crate::detectors::EcosystemTools;
use anyhow::{anyhow, Result};
use dirs::home_dir;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CacheTarget {
    pub id: String,
    pub ecosystem: String,
    pub label: String,
    pub path: PathBuf,
    pub exists: bool,
    pub size_bytes: u64,
    pub optional: bool,
}

pub fn collect_cache_targets(
    tool_inventory: &[EcosystemTools],
    include_venv: bool,
) -> Result<Vec<CacheTarget>> {
    let home = home_dir().ok_or_else(|| anyhow!("Could not resolve home directory"))?;
    let installed = installed_tool_set(tool_inventory);
    let mut out = Vec::new();

    let mut add = |id: &str, ecosystem: &str, label: &str, path: PathBuf, optional: bool| {
        out.push(CacheTarget {
            id: id.to_string(),
            ecosystem: ecosystem.to_string(),
            label: label.to_string(),
            exists: path.exists(),
            path,
            size_bytes: 0,
            optional,
        });
    };

    if installed.contains("npm") {
        add(
            "npm_cache",
            "JavaScript/Node",
            "npm cache",
            home.join(".npm"),
            false,
        );
    }
    if installed.contains("yarn") {
        add(
            "yarn_cache",
            "JavaScript/Node",
            "Yarn cache",
            home.join("Library/Caches/Yarn"),
            false,
        );
    }
    if installed.contains("pnpm") {
        add(
            "pnpm_store",
            "JavaScript/Node",
            "pnpm store",
            home.join("Library/pnpm/store"),
            false,
        );
    }
    if installed.contains("uv") {
        add(
            "uv_cache",
            "Python",
            "uv cache",
            home.join(".cache/uv"),
            false,
        );
    }
    if installed.contains("pip3") || installed.contains("python3") {
        add(
            "pip_cache",
            "Python",
            "pip cache",
            home.join("Library/Caches/pip"),
            false,
        );
    }
    if installed.contains("poetry") {
        add(
            "poetry_cache",
            "Python",
            "Poetry cache",
            home.join("Library/Caches/pypoetry"),
            false,
        );
    }
    if installed.contains("pipx") {
        add(
            "pipx_cache",
            "Python",
            "pipx cache",
            home.join(".cache/pipx"),
            false,
        );
    }
    if installed.contains("cargo") || installed.contains("rustup") {
        add(
            "cargo_registry",
            "Rust",
            "Cargo registry cache",
            home.join(".cargo/registry"),
            false,
        );
        add(
            "cargo_git",
            "Rust",
            "Cargo git cache",
            home.join(".cargo/git"),
            false,
        );
    }
    if installed.contains("rustup") {
        add(
            "rustup_downloads",
            "Rust",
            "Rustup download cache",
            home.join(".rustup/downloads"),
            false,
        );
        add(
            "rustup_tmp",
            "Rust",
            "Rustup tmp cache",
            home.join(".rustup/tmp"),
            false,
        );
    }
    if installed.contains("docker") {
        add(
            "docker_cache",
            "Container",
            "Docker cache dir",
            home.join("Library/Caches/com.docker.docker"),
            false,
        );
    }
    if installed.contains("wasp") {
        add(
            "wasp_cache",
            "Wasp",
            "Wasp cache",
            home.join(".wasp/cache"),
            false,
        );
    }
    if include_venv {
        add(
            "venv_dirs",
            "Python",
            "Project .venv folders (recursive)",
            home.join(""),
            true,
        );
    }

    Ok(out)
}

fn installed_tool_set(inventory: &[EcosystemTools]) -> HashSet<String> {
    let mut set = HashSet::new();
    for eco in inventory {
        for t in &eco.tools {
            if t.installed {
                set.insert(t.name.clone());
            }
        }
    }
    set
}

