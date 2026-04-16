use crate::cache_targets::CacheTarget;
use crate::scanner;
use anyhow::Result;
use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use humansize::{format_size, DECIMAL};
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

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

    if !skip_confirm && !prompt_confirm()? {
        println!("Cleanup cancelled.");
        return Ok(());
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

pub fn confirm_and_clean_paths(mut entries: Vec<(PathBuf, u64)>, skip_confirm: bool) -> Result<()> {
    entries.sort_by_key(|(p, _)| std::cmp::Reverse(scanner::path_depth(p)));
    entries.dedup_by(|a, b| a.0 == b.0);

    println!("Paths scheduled for deletion:");
    for (p, size) in &entries {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or(Cow::Borrowed("/"));
        let emphasized_name = format!("\x1b[1m{}\x1b[0m", name);
        let line = format_path_line(p, &emphasized_name, *size, 120);
        println!("- {}", line);
    }

    if !skip_confirm && !prompt_confirm()? {
        println!("Cleanup cancelled.");
        return Ok(());
    }

    let start = Instant::now();
    let planned_bytes: u64 = entries.iter().map(|(_, size)| *size).sum();
    println!(
        "Deleting {} items ({})...",
        entries.len(),
        format_size(planned_bytes, DECIMAL)
    );
    let mut removed_bytes = 0u64;
    for (path, size) in entries {
        if path.is_file() {
            match fs::remove_file(&path) {
                Ok(_) => {
                    removed_bytes = removed_bytes.saturating_add(size);
                    println!("→ {}  ✔ removed", path.display());
                }
                Err(e) => println!("→ {}  ✖ failed: {}", path.display(), e),
            }
        } else if path.is_dir() {
            match fs::remove_dir_all(&path) {
                Ok(_) => {
                    removed_bytes = removed_bytes.saturating_add(size);
                    println!("→ {}  ✔ removed", path.display());
                }
                Err(e) => {
                    if path.exists() {
                        println!("→ {}  ✖ failed: {}", path.display(), e);
                    }
                }
            }
        }
    }
    println!(
        "Freed: {} in {:.1}s",
        format_size(removed_bytes, DECIMAL),
        start.elapsed().as_secs_f64()
    );
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

fn prompt_confirm() -> Result<bool> {
    println!("Confirm cleanup: press [y] to continue or [n]/[Esc] to cancel.");
    enable_raw_mode()?;
    let decision = loop {
        if let Event::Key(key) = read()? {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => break true,
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => break false,
                _ => {}
            }
        }
    };
    disable_raw_mode()?;
    println!();
    Ok(decision)
}
