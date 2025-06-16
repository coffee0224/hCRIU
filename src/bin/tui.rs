use std::{io::stdout, ops::ControlFlow, time::Duration};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, MouseEventKind, MouseButton, MouseEvent};
use crossterm::execute;
use ratatui::{
    layout::{Constraint, Layout, Position},
    widgets::{Block, Borders},
    style::{Style, Stylize, Color},
    Frame,
};

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    execute!(stdout(), EnableMouseCapture)?;
    let mut focused_area = FocusedArea::Checkpoints;
    let widgets = WidgetsArea::new(&terminal.get_frame());
    loop {
        terminal.draw(|f| draw(f, focused_area, &widgets)).expect("failed to draw frame");
        if handle_events(&widgets, &mut focused_area)? {
            break;
        }
    }
    ratatui::restore();
    if let Err(err) = execute!(stdout(), DisableMouseCapture) {
        eprintln!("Error disabling mouse capture: {err}");
    }
    Ok(())
}


fn draw(frame: &mut Frame, focused_area: FocusedArea, widgets: &WidgetsArea) {
    let default_border_style = Style::default().fg(Color::White);
    let focused_border_style = Style::default().fg(Color::Green);
    let [title_area, checkpoints_area, tasks_area, processes_area, status_area] = [
        widgets.title,
        widgets.checkpoints,
        widgets.tasks,
        widgets.processes,
        widgets.status,
    ];

    frame.render_widget(
        Block::default()
            .title("hcriu-ui")
            .borders(Borders::ALL)
            .border_style(if focused_area == FocusedArea::Title { focused_border_style } else { default_border_style }),
        title_area,
    );
    frame.render_widget(
        Block::default()
            .title("Checkpoints")
            .borders(Borders::ALL)
            .border_style(if focused_area == FocusedArea::Checkpoints { focused_border_style } else { default_border_style }),
        checkpoints_area,
    );
    frame.render_widget(
        Block::default()
            .title("Tasks")
            .borders(Borders::ALL)
            .border_style(if focused_area == FocusedArea::Tasks { focused_border_style } else { default_border_style }),
        tasks_area,
    );
    frame.render_widget(
        Block::default()
            .title("Processes")
            .borders(Borders::ALL)
            .border_style(if focused_area == FocusedArea::Processes { focused_border_style } else { default_border_style }),
        processes_area,
    );
    frame.render_widget(
        Block::default()
            .borders(Borders::empty()),
        status_area,
    );
}


struct WidgetsArea {
    title: ratatui::layout::Rect,
    checkpoints: ratatui::layout::Rect,
    tasks: ratatui::layout::Rect,
    processes: ratatui::layout::Rect,
    status: ratatui::layout::Rect,
}

impl WidgetsArea {
    fn new(frame: &Frame) -> Self {
        use Constraint::{Fill, Length, Min};

        let vertical = Layout::vertical([
            Length(1),
            Min(20),
            Length(1),
        ]);
    
        let [title_area, main_area, status_area] = vertical.areas(frame.area());
        let horizontal = Layout::horizontal([
            Fill(1); 2
        ]);
    
        let [left_area, processes_area] = horizontal.areas(main_area);
    
        let left_split = Layout::vertical([Fill(1); 2]);
        let [checkpoints_area, tasks_area]  = left_split.areas(left_area);

        Self {
            title: title_area,
            checkpoints: checkpoints_area,
            tasks: tasks_area,
            processes: processes_area,
            status: status_area,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum FocusedArea {
    Title,
    Checkpoints,
    Tasks,
    Processes,
}

fn handle_events(widgets: &WidgetsArea, focused_area: &mut FocusedArea) -> std::io::Result<bool> {
    match event::read()? {
        Event::Mouse(mouse_event) => {
            handle_mouse_events(mouse_event, focused_area, widgets);
        }
        Event::Key(key) if key.kind == KeyEventKind::Press =>  {
            return Ok(handle_key_events(key.code, focused_area));
        },
        _ => {}
    }
    Ok(false)
}

fn handle_mouse_events(
    mouse_event: MouseEvent,
    focused_area: &mut FocusedArea,
    widgets: &WidgetsArea,
) {
    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let [checkpoints_area, tasks_area, processes_area] = [
                widgets.checkpoints,
                widgets.tasks,
                widgets.processes,
            ];
            let mouse_pos = Position { x: mouse_event.column, y: mouse_event.row };
            // Check which area was clicked based on `mouse_pos` and update `focused_area`
            if checkpoints_area.contains(mouse_pos) {
                *focused_area = FocusedArea::Checkpoints;
            } else if tasks_area.contains(mouse_pos) {
                *focused_area = FocusedArea::Tasks;
            } else if processes_area.contains(mouse_pos) {
                *focused_area = FocusedArea::Processes;
            }
        }
        _ => {}
    }
}

fn handle_key_events(key: KeyCode, focused_area: &mut FocusedArea) -> bool {
    match key {
        KeyCode::Char('q') => {
            // Exit the application
            return true;
        }
        _ => {}
    }
    return false;
}
