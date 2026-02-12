use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use portable_pty::{Child, CommandBuilder, PtyPair, PtySize, native_pty_system};
use vt100::Parser;

pub struct TerminalPanel {
    pty: Option<PtyPair>,
    child: Option<Box<dyn Child + Send + Sync>>,
    writer: Option<Box<dyn Write + Send>>,
    parser: Arc<Mutex<Parser>>,
    output_rx: Option<mpsc::Receiver<Vec<u8>>>,
    rows: u16,
    cols: u16,
    current_dir: PathBuf,
}

impl Default for TerminalPanel {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

impl TerminalPanel {
    pub fn new(cols: u16, rows: u16) -> Self {
        let parser = Parser::new(rows, cols, 1000); // 1000 lines scrollback

        Self {
            pty: None,
            child: None,
            writer: None,
            parser: Arc::new(Mutex::new(parser)),
            output_rx: None,
            rows,
            cols,
            current_dir: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        }
    }

    pub fn spawn_shell(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            rows: self.rows,
            cols: self.cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Use platform-appropriate shell
        let shell = if cfg!(windows) {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        };
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(&self.current_dir);

        let child = pair.slave.spawn_command(cmd)?;
        self.child = Some(child);

        // Take the writer for reuse across send_input calls
        let writer = pair.master.take_writer()?;
        self.writer = Some(writer);

        // Spawn background thread to read PTY output
        let mut reader = pair.master.try_clone_reader()?;
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        self.output_rx = Some(rx);
        self.pty = Some(pair);
        Ok(())
    }

    /// Poll for new output from the background reader thread (non-blocking).
    /// Returns true if the shell process exited and was respawned.
    pub fn poll_output(&mut self) -> bool {
        // Read any available output
        if let Some(ref rx) = self.output_rx {
            loop {
                match rx.try_recv() {
                    Ok(data) => {
                        let mut parser = self.parser.lock().unwrap();
                        parser.process(&data);
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
            }
        }

        // Check if child process has exited
        let exited = if let Some(ref mut child) = self.child {
            matches!(child.try_wait(), Ok(Some(_)))
        } else {
            false
        };

        if exited {
            // Shell exited — clean up and respawn with fresh screen
            self.child = None;
            self.pty = None;
            self.writer = None;
            self.output_rx = None;
            self.parser = Arc::new(Mutex::new(Parser::new(self.rows, self.cols, 1000)));
            let _ = self.spawn_shell();
            return true;
        }
        false
    }

    pub fn set_working_dir(&mut self, dir: PathBuf) {
        if self.current_dir == dir {
            return;
        }
        self.current_dir = dir.clone();
        if self.is_running() {
            // cd /d handles drive letter changes on Windows
            let cd_cmd = if cfg!(windows) {
                format!("cd /d \"{}\"\r", dir.display())
            } else {
                format!("cd \"{}\"\n", dir.display())
            };
            self.send_input(&cd_cmd);
        }
    }

    pub fn send_input(&mut self, input: &str) {
        if let Some(ref mut writer) = self.writer {
            let _ = writer.write_all(input.as_bytes());
            let _ = writer.flush();
        }
    }

    pub fn send_key(&mut self, key: TerminalKey) {
        let bytes = match key {
            TerminalKey::Char(c) => c.to_string(),
            TerminalKey::Enter => "\r".to_string(),
            TerminalKey::Backspace => "\x7f".to_string(),
            TerminalKey::Tab => "\t".to_string(),
            TerminalKey::Escape => "\x1b".to_string(),
            TerminalKey::Up => "\x1b[A".to_string(),
            TerminalKey::Down => "\x1b[B".to_string(),
            TerminalKey::Right => "\x1b[C".to_string(),
            TerminalKey::Left => "\x1b[D".to_string(),
            TerminalKey::Home => "\x1b[H".to_string(),
            TerminalKey::End => "\x1b[F".to_string(),
            TerminalKey::Delete => "\x1b[3~".to_string(),
            TerminalKey::CtrlC => "\x03".to_string(),
            TerminalKey::CtrlD => "\x04".to_string(),
            TerminalKey::CtrlZ => "\x1a".to_string(),
            TerminalKey::CtrlL => "\x0c".to_string(),
        };
        self.send_input(&bytes);
    }

    pub fn screen_contents(&self) -> Vec<String> {
        let parser = self.parser.lock().unwrap();
        let screen = parser.screen();
        screen
            .contents()
            .lines()
            .map(|line| line.to_string())
            .collect()
    }

    pub fn cursor_position(&self) -> (u16, u16) {
        let parser = self.parser.lock().unwrap();
        let screen = parser.screen();
        let pos = screen.cursor_position();
        (pos.0, pos.1)
    }

    /// Try to detect the shell's current directory from the prompt line.
    /// Windows cmd.exe: "C:\Users\grzeg>"
    /// Linux bash: "user@host:/path$" or "user@host:~$" or "user@host:~/sub$"
    pub fn detect_cwd(&self) -> Option<PathBuf> {
        let parser = self.parser.lock().unwrap();
        let screen = parser.screen();
        let (cursor_row, _) = screen.cursor_position();

        let line = screen
            .contents_between(cursor_row, 0, cursor_row, self.cols - 1)
            .trim()
            .to_string();

        // Windows cmd.exe prompt: "C:\some\path>"
        if let Some(pos) = line.rfind('>') {
            let candidate = &line[..pos];
            let path = PathBuf::from(candidate);
            if path.is_dir() {
                return Some(path);
            }
        }

        // Linux/macOS bash prompt: "user@host:/path$ " or "user@host:~$ "
        // Look for `:path$` or `:path#` pattern
        if let Some(colon_pos) = line.rfind(':') {
            let after_colon = &line[colon_pos + 1..];
            // Strip trailing $ or # and whitespace
            let path_str = after_colon
                .trim_end()
                .trim_end_matches('$')
                .trim_end_matches('#')
                .trim_end();
            if !path_str.is_empty() {
                // Expand ~ to home directory
                let expanded = if path_str == "~" {
                    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
                } else if let Some(rest) = path_str.strip_prefix("~/") {
                    dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("/"))
                        .join(rest)
                } else {
                    PathBuf::from(path_str)
                };
                if expanded.is_dir() {
                    return Some(expanded);
                }
            }
        }

        None
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;

        if let Some(ref pty) = self.pty {
            let _ = pty.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }

        let mut parser = self.parser.lock().unwrap();
        parser.set_size(rows, cols);
    }

    pub fn is_running(&self) -> bool {
        self.pty.is_some()
    }
}

#[derive(Debug, Clone)]
pub enum TerminalKey {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Delete,
    CtrlC,
    CtrlD,
    CtrlZ,
    CtrlL,
}
