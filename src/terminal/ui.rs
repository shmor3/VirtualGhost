use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::vterm::VirtualTerminal;

pub fn render(frame: &mut Frame, vterm: &VirtualTerminal, status_text: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_terminal(frame, chunks[0], vterm);
    render_status_bar(frame, chunks[1], status_text);
}

fn render_terminal(frame: &mut Frame, area: Rect, vterm: &VirtualTerminal) {
    let mut lines = Vec::new();

    for row in vterm.grid() {
        let mut spans = Vec::new();
        for cell in row {
            spans.push(Span::styled(cell.ch.to_string(), cell.style));
        }
        lines.push(Line::from(spans));
    }

    let terminal_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(terminal_widget, area);

    let (cx, cy) = vterm.cursor();
    if cx < area.width as usize && cy < area.height as usize {
        frame.set_cursor_position((
            area.x + cx as u16,
            area.y + cy as u16,
        ));
    }
}

fn render_status_bar(frame: &mut Frame, area: Rect, status_text: &str) {
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" Ghostly Term ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(status_text, Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(status, area);
}
