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

fn main() -> Result<()> {
    let cli = Cli::parse_args();
    let tool_inventory = detectors::detect_tools();
    let include_venv = cli.include_venv;
    let mut targets = cache_targets::collect_cache_targets(&tool_inventory, include_venv)?;
    if matches!(cli.command, None | Some(Commands::Tui)) {
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
            print!("\rScanning [{bar}] {current}/{total} {path_str}");
            let _ = io::stdout().flush();
        });
        println!("\nScan complete. Opening UI...");
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
            let selected_paths = ui::run_tui(&targets)?;
            if selected_paths.is_empty() {
                println!("No paths selected.");
                return Ok(());
            }
            cleaner::confirm_and_clean_paths(selected_paths, false)
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
