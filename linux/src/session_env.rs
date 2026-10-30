//! The environment and shell a session starts with (docs/ROADMAP.md,
//! decision 2). Unlike the Mac app, which builds a clean environment with a
//! Homebrew PATH, the session inherits omux's environment: on Linux that
//! carries the display, D-Bus, SSH agent and locale programs need.

use crate::term::pty;

/// Variables that would make Claude Code think it runs inside another
/// Claude Code session when omux itself was started from one.
const STRIPPED: [&str; 3] = [
    "CLAUDECODE",
    "CLAUDE_CODE_ENTRYPOINT",
    "CLAUDE_CODE_SSE_PORT",
];

pub struct Extra<'a> {
    pub codex_token: Option<&'a str>,
    pub repo: Option<&'a str>,
}

pub fn shell() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(pty::passwd_shell)
        .unwrap_or_else(|| "/bin/bash".into())
}

pub fn build(extra: Extra<'_>) -> Vec<(String, String)> {
    build_from(std::env::vars().collect(), extra)
}

fn build_from(inherited: Vec<(String, String)>, extra: Extra<'_>) -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = inherited
        .into_iter()
        .filter(|(key, _)| !STRIPPED.contains(&key.as_str()) && !key.starts_with("OMUX_"))
        .filter(|(key, _)| {
            !matches!(
                key.as_str(),
                "TERM" | "COLORTERM" | "TERM_PROGRAM" | "TERM_PROGRAM_VERSION" | "VTE_VERSION"
            )
        })
        .collect();
    let mut set = |key: &str, value: &str| {
        env.retain(|(k, _)| k != key);
        env.push((key.into(), value.into()));
    };
    if std::env::var("USER").is_err() {
        if let Some(name) = pty::login_name() {
            set("USER", &name);
            set("LOGNAME", &name);
        }
    }
    if std::env::var("LANG").is_err() {
        set("LANG", "C.UTF-8");
    }
    set("SHELL", &shell());
    set("TERM", "xterm-256color");
    set("COLORTERM", "truecolor");
    set("TERM_PROGRAM", "omux");
    set("TERM_PROGRAM_VERSION", env!("CARGO_PKG_VERSION"));
    if let Some(token) = extra.codex_token {
        set("OMUX_CODEX_TOKEN", token);
    }
    if let Some(repo) = extra.repo {
        set("OMUX_REPO", repo);
    }
    env
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
        env.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    #[test]
    fn strips_nested_claude_markers_and_sets_terminal_identity() {
        let inherited = vec![
            ("CLAUDECODE".into(), "1".into()),
            ("TERM".into(), "dumb".into()),
            ("DISPLAY".into(), ":0".into()),
            ("OMUX_CODEX_TOKEN".into(), "stale".into()),
        ];
        let env = build_from(
            inherited,
            Extra {
                codex_token: None,
                repo: Some("/src/app"),
            },
        );
        assert_eq!(get(&env, "CLAUDECODE"), None);
        assert_eq!(
            get(&env, "OMUX_CODEX_TOKEN"),
            None,
            "a token never leaks between sessions"
        );
        assert_eq!(get(&env, "DISPLAY"), Some(":0"));
        assert_eq!(get(&env, "TERM"), Some("xterm-256color"));
        assert_eq!(get(&env, "TERM_PROGRAM"), Some("omux"));
        assert_eq!(get(&env, "OMUX_REPO"), Some("/src/app"));
        assert_eq!(env.iter().filter(|(k, _)| k == "TERM").count(), 1);
    }
}
