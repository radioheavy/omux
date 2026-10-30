//! The right panel: Files, Changes and Browser (filled in with milestones 6 and 7).

use std::path::Path;
use std::rc::Rc;

use gtk::prelude::*;
use uuid::Uuid;

use crate::app::App;

pub fn build(app: &Rc<App>) {
    let page = adw::StatusPage::builder()
        .title("Files, Changes and Browser")
        .description("Coming in the next milestones.")
        .icon_name("folder-symbolic")
        .vexpand(true)
        .build();
    app.ui.panel.append(&page);
}

pub fn selection_changed(_app: &Rc<App>) {}

pub fn session_closed(_app: &Rc<App>, _id: Uuid) {}

pub fn app_activated(_app: &Rc<App>) {}

pub fn preview_file(app: &Rc<App>, path: &Path) {
    let uri = gtk::gio::File::for_path(path).uri();
    app.open_uri(&uri);
}

pub fn show_tab(app: &Rc<App>, tab: &str) {
    let tab = tab.to_string();
    app.update_settings(|s| s.panel_tab = tab);
    if !app.settings.borrow().show_files {
        gtk::prelude::ActionGroupExt::activate_action(&app.gtk_app, "toggle-panel", None);
    }
}
