use std::process::Command;

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub installed: bool,
}

pub fn detect_many(names: &[&str]) -> Vec<ToolInfo> {
    names.iter().map(|n| detect_tool(n)).collect()
}

fn detect_tool(name: &str) -> ToolInfo {
    let installed = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    ToolInfo {
        name: name.to_string(),
        installed,
    }
}
