use crate::error::VirtualGhostError;
use crossterm::event;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;
use tracing::info;

use super::input::{Action, InputHandler};
use super::ui;
use super::vterm::VirtualTerminal;

pub struct App {
    vterm: VirtualTerminal,
    status: String,
    should_quit: bool,
}

impl App {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            vterm: VirtualTerminal::new(cols, rows),
            status: "Initializing...".to_string(),
            should_quit: false,
        }
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    pub fn process_output(&mut self, bytes: &[u8]) {
        self.vterm.process(bytes);
    }

    /// Run the standalone terminal UI event loop (for testing without a VM).
    pub async fn run_standalone(&mut self) -> Result<(), VirtualGhostError> {
        terminal::enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;

        self.set_status("Standalone mode — no VM connected");

        while !self.should_quit {
            terminal.draw(|frame| {
                ui::render(frame, &self.vterm, &self.status);
            })?;

            if event::poll(Duration::from_millis(16))? {
                let evt = event::read()?;
                match InputHandler::handle_event(evt) {
                    Action::Quit => self.should_quit = true,
                    Action::Resize(cols, rows) => {
                        self.vterm.resize(cols as usize, rows as usize);
                    }
                    Action::SendBytes(bytes) => {
                        // In standalone mode, echo locally
                        self.vterm.process(&bytes);
                    }
                    Action::None => {}
                }
            }
        }

        terminal::disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        info!("Terminal UI shut down");
        Ok(())
    }
}
