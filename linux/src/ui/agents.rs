//! Subagents in the sidebar and header (filled in with milestone 4).

use uuid::Uuid;

use crate::app::App;

pub fn running_count(_app: &App, _id: Uuid) -> usize {
    0
}

pub fn update_sidebar(_app: &App, _id: Uuid, _container: &gtk::Box) {}

pub fn update_header(_app: &App, _id: Uuid, _container: &gtk::Box) {}
