//! Worktree items in the session menu (milestone 6).

use std::rc::Rc;

use gtk::gio;
use uuid::Uuid;

use crate::app::App;

pub fn menu_section(_app: &Rc<App>, _id: Uuid) -> Option<gio::Menu> {
    None
}

/// Close Session. Worktree sessions ask whether to keep the worktree
/// (milestone 6); others close right away.
pub fn request_close(app: &Rc<App>, id: Uuid) {
    app.close_tab(id);
}

pub fn install(_app: &Rc<App>) {}
