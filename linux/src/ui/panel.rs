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

/// Opens a subagent's transcript. (Shown in the Files panel's preview area
/// once the panel lands; a window until then.)
pub fn show_transcript(_id: Uuid, agent: &crate::claude::SubAgent) {
    let dialog = adw::Dialog::new();
    dialog.set_title(&agent.description);
    dialog.set_content_width(560);
    dialog.set_content_height(600);
    let view = adw::ToolbarView::new();
    view.add_top_bar(&adw::HeaderBar::new());
    view.set_content(Some(&crate::ui::agents::transcript_view(
        agent.transcript.clone(),
    )));
    dialog.set_child(Some(&view));
    let window = gtk::gio::Application::default()
        .and_then(|a| a.downcast::<gtk::Application>().ok())
        .and_then(|a| a.active_window());
    adw::prelude::AdwDialogExt::present(&dialog, window.as_ref());
}
