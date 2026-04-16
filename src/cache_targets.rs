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
    if installed.contains("go") {
        add(
            "go_build_cache",
            "Go",
            "Go build cache",
            home.join("Library/Caches/go-build"),
            false,
        );
    }
    if installed.contains("composer") {
        add(
            "composer_cache",
            "PHP",
            "Composer cache",
            home.join("Library/Caches/composer"),
            false,
        );
    }
    if installed.contains("dotnet") {
        add(
            "nuget_packages",
            ".NET",
            "NuGet global packages",
            home.join(".nuget/packages"),
            false,
        );
        add(
            "nuget_http_cache",
            ".NET",
            "NuGet HTTP cache",
            home.join(".local/share/NuGet/v3-cache"),
            false,
        );
        add(
            "nuget_plugins_cache",
            ".NET",
            "NuGet plugins cache",
            home.join(".local/share/NuGet/plugins-cache"),
            false,
        );
    }
    if installed.contains("gradle")
        || installed.contains("adb")
        || installed.contains("sdkmanager")
        || installed.contains("avdmanager")
    {
        add(
            "gradle_caches",
            "Android",
            "Gradle caches",
            home.join(".gradle/caches"),
            false,
        );
        add(
            "gradle_wrapper_dists",
            "Android",
            "Gradle wrapper dists",
            home.join(".gradle/wrapper/dists"),
            false,
        );
    }
    if installed.contains("swift") || installed.contains("swiftc") || installed.contains("xcodebuild") {
        add(
            "xcode_derived_data",
            "Swift/iOS",
            "Xcode DerivedData",
            home.join("Library/Developer/Xcode/DerivedData"),
            false,
        );
        add(
            "swiftpm_cache",
            "Swift/iOS",
            "SwiftPM cache",
            home.join("Library/Caches/org.swift.swiftpm"),
            false,
        );
    }
    if installed.contains("flutter") || installed.contains("dart") {
        add(
            "pub_cache",
            "Flutter/Dart",
            "Dart pub cache",
            home.join(".pub-cache"),
            false,
        );
    }
    if installed.contains("cabal") || installed.contains("stack") || installed.contains("ghc") {
        add(
            "cabal_cache",
            "Haskell",
            "Cabal cache",
            home.join(".cache/cabal"),
            false,
        );
        add(
            "stack_cache",
            "Haskell",
            "Stack cache",
            home.join(".stack/pantry"),
            false,
        );
    }
    if installed.contains("elixir") || installed.contains("mix") {
        add(
            "hex_cache",
            "Elixir/Erlang",
            "Hex package cache",
            home.join(".hex/packages"),
            false,
        );
    }
    if installed.contains("sbt") || installed.contains("scala") {
        add(
            "ivy_cache",
            "Scala",
            "Ivy cache",
            home.join(".ivy2/cache"),
            false,
        );
        add(
            "coursier_cache",
            "Scala",
            "Coursier cache",
            home.join(".cache/coursier"),
            false,
        );
        add(
            "sbt_boot_cache",
            "Scala",
            "sbt boot cache",
            home.join(".sbt/boot"),
            false,
        );
    }
    if installed.contains("pipenv") {
        add(
            "pipenv_cache",
            "ML/Data",
            "Pipenv cache",
            home.join("Library/Caches/pipenv"),
            false,
        );
    }
    if installed.contains("huggingface-cli") {
        add(
            "huggingface_hub_cache",
            "ML/Data",
            "Hugging Face hub cache",
            home.join(".cache/huggingface/hub"),
            false,
        );
    }
    if installed.contains("kubectl") {
        add(
            "kube_cache",
            "DevOps",
            "kubectl cache",
            home.join(".kube/cache"),
            false,
        );
    }
    if installed.contains("helm") {
        add(
            "helm_cache",
            "DevOps",
            "Helm cache",
            home.join("Library/Caches/helm"),
            false,
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

