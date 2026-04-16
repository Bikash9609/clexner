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
            ecosystem: "Go".to_string(),
            tools: tools::detect_many(&["go"]),
        },
        EcosystemTools {
            ecosystem: "Ruby".to_string(),
            tools: tools::detect_many(&["ruby", "gem", "bundle", "bundler"]),
        },
        EcosystemTools {
            ecosystem: "PHP".to_string(),
            tools: tools::detect_many(&["php", "composer"]),
        },
        EcosystemTools {
            ecosystem: ".NET".to_string(),
            tools: tools::detect_many(&["dotnet"]),
        },
        EcosystemTools {
            ecosystem: "C/C++".to_string(),
            tools: tools::detect_many(&["gcc", "g++", "clang", "cmake", "make", "ninja"]),
        },
        EcosystemTools {
            ecosystem: "Android".to_string(),
            tools: tools::detect_many(&["adb", "gradle", "sdkmanager", "avdmanager"]),
        },
        EcosystemTools {
            ecosystem: "Swift/iOS".to_string(),
            tools: tools::detect_many(&["swift", "swiftc", "xcodebuild"]),
        },
        EcosystemTools {
            ecosystem: "Flutter/Dart".to_string(),
            tools: tools::detect_many(&["flutter", "dart"]),
        },
        EcosystemTools {
            ecosystem: "Haskell".to_string(),
            tools: tools::detect_many(&["ghc", "cabal", "stack"]),
        },
        EcosystemTools {
            ecosystem: "Elixir/Erlang".to_string(),
            tools: tools::detect_many(&["elixir", "mix", "erl"]),
        },
        EcosystemTools {
            ecosystem: "Scala".to_string(),
            tools: tools::detect_many(&["scala", "sbt"]),
        },
        EcosystemTools {
            ecosystem: "ML/Data".to_string(),
            tools: tools::detect_many(&["conda", "mamba", "pipenv", "jupyter", "huggingface-cli"]),
        },
        EcosystemTools {
            ecosystem: "DevOps".to_string(),
            tools: tools::detect_many(&["kubectl", "helm", "terraform", "ansible"]),
        },
        EcosystemTools {
            ecosystem: "Databases".to_string(),
            tools: tools::detect_many(&["mysql", "psql", "redis-cli", "mongod"]),
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
            tools: tools::detect_many(&[
                "brew", "apt", "apt-get", "yum", "dnf", "pacman", "snap",
            ]),
        },
        EcosystemTools {
            ecosystem: "Misc".to_string(),
            tools: tools::detect_many(&["git", "ffmpeg", "imagemagick"]),
        },
    ]
}
