use crate::cache_targets::CacheTarget;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

pub fn scan_targets(targets: &mut [CacheTarget]) {
    scan_targets_with_progress(targets, |_, _, _| {});
}

pub fn scan_targets_with_progress<F>(targets: &mut [CacheTarget], mut on_progress: F)
where
    F: FnMut(usize, usize, &std::path::Path),
{
    let total = targets.len();
    if targets.is_empty() {
        return;
    }
    on_progress(0, total, &targets[0].path);

    let worker_count = recommended_scan_workers(targets.len());
    if worker_count <= 1 {
        for (idx, target) in targets.iter_mut().enumerate() {
            let (exists, size_bytes) = scan_target(&target.id, &target.path);
            target.exists = exists;
            target.size_bytes = size_bytes;
            on_progress(idx + 1, total, &target.path);
        }
        return;
    }

    let jobs: Vec<(usize, String, PathBuf)> = targets
        .iter()
        .enumerate()
        .map(|(idx, target)| (idx, target.id.clone(), target.path.clone()))
        .collect();
    let job_queue = Arc::new(Mutex::new(jobs));
    let (tx, rx) = mpsc::channel::<(usize, bool, u64)>();

    let mut handles = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let tx = tx.clone();
        let job_queue = Arc::clone(&job_queue);
        handles.push(thread::spawn(move || loop {
            let next_job = {
                let mut locked = match job_queue.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };
                locked.pop()
            };

            let Some((idx, id, path)) = next_job else {
                break;
            };

            let (exists, size_bytes) = scan_target(&id, &path);
            if tx.send((idx, exists, size_bytes)).is_err() {
                break;
            }
        }));
    }
    drop(tx);

    let mut completed = 0usize;
    for (idx, exists, size_bytes) in rx {
        if let Some(target) = targets.get_mut(idx) {
            target.exists = exists;
            target.size_bytes = size_bytes;
            completed += 1;
            on_progress(completed, total, &target.path);
        }
    }

    for handle in handles {
        let _ = handle.join();
    }
}

fn recommended_scan_workers(target_count: usize) -> usize {
    if target_count == 0 {
        return 0;
    }
    let logical_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    let budgeted_workers = logical_cores.saturating_sub(1).max(1);
    budgeted_workers.min(4).min(target_count)
}

fn scan_target(id: &str, path: &Path) -> (bool, u64) {
    if id == "venv_dirs" {
        return scan_venv_dirs(path);
    }

    let exists = path.exists();
    let size_bytes = if exists { dir_size_bytes(path) } else { 0 };
    (exists, size_bytes)
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

pub fn collect_deletion_entries_compact(
    target: &CacheTarget,
    min_file_size_bytes: u64,
) -> Vec<DeletionEntry> {
    let mut out = Vec::new();
    let roots: Vec<PathBuf> = if target.id == "venv_dirs" {
        find_venv_paths(&target.path)
    } else if target.path.exists() {
        vec![target.path.clone()]
    } else {
        vec![]
    };

    for root in roots {
        for entry in WalkDir::new(&root)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path().to_path_buf();
            if entry.file_type().is_dir() {
                out.push(DeletionEntry {
                    category: target.id.clone(),
                    path,
                    size_bytes: dir_size_bytes(entry.path()),
                    is_essential_warning: is_essential_target(&target.id),
                });
                continue;
            }

            let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
            if size_bytes >= min_file_size_bytes {
                out.push(DeletionEntry {
                    category: target.id.clone(),
                    path,
                    size_bytes,
                    is_essential_warning: is_essential_target(&target.id),
                });
            }
        }
    }

    out.sort_by(|a, b| {
        b.size_bytes
            .cmp(&a.size_bytes)
            .then_with(|| a.path.cmp(&b.path))
    });
    out
}

fn is_essential_target(target_id: &str) -> bool {
    matches!(
        target_id,
        "cargo_registry"
            | "cargo_git"
            | "rustup_downloads"
            | "rustup_tmp"
            | "venv_dirs"
            | "nuget_packages"
            | "gradle_caches"
            | "gradle_wrapper_dists"
            | "xcode_derived_data"
            | "swiftpm_cache"
            | "pub_cache"
            | "cabal_cache"
            | "stack_cache"
            | "ivy_cache"
            | "coursier_cache"
            | "sbt_boot_cache"
            | "huggingface_hub_cache"
    )
}

pub fn path_depth(path: &Path) -> usize {
    path.components().count()
}
