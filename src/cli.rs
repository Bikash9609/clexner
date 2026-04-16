use clap::{Parser, Subcommand};
use std::collections::HashSet;

#[derive(Parser, Debug)]
#[command(name = "cachectl", about = "On-demand dev tooling cache cleaner")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, help = "Include optional .venv folders in cache target catalog")]
    pub include_venv: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Scan,
    ListTools,
    ListCaches,
    Tui,
    Clean {
        #[arg(
            long,
            help = "Comma-separated target IDs (example: npm_cache,yarn_cache)"
        )]
        targets: String,
        #[arg(long, help = "Skip interactive confirm prompt")]
        confirm: bool,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

pub fn parse_target_list(csv: &str) -> HashSet<String> {
    csv.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}
