//! Claude Code's files: where conversations live.

use std::path::PathBuf;

use uuid::Uuid;

use crate::persistence::home_dir;

/// `$CLAUDE_CONFIG_DIR`, else `~/.claude`.
pub fn config_dir() -> PathBuf {
    match std::env::var("CLAUDE_CONFIG_DIR") {
        Ok(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => home_dir().join(".claude"),
    }
}

pub fn projects_dir() -> PathBuf {
    config_dir().join("projects")
}

/// Whether Claude has a transcript for this session id, in any project
/// folder (decides `--resume` over `--session-id`).
pub fn conversation_exists(session: Uuid) -> bool {
    let name = format!("{}.jsonl", session.to_string().to_lowercase());
    let Ok(entries) = std::fs::read_dir(projects_dir()) else {
        return false;
    };
    entries
        .flatten()
        .any(|entry| entry.path().join(&name).is_file())
}
