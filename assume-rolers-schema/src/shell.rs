use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Unknown(String),
}

impl Shell {
    pub fn from_process_path<P: AsRef<Path>>(process_path: P) -> Option<Shell> {
        let process = process_path.as_ref().file_stem().and_then(|f| f.to_str());
        process.map(|p| match p {
            "bash" => Shell::Bash,
            "zsh" => Shell::Zsh,
            "fish" => Shell::Fish,
            _ => Shell::Unknown(p.to_string()),
        })
    }
}
