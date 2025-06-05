use crate::utils::CheckpointMeta;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Row, Table};
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
