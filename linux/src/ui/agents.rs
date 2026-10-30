//! Subagents in the sidebar and header (filled in with milestone 4).

use std::rc::Rc;

use uuid::Uuid;

use crate::app::App;

pub fn running_count(_app: &Rc<App>, _id: Uuid) -> usize {
    0
}

pub fn update_sidebar(_app: &Rc<App>, _id: Uuid, _container: &gtk::Box) {}

pub fn update_header(_app: &Rc<App>, _id: Uuid, _container: &gtk::Box) {}
