//! Paste and drop into a session (PARITY.md section 10): files paste as
//! shell-escaped paths, an image on the clipboard is saved as a PNG and its
//! path pasted (Claude Code and Codex read it as an attachment), anything
//! else is a normal paste.

use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use uuid::Uuid;
use vte::prelude::*;

use crate::app::App;
use crate::term::SessionTerminal;

/// Escapes the characters a shell treats specially, so a dropped path is
/// one word. `~` is escaped everywhere, as on the Mac.
pub fn shell_escaped(paths: &[String]) -> String {
    paths
        .iter()
        .map(|path| {
            let mut out = String::with_capacity(path.len());
            for c in path.chars() {
                if " '\"()[]{}&;|<>$`\\*?#~".contains(c) {
                    out.push('\\');
                }
                out.push(c);
            }
            out
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn install(_app: &Rc<App>, _id: Uuid, terminal: &Rc<SessionTerminal>) {
    let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    let weak = Rc::downgrade(terminal);
    target.connect_drop(move |_, value, _, _| {
        let (Some(term), Ok(list)) = (weak.upgrade(), value.get::<gdk::FileList>()) else {
            return false;
        };
        let paths = file_paths(&list);
        if paths.is_empty() {
            return false;
        }
        term.widget.grab_focus();
        term.paste(&format!("{} ", shell_escaped(&paths)));
        true
    });
    terminal.widget.add_controller(target);
}

fn file_paths(list: &gdk::FileList) -> Vec<String> {
    list.files()
        .iter()
        .filter_map(|f| f.path())
        .map(|p| p.display().to_string())
        .collect()
}

/// Ctrl+Shift+V.
pub fn paste(terminal: &Rc<SessionTerminal>) {
    let clipboard = terminal.widget.clipboard();
    let formats = clipboard.formats();
    let has_text = formats.contains_type(glib::Type::STRING)
        || formats.contain_mime_type("text/plain")
        || formats.contain_mime_type("text/plain;charset=utf-8");
    if formats.contains_type(gdk::FileList::static_type()) {
        let weak = Rc::downgrade(terminal);
        clipboard.read_value_async(
            gdk::FileList::static_type(),
            glib::Priority::DEFAULT,
            gio::Cancellable::NONE,
            move |result| {
                let Some(term) = weak.upgrade() else { return };
                match result.ok().and_then(|v| v.get::<gdk::FileList>().ok()) {
                    Some(list) if !file_paths(&list).is_empty() => {
                        term.paste(&format!("{} ", shell_escaped(&file_paths(&list))));
                    }
                    _ => term.widget.paste_clipboard(),
                }
            },
        );
    } else if !has_text && formats.contains_type(gdk::Texture::static_type()) {
        let weak = Rc::downgrade(terminal);
        clipboard.read_texture_async(gio::Cancellable::NONE, move |result| {
            let Some(term) = weak.upgrade() else { return };
            let Ok(Some(texture)) = result else { return };
            match save_image(&texture) {
                Some(path) => term.paste(&format!("{} ", shell_escaped(&[path]))),
                None => term.widget.paste_clipboard(),
            }
        });
    } else {
        terminal.widget.paste_clipboard();
    }
}

/// `<tmp>/omux-paste/image-<date>-<time>-<ms>.png`. Milliseconds keep two
/// pastes in one second apart (the Mac overwrites).
fn save_image(texture: &gdk::Texture) -> Option<String> {
    let dir: PathBuf = std::env::temp_dir().join("omux-paste");
    std::fs::create_dir_all(&dir).ok()?;
    let now = glib::DateTime::now_local().ok()?;
    let stamp = now.format("%Y%m%d-%H%M%S").ok()?;
    let millis = now.microsecond() / 1000;
    let path = dir.join(format!("image-{stamp}-{millis:03}.png"));
    texture.save_to_png(&path).ok()?;
    Some(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_like_the_mac() {
        assert_eq!(
            shell_escaped(&["/tmp/My File (1).png".into(), "~/a&b".into()]),
            r"/tmp/My\ File\ \(1\).png \~/a\&b"
        );
        assert_eq!(shell_escaped(&["/tmp/çay-ü.txt".into()]), "/tmp/çay-ü.txt");
    }
}
