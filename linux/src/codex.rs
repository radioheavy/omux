//! Managed Codex sessions. Until the app-server integration lands
//! (docs/ROADMAP.md, milestone 5) a Codex session runs the `codex` TUI
//! directly in its shell.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use uuid::Uuid;

use crate::app::App;

thread_local! {
    static TOKENS: RefCell<HashMap<Uuid, String>> = RefCell::new(HashMap::new());
    static SETUP: RefCell<HashMap<Uuid, String>> = RefCell::new(HashMap::new());
}

/// The bearer token for a session's private app-server, kept in memory only.
pub fn token_for(_app: &Rc<App>, id: Uuid) -> String {
    TOKENS.with(|t| {
        t.borrow_mut()
            .entry(id)
            .or_insert_with(|| format!("{}{}", Uuid::new_v4(), Uuid::new_v4()).to_uppercase())
            .clone()
    })
}

pub fn start(app: &Rc<App>, id: Uuid, _dir: &Path) {
    let setup = SETUP.with(|s| s.borrow_mut().remove(&id));
    let command = match setup {
        Some(setup) => format!("{setup} && codex\n"),
        None => "codex\n".to_string(),
    };
    app.type_later(id, command, Duration::from_millis(300));
}

pub fn set_pending_setup(_app: &Rc<App>, id: Uuid, setup: String) {
    SETUP.with(|s| s.borrow_mut().insert(id, setup));
}

pub fn is_starting(_app: &Rc<App>, _id: Uuid) -> bool {
    false
}

pub fn close(_app: &Rc<App>, _id: Uuid) {}

pub fn interrupt(app: &Rc<App>, id: Uuid) {
    if let Some(term) = app.terminal(id) {
        term.write(b"\x03");
    }
}

pub fn answer(_app: &Rc<App>, _id: Uuid, _allow: bool) {}

/// The pending approval's detail text, if Codex is waiting for one.
pub fn pending_approval(_app: &App, _id: Uuid) -> Option<String> {
    None
}

pub fn shutdown(_app: &App) {}

/// Recent Codex conversations (from the catalog app-server).
pub fn recent() -> Result<Vec<crate::ui::recent::Conversation>, String> {
    Err("Codex conversations are listed once omux manages Codex sessions.".into())
}

/// The selected Codex session's plan limits, and its plan name.
pub fn usage(_app: &App) -> (Option<crate::usage::Usage>, Option<String>) {
    (None, None)
}

/// The Codex subagents button in the session header.
pub fn header_agents(_app: &App, _id: Uuid, _container: &gtk::Box) {}
