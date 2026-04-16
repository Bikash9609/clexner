mod cache_targets;
mod cleaner;
mod cli;
mod detectors;
mod report;
mod scanner;
mod ui;

use anyhow::Result;
use cli::{Cli, Commands};
use std::io::{self, Write};
use std::time::Instant;

fn main() -> Result<()> {
    let cli = Cli::parse_args();
    println!("Phase 1/3: Detecting tools...");
    let tool_inventory = detectors::detect_tools();
    for eco in &tool_inventory {
        for tool in &eco.tools {
            if tool.installed {
                println!("  ✓ {}", tool.name);
            } else {
                println!("  ✗ {}", tool.name);
            }
        }
    }
    let include_venv = cli.include_venv;
    let mut targets = cache_targets::collect_cache_targets(&tool_inventory, include_venv)?;
    if matches!(cli.command, None | Some(Commands::Tui)) {
        println!("Phase 2/3: Scanning caches...");
        let start = Instant::now();
        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        scanner::scan_targets_with_progress(&mut targets, |current, total, path| {
            let width = 30usize;
            let filled = if total == 0 { 0 } else { (current * width) / total };
            let bar = format!("{}{}", "=".repeat(filled), " ".repeat(width - filled));
            let mut path_str = path.display().to_string();
            if path_str.len() > 72 {
                let keep = 72usize;
                let left = keep / 2;
                let right = keep - left;
                let left_part: String = path_str.chars().take(left).collect();
                let right_part: String = path_str
                    .chars()
                    .rev()
                    .take(right)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                path_str = format!("{}...{}", left_part, right_part);
            }
            let elapsed = start.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                current as f64 / elapsed
            } else {
                0.0
            };
            let spinner = spinner_frames[current % spinner_frames.len()];
            print!(
                "\r{spinner} Scanned [{bar}] {current}/{total}  elapsed: {elapsed:.1}s  speed: ~{speed:.1} targets/s  current: {path_str}"
            );
            let _ = io::stdout().flush();
            if current > 0 && total > 0 {
                println!("\n[✓] scanned {}", path_str);
            } else if total > 0 {
                println!("\n[~] scanning {}...", path_str);
            }
        });
        println!("\nPhase 3/3: Ready for cleanup. Opening UI...");
    } else {
        scanner::scan_targets(&mut targets);
    }

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Scan => {
            report::print_scan_report(&tool_inventory, &targets);
            Ok(())
        }
        Commands::ListTools => {
            report::print_tool_inventory(&tool_inventory);
            Ok(())
        }
        Commands::ListCaches => {
            report::print_cache_list(&targets);
            Ok(())
        }
        Commands::Tui => {
            ui::run_tui(&targets)?;
            Ok(())
        }
        Commands::Clean {
            targets: target_csv,
            confirm,
        } => {
            let requested = cli::parse_target_list(&target_csv);
            let selected: Vec<_> = targets
                .iter()
                .filter(|t| requested.contains(&t.id))
                .cloned()
                .collect();

            if selected.is_empty() {
                println!("No matching cache targets found.");
                return Ok(());
            }

            cleaner::confirm_and_clean(selected, confirm)
        }
    }
}
