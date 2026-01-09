use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use vt100::Parser;

pub struct TerminalPanel {
    pty: Option<PtyPair>,
    parser: Arc<Mutex<Parser>>,
    output_buffer: Arc<Mutex<Vec<u8>>>,
    input_buffer: String,
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
            parser: Arc::new(Mutex::new(parser)),
            output_buffer: Arc::new(Mutex::new(Vec::new())),
            input_buffer: String::new(),
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

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(&self.current_dir);

        let _child = pair.slave.spawn_command(cmd)?;

        self.pty = Some(pair);
        Ok(())
    }

    pub fn set_working_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
        // Send cd command to terminal if running
        if self.pty.is_some() {
            let cd_cmd = format!("cd {:?}\n", self.current_dir);
            self.send_input(&cd_cmd);
        }
    }

    pub fn send_input(&mut self, input: &str) {
        if let Some(ref mut pty) = self.pty {
            if let Ok(mut writer) = pty.master.take_writer() {
                let _ = writer.write_all(input.as_bytes());
                let _ = writer.flush();
            }
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

    pub fn read_output(&mut self) -> Option<String> {
        if let Some(ref mut pty) = self.pty {
            if let Ok(mut reader) = pty.master.try_clone_reader() {
                let mut buf = [0u8; 4096];
                if let Ok(n) = reader.read(&mut buf) {
                    if n > 0 {
                        let mut parser = self.parser.lock().unwrap();
                        parser.process(&buf[..n]);
                        return Some(String::from_utf8_lossy(&buf[..n]).to_string());
                    }
                }
            }
        }
        None
    }

    pub fn screen_contents(&self) -> Vec<String> {
        let parser = self.parser.lock().unwrap();
        let screen = parser.screen();
        (0..self.rows)
            .map(|row| screen.row_wrapped(row).to_string())
            .collect()
    }

    pub fn cursor_position(&self) -> (u16, u16) {
        let parser = self.parser.lock().unwrap();
        let screen = parser.screen();
        (screen.cursor_position().0, screen.cursor_position().1)
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
