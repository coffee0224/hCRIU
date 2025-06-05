use crate::utils::CheckpointMeta;
use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use std::io::stdout;

pub fn show_checkpoints_tui(checkpoints: Vec<&CheckpointMeta>) {
    let mut stdout = stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    terminal
        .draw(|f| {
            let area = f.area();
            let rows: Vec<Row> = checkpoints
                .iter()
                .map(|c| {
                    Row::new(vec![
                        c.checkpoint_id[..7].to_string(),
                        c.tag.clone(),
                        c.pid.to_string(),
                        c.cmd.clone(),
                        c.dump_time.clone(),
                    ])
                })
                .collect();
            let table = Table::new(
                rows,
                [
                    Constraint::Length(12),
                    Constraint::Length(12),
                    Constraint::Length(8),
                    Constraint::Length(30),
                    Constraint::Length(28),
                ],
            )
            .header(Row::new(vec![
                "Checkpoint ID",
                "Tag",
                "PID",
                "Command",
                "Dump Time",
            ]))
            .block(Block::default().borders(Borders::ALL).title("Checkpoints"))
            .style(Style::default().fg(Color::White));
            f.render_widget(table, area);
        })
        .unwrap();
}

pub fn interactive_tui<F: Fn(&str) -> String>(handler: F) {
    let mut stdout = stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut input = String::new();
    let mut output = String::new();
    terminal.clear().unwrap();
    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ])
                    .split(f.area());
                let input_box = Paragraph::new(input.as_str()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("input command: (dump/list/merge/restore/quit)"),
                );
                let output_box = Paragraph::new(output.as_str())
                    .block(Block::default().borders(Borders::ALL).title("输出"));
                f.render_widget(input_box, chunks[0]);
                f.render_widget(output_box, chunks[1]);
            })
            .unwrap();
        if event::poll(std::time::Duration::from_millis(200)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('q') if input.is_empty() => break,
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        if input.trim() == "quit" || input.trim() == "exit" {
                            break;
                        }
                        output = handler(input.trim());
                        input.clear();
                    }
                    _ => {}
                }
            }
        }
    }
    terminal.clear().unwrap();
}
