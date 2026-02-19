use ratatui::style::{Color, Modifier, Style};
use vte::{Params, Parser, Perform};

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
        }
    }
}

/// Internal state that implements vte::Perform, separate from the Parser
/// to avoid borrow checker issues.
struct TermState {
    cols: usize,
    rows: usize,
    grid: Vec<Vec<Cell>>,
    cursor_x: usize,
    cursor_y: usize,
    current_style: Style,
}

impl TermState {
    fn scroll_up(&mut self) {
        self.grid.remove(0);
        self.grid.push(vec![Cell::default(); self.cols]);
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        if self.cursor_y + 1 >= self.rows {
            self.scroll_up();
        } else {
            self.cursor_y += 1;
        }
    }
}

impl Perform for TermState {
    fn print(&mut self, c: char) {
        if self.cursor_x >= self.cols {
            self.newline();
        }
        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            self.grid[self.cursor_y][self.cursor_x] = Cell {
                ch: c,
                style: self.current_style,
            };
            self.cursor_x += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => self.cursor_x = 0,
            b'\x08' => self.cursor_x = self.cursor_x.saturating_sub(1),
            b'\t' => {
                let next_tab = (self.cursor_x / 8 + 1) * 8;
                self.cursor_x = next_tab.min(self.cols - 1);
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        let params: Vec<u16> = params.iter().map(|p| p[0]).collect();
        match action {
            'A' => {
                let n = params.first().copied().unwrap_or(1) as usize;
                self.cursor_y = self.cursor_y.saturating_sub(n);
            }
            'B' => {
                let n = params.first().copied().unwrap_or(1) as usize;
                self.cursor_y = (self.cursor_y + n).min(self.rows - 1);
            }
            'C' => {
                let n = params.first().copied().unwrap_or(1) as usize;
                self.cursor_x = (self.cursor_x + n).min(self.cols - 1);
            }
            'D' => {
                let n = params.first().copied().unwrap_or(1) as usize;
                self.cursor_x = self.cursor_x.saturating_sub(n);
            }
            'H' | 'f' => {
                let row = params.first().copied().unwrap_or(1).max(1) as usize - 1;
                let col = params.get(1).copied().unwrap_or(1).max(1) as usize - 1;
                self.cursor_y = row.min(self.rows - 1);
                self.cursor_x = col.min(self.cols - 1);
            }
            'J' => {
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => {
                        for x in self.cursor_x..self.cols {
                            self.grid[self.cursor_y][x] = Cell::default();
                        }
                        for y in (self.cursor_y + 1)..self.rows {
                            self.grid[y] = vec![Cell::default(); self.cols];
                        }
                    }
                    2 | 3 => {
                        self.grid = vec![vec![Cell::default(); self.cols]; self.rows];
                        self.cursor_x = 0;
                        self.cursor_y = 0;
                    }
                    _ => {}
                }
            }
            'K' => {
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => {
                        for x in self.cursor_x..self.cols {
                            self.grid[self.cursor_y][x] = Cell::default();
                        }
                    }
                    2 => {
                        self.grid[self.cursor_y] = vec![Cell::default(); self.cols];
                    }
                    _ => {}
                }
            }
            'm' => {
                if params.is_empty() {
                    self.current_style = Style::default();
                    return;
                }
                for &p in &params {
                    match p {
                        0 => self.current_style = Style::default(),
                        1 => self.current_style = self.current_style.add_modifier(Modifier::BOLD),
                        4 => self.current_style = self.current_style.add_modifier(Modifier::UNDERLINED),
                        7 => self.current_style = self.current_style.add_modifier(Modifier::REVERSED),
                        30..=37 => {
                            self.current_style = self.current_style.fg(ansi_color(p - 30));
                        }
                        40..=47 => {
                            self.current_style = self.current_style.bg(ansi_color(p - 40));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

pub struct VirtualTerminal {
    state: TermState,
    parser: Parser,
}

impl VirtualTerminal {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            state: TermState {
                cols,
                rows,
                grid: vec![vec![Cell::default(); cols]; rows],
                cursor_x: 0,
                cursor_y: 0,
                current_style: Style::default(),
            },
            parser: Parser::new(),
        }
    }

    pub fn process(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.parser.advance(&mut self.state, byte);
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.state.cols = cols;
        self.state.rows = rows;
        self.state.grid.resize(rows, vec![Cell::default(); cols]);
        for row in &mut self.state.grid {
            row.resize(cols, Cell::default());
        }
        if self.state.cursor_x >= cols {
            self.state.cursor_x = cols.saturating_sub(1);
        }
        if self.state.cursor_y >= rows {
            self.state.cursor_y = rows.saturating_sub(1);
        }
    }

    pub fn grid(&self) -> &Vec<Vec<Cell>> {
        &self.state.grid
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.state.cursor_x, self.state.cursor_y)
    }
}

fn ansi_color(code: u16) -> Color {
    match code {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        _ => Color::Reset,
    }
}
