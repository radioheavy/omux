use adw::prelude::*;
use gtk::{gio, glib};
use vte::prelude::*;

const APP_ID: &str = "net.pf1.omux";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_window);
    app.run()
}

fn build_window(app: &adw::Application) {
    let terminal = vte::Terminal::new();
    terminal.set_vexpand(true);
    spawn_shell(&terminal);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&adw::HeaderBar::new());
    content.append(&terminal);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("omux")
        .default_width(1100)
        .default_height(720)
        .content(&content)
        .build();

    // Closing the shell closes the window, like a plain terminal for now.
    let win = window.downgrade();
    terminal.connect_child_exited(move |_, _| {
        if let Some(win) = win.upgrade() {
            win.close();
        }
    });

    window.present();
    terminal.grab_focus();
}

fn spawn_shell(terminal: &vte::Terminal) {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
    let home = glib::home_dir();
    terminal.spawn_async(
        vte::PtyFlags::DEFAULT,
        home.to_str(),
        &[&shell],
        &[],
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        None::<&gio::Cancellable>,
        |result| {
            if let Err(err) = result {
                eprintln!("omux: could not start shell: {err}");
            }
        },
    );
}
