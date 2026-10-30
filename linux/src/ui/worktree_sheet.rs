//! New Session in Worktree (milestone 6). Until then a plain session is made.

use std::rc::Rc;

use uuid::Uuid;

use crate::app::App;
use crate::model::{SessionProvider, TabItem};

pub fn present(app: &Rc<App>, group_id: Uuid, provider: Option<SessionProvider>) {
    let count = app
        .group_index(group_id)
        .map(|gi| app.ws.borrow().groups[gi].tabs.len())
        .unwrap_or(0);
    let mut tab = TabItem::new(format!("Session {}", count + 1));
    if let Some(provider) = provider {
        tab.provider = Some(provider);
        tab.command = Some(provider.command().to_string());
    }
    app.add_session(group_id, tab, None, None);
}
