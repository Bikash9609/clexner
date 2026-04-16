use crate::cache_targets::CacheTarget;
use crate::scanner;
use anyhow::Result;
use humansize::{format_size, DECIMAL};
use std::borrow::Cow;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn confirm_and_clean(targets: Vec<CacheTarget>, skip_confirm: bool) -> Result<()> {
    println!("Selected targets (specific = Space, group = g, all = a):");
    let mut expanded_paths: Vec<(PathBuf, u64)> = Vec::new();
    for t in &targets {
        expanded_paths.extend(
            scanner::collect_deletion_entries(t)
                .into_iter()
                .map(|e| (e.path, e.size_bytes)),
        );
    }

    let total: u64 = expanded_paths.iter().map(|(_, s)| *s).sum();
    println!("Paths scheduled for deletion:");
    for (path, size) in &expanded_paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or(Cow::Borrowed("/"));
        let emphasized_name = format!("\x1b[1m{}\x1b[0m", name);
        let line = format_path_line(path, &emphasized_name, *size, 120);
        println!("- {}", line);
    }
    println!("Total potential cleanup: {}", format_size(total, DECIMAL));

    if !skip_confirm {
        print!("Type YES to continue: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim() != "YES" {
            println!("Cleanup cancelled.");
            return Ok(());
        }
    }

    for target in targets {
        if target.id == "venv_dirs" {
            let venvs = scanner::find_venv_paths(&target.path);
            for venv in venvs {
                match fs::remove_dir_all(&venv) {
                    Ok(_) => println!("Removed {}", venv.display()),
                    Err(e) => println!("Failed {}: {}", venv.display(), e),
                }
            }
            continue;
        }

        if !target.path.exists() {
            println!("Skipped {} (missing)", target.label);
            continue;
        }
        match fs::remove_dir_all(&target.path) {
            Ok(_) => println!("Removed {}", target.path.display()),
            Err(e) => println!("Failed {}: {}", target.path.display(), e),
        }
    }

    Ok(())
}

pub fn confirm_and_clean_paths(mut paths: Vec<PathBuf>, skip_confirm: bool) -> Result<()> {
    paths.sort_by_key(|p| std::cmp::Reverse(scanner::path_depth(p)));
    paths.dedup();

    println!("Paths scheduled for deletion:");
    for p in &paths {
        let size = if p.is_file() {
            fs::metadata(p).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or(Cow::Borrowed("/"));
        let emphasized_name = format!("\x1b[1m{}\x1b[0m", name);
        let line = format_path_line(p, &emphasized_name, size, 120);
        println!("- {}", line);
    }

    if !skip_confirm {
        print!("Type YES to continue: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim() != "YES" {
            println!("Cleanup cancelled.");
            return Ok(());
        }
    }

    for path in paths {
        if path.is_file() {
            match fs::remove_file(&path) {
                Ok(_) => println!("Removed {}", path.display()),
                Err(e) => println!("Failed {}: {}", path.display(), e),
            }
        } else if path.is_dir() {
            match fs::remove_dir_all(&path) {
                Ok(_) => println!("Removed {}", path.display()),
                Err(e) => {
                    if path.exists() {
                        println!("Failed {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    Ok(())
}

fn format_path_line(path: &Path, highlighted_name: &str, size: u64, width: usize) -> String {
    let size_str = format_size(size, DECIMAL);
    let mut full = path.display().to_string();
    let name_plain = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if !name_plain.is_empty() {
        full = full.replacen(&name_plain, highlighted_name, 1);
    }

    let tail = format!("  {}", size_str);
    if full.len() + tail.len() <= width {
        return format!("{}{}", full, tail);
    }

    let keep = width.saturating_sub(tail.len() + 3);
    let left = keep / 2;
    let right = keep.saturating_sub(left);

    let left_part: String = full.chars().take(left).collect();
    let right_part: String = full
        .chars()
        .rev()
        .take(right)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{}...{}{}", left_part, right_part, tail)
}
