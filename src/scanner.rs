use crate::cache_targets::CacheTarget;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn scan_targets(targets: &mut [CacheTarget]) {
    scan_targets_with_progress(targets, |_, _, _| {});
}

pub fn scan_targets_with_progress<F>(targets: &mut [CacheTarget], mut on_progress: F)
where
    F: FnMut(usize, usize, &std::path::Path),
{
    let total = targets.len();
    for (i, target) in targets.iter_mut().enumerate() {
        let current = i + 1;
        on_progress(current, total, &target.path);

        if target.id == "venv_dirs" {
            let (exists, size) = scan_venv_dirs(&target.path);
            target.exists = exists;
            target.size_bytes = size;
            continue;
        }

        target.exists = target.path.exists();
        target.size_bytes = if target.exists {
            dir_size_bytes(&target.path)
        } else {
            0
        };
    }
}

pub fn dir_size_bytes(path: &std::path::Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

fn scan_venv_dirs(root: &std::path::Path) -> (bool, u64) {
    let mut found = false;
    let mut total = 0_u64;
    for entry in WalkDir::new(root).max_depth(6).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_dir() && entry.file_name() == ".venv" {
            found = true;
            total += dir_size_bytes(entry.path());
        }
    }
    (found, total)
}

pub fn find_venv_paths(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(root)
        .max_depth(6)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir() && e.file_name() == ".venv")
        .map(|e| e.path().to_path_buf())
        .collect()
}

#[derive(Debug, Clone)]
pub struct DeletionEntry {
    pub category: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub is_essential_warning: bool,
}

pub fn collect_deletion_entries(target: &CacheTarget) -> Vec<DeletionEntry> {
    let mut out = Vec::new();
    let roots: Vec<PathBuf> = if target.id == "venv_dirs" {
        find_venv_paths(&target.path)
    } else if target.path.exists() {
        vec![target.path.clone()]
    } else {
        vec![]
    };

    for root in roots {
        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            let path = entry.path().to_path_buf();
            let size_bytes = if entry.file_type().is_dir() {
                0
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            };
            out.push(DeletionEntry {
                category: target.id.clone(),
                path,
                size_bytes,
                is_essential_warning: is_essential_target(&target.id),
            });
        }
    }
    out
}

fn is_essential_target(target_id: &str) -> bool {
    matches!(
        target_id,
        "cargo_registry" | "cargo_git" | "rustup_downloads" | "rustup_tmp" | "venv_dirs"
    )
}

pub fn path_depth(path: &Path) -> usize {
    path.components().count()
}
