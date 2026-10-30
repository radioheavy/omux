// The port is being built up milestone by milestone (docs/ROADMAP.md); parts
// of the core are not wired into the UI yet.
#![allow(dead_code)]

mod model;
mod persistence;
mod session_env;
mod settings;
mod swift_json;
mod term;

use adw::prelude::*;
use gtk::glib;

use term::SessionTerminal;

const APP_ID: &str = "net.pf1.omux";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_window);
    app.run()
}

fn build_window(app: &adw::Application) {
    let terminal = SessionTerminal::new();

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&adw::HeaderBar::new());
    content.append(&terminal.widget);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("omux")
        .default_width(1100)
        .default_height(720)
        .content(&content)
        .build();

    let win = window.downgrade();
    terminal.set_handler(move |event| {
        if let (term::TermEvent::Exited, Some(win)) = (&event, win.upgrade()) {
            win.close();
        }
    });
    let env = session_env::build(session_env::Extra {
        codex_token: None,
        repo: None,
    });
    if let Err(err) = terminal.spawn(&session_env::shell(), &persistence::home_dir(), env) {
        eprintln!("omux: could not start shell: {err}");
    }

    window.present();
    terminal.widget.grab_focus();
    // Keep the terminal alive as long as the window.
    unsafe { window.set_data("omux-terminal", terminal) };
}
