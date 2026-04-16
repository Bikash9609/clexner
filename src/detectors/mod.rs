pub mod tools;

use tools::ToolInfo;

#[derive(Debug, Clone)]
pub struct EcosystemTools {
    pub ecosystem: String,
    pub tools: Vec<ToolInfo>,
}

impl EcosystemTools {
}

pub fn detect_tools() -> Vec<EcosystemTools> {
    vec![
        EcosystemTools {
            ecosystem: "Java".to_string(),
            tools: tools::detect_many(&["java", "javac", "mvn", "gradle"]),
        },
        EcosystemTools {
            ecosystem: "Python".to_string(),
            tools: tools::detect_many(&["python3", "pip3", "uv", "poetry", "pipx"]),
        },
        EcosystemTools {
            ecosystem: "JavaScript/Node".to_string(),
            tools: tools::detect_many(&["node", "npm", "yarn", "pnpm"]),
        },
        EcosystemTools {
            ecosystem: "Rust".to_string(),
            tools: tools::detect_many(&["rustc", "cargo", "rustup"]),
        },
        EcosystemTools {
            ecosystem: "Container".to_string(),
            tools: tools::detect_many(&["docker"]),
        },
        EcosystemTools {
            ecosystem: "Wasp".to_string(),
            tools: tools::detect_many(&["wasp"]),
        },
        EcosystemTools {
            ecosystem: "Common Managers".to_string(),
            tools: tools::detect_many(&["brew"]),
        },
    ]
}
