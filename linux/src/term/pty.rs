//! A session's pseudo-terminal. omux owns the PTY (instead of letting VTE
//! spawn the shell) so it can watch the output for OSC 7501 and friends; see
//! docs/ROADMAP.md, decision 1.

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use super::osc::{Event, Scanner};

/// What the reader thread reports to the main loop.
pub enum Output {
    Bytes(Vec<u8>, Vec<Event>),
    Exited,
}

pub struct Pty {
    master: Arc<File>,
    child: Arc<Mutex<Child>>,
    pid: i32,
}

pub struct Spawn<'a> {
    pub shell: &'a str,
    pub cwd: &'a Path,
    pub env: Vec<(String, String)>,
    pub rows: u16,
    pub cols: u16,
}

impl Pty {
    /// Starts `shell -l` as the session leader of a new PTY. Output arrives on
    /// `sink` from a background thread, already scanned for OSC events.
    pub fn spawn(spec: Spawn<'_>, sink: async_channel::Sender<Output>) -> io::Result<Self> {
        let (master, slave) = open_pty(spec.rows, spec.cols)?;

        let mut command = Command::new(spec.shell);
        command
            .arg("-l")
            .current_dir(spec.cwd)
            .env_clear()
            .envs(spec.env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .stdin(Stdio::from(slave.try_clone()?))
            .stdout(Stdio::from(slave.try_clone()?))
            .stderr(Stdio::from(slave));
        // Become the session leader with the PTY as controlling terminal, so
        // job control works and killpg reaches everything the shell started.
        unsafe {
            command.pre_exec(move || {
                if libc::setsid() < 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command.spawn()?;
        let pid = child.id() as i32;
        let master = Arc::new(File::from(master));
        let child = Arc::new(Mutex::new(child));

        let reader = master.clone();
        let reaper = child.clone();
        std::thread::Builder::new()
            .name(format!("omux-pty-{pid}"))
            .spawn(move || read_loop(reader, reaper, sink))?;

        Ok(Self { master, child, pid })
    }

    pub fn pid(&self) -> i32 {
        self.pid
    }

    pub fn write(&self, bytes: &[u8]) {
        let _ = (&*self.master).write_all(bytes);
    }

    pub fn resize(&self, rows: u16, cols: u16) {
        let size = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ, &size);
        }
    }

    /// Whether something other than the shell runs in the foreground, such as
    /// an agent or a build. (The Mac checks for any child of the shell.)
    pub fn is_busy(&self) -> bool {
        let group = unsafe { libc::tcgetpgrp(self.master.as_raw_fd()) };
        group > 0 && group != self.pid
    }

    /// Hangs up the whole session, then makes sure the shell goes.
    pub fn kill_tree(&self) {
        unsafe {
            libc::killpg(self.pid, libc::SIGHUP);
        }
        if let Ok(mut child) = self.child.lock() {
            if matches!(child.try_wait(), Ok(None)) {
                let _ = child.kill();
            }
        }
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        self.kill_tree();
    }
}

fn open_pty(rows: u16, cols: u16) -> io::Result<(OwnedFd, OwnedFd)> {
    let mut master: RawFd = -1;
    let mut slave: RawFd = -1;
    let size = libc::winsize {
        ws_row: rows.max(1),
        ws_col: cols.max(1),
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let result = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null(),
            &size,
        )
    };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    unsafe {
        // The shell must not inherit the master side.
        libc::fcntl(master, libc::F_SETFD, libc::FD_CLOEXEC);
        Ok((OwnedFd::from_raw_fd(master), OwnedFd::from_raw_fd(slave)))
    }
}

fn read_loop(master: Arc<File>, child: Arc<Mutex<Child>>, sink: async_channel::Sender<Output>) {
    let mut scanner = Scanner::default();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        match (&*master).read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let bytes = buffer[..n].to_vec();
                let mut events = Vec::new();
                scanner.feed(&bytes, &mut events);
                // SwiftTerm answers the program status query itself.
                for event in &events {
                    if let Event::StatusQuery { bel } = event {
                        let ending: &[u8] = if *bel { b"\x07" } else { b"\x1b\\" };
                        let _ = (&*master).write_all(&[b"\x1b]7501;?", ending].concat());
                    }
                }
                if sink.send_blocking(Output::Bytes(bytes, events)).is_err() {
                    break;
                }
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            // EIO once the last process holding the slave side exits.
            Err(_) => break,
        }
    }
    if let Ok(mut child) = child.lock() {
        let _ = child.wait();
    }
    let _ = sink.send_blocking(Output::Exited);
}

/// The login name for `USER`/`LOGNAME` when the environment lacks them.
pub fn login_name() -> Option<String> {
    unsafe {
        let pw = libc::getpwuid(libc::getuid());
        if pw.is_null() || (*pw).pw_name.is_null() {
            return None;
        }
        let name = std::ffi::CStr::from_ptr((*pw).pw_name);
        name.to_str().ok().map(str::to_string)
    }
}

/// The user's login shell from the password database, for when `$SHELL` is unset.
pub fn passwd_shell() -> Option<String> {
    unsafe {
        let pw = libc::getpwuid(libc::getuid());
        if pw.is_null() || (*pw).pw_shell.is_null() {
            return None;
        }
        let shell = std::ffi::CStr::from_ptr((*pw).pw_shell);
        shell
            .to_str()
            .ok()
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_a_shell_and_reports_output_and_exit() {
        let (tx, rx) = async_channel::unbounded();
        let pty = Pty::spawn(
            Spawn {
                shell: "/bin/sh",
                cwd: Path::new("/"),
                env: vec![("PATH".into(), "/usr/bin:/bin".into())],
                rows: 24,
                cols: 80,
            },
            tx,
        )
        .unwrap();
        pty.write(b"printf '\\033]7501;state=done\\007'; echo hi; exit 3\n");
        let mut text = Vec::new();
        let mut saw_status = false;
        while let Output::Bytes(bytes, events) = rx.recv_blocking().unwrap() {
            text.extend(bytes);
            saw_status |= events.iter().any(|e| matches!(e, Event::Status(_)));
        }
        assert!(String::from_utf8_lossy(&text).contains("hi"));
        assert!(saw_status);
    }
}
