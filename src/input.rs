use anyhow::Result;
use std::io::Read;

/// Strip ANSI escape sequences (e.g. colour codes git adds when color.pager is on).
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&d) = chars.peek() {
                    chars.next();
                    if d.is_ascii_alphabetic() { break; }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Read all of stdin, strip ANSI codes, then reopen /dev/tty on fd 0 so
/// crossterm can read keyboard events after the pipe is consumed.
/// Returns (stripped, raw) — stripped for diff parsing, raw for passthrough.
pub fn read_piped_stdin() -> Result<(String, String)> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw)?;
    let input = strip_ansi(&raw);

    unsafe { libc::close(libc::STDIN_FILENO) };

    let tty = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")?;

    let tty_fd = std::os::unix::io::AsRawFd::as_raw_fd(&tty);
    if tty_fd != libc::STDIN_FILENO {
        if unsafe { libc::dup2(tty_fd, libc::STDIN_FILENO) } == -1 {
            anyhow::bail!("dup2 failed: {}", std::io::Error::last_os_error());
        }
    } else {
        std::mem::forget(tty);
    }

    Ok((input, raw))
}
