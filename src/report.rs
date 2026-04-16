use crate::cache_targets::CacheTarget;
use crate::detectors::EcosystemTools;
use humansize::{format_size, DECIMAL};

pub fn print_tool_inventory(inventory: &[EcosystemTools]) {
    println!("Installed tooling inventory:");
    for eco in inventory {
        println!("\n{}:", eco.ecosystem);
        for tool in &eco.tools {
            let state = if tool.installed { "installed" } else { "missing" };
            println!("- {:<12} {}", tool.name, state);
        }
    }
}

pub fn print_cache_list(targets: &[CacheTarget]) {
    println!("Known cache targets:");
    for t in targets {
        let mut extra = String::new();
        if t.optional {
            extra = " [optional]".to_string();
        }
        println!(
            "- {:<16} {:<16} {:<18} {}{}",
            t.id,
            t.ecosystem,
            format_size(t.size_bytes, DECIMAL),
            t.path.display(),
            extra
        );
    }
}

pub fn print_scan_report(inventory: &[EcosystemTools], targets: &[CacheTarget]) {
    print_tool_inventory(inventory);
    println!("\n----\n");
    print_cache_list(targets);
}
