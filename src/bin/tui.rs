use std::{io::stdout, ops::ControlFlow, time::Duration};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind, MouseButton, MouseEvent};
use crossterm::execute;
use ratatui::{
    layout::{Constraint, Layout, Position},
    widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    style::{Style, Color},
    Frame,
};
use hcriu::utils::{CheckpointMeta, get_all_checkpoints, set_hcriu_dir};
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    execute!(stdout(), EnableMouseCapture)?;
    let mut focused_area = FocusedArea::Checkpoints;
    let widgets = WidgetsArea::new(&terminal.get_frame());
    
    // Initialize hcriu directory
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let hcriu_dir = home_dir.join(".hcriu");
    set_hcriu_dir(hcriu_dir);
    
    // Initialize app state

    let checkpoints = get_all_checkpoints();
    let mut app_state = AppState::new(checkpoints);
    
    loop {
        terminal.draw(|f| draw(f, focused_area, &widgets, &mut app_state)).expect("failed to draw frame");
        if handle_events(&widgets, &mut focused_area, &mut app_state)? {
            break;
        }
    }
    ratatui::restore();
    if let Err(err) = execute!(stdout(), DisableMouseCapture) {
        eprintln!("Error disabling mouse capture: {err}");
    }
    Ok(())
}


fn draw(frame: &mut Frame, focused_area: FocusedArea, widgets: &WidgetsArea, app_state: &mut AppState) {
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
    
    // Render checkpoints list with scrollbar
    let checkpoints_block = Block::default()
        .title("Checkpoints")
        .borders(Borders::ALL)
        .border_style(if focused_area == FocusedArea::Checkpoints { focused_border_style } else { default_border_style });
    
    let inner_area = checkpoints_block.inner(checkpoints_area);
    frame.render_widget(checkpoints_block, checkpoints_area);
    
    // Create list items from checkpoints
    let items: Vec<ListItem> = app_state.checkpoints
        .iter()
        .map(|checkpoint| {
            ListItem::new(format!("{}  {}  {}  {}", 
                checkpoint.checkpoint_id[..7].to_string(),
                checkpoint.tag,
                checkpoint.pid,
                checkpoint.dump_time,
            ))
        })
        .collect();
    
    // Create the list widget
    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(" ");
    
    // Render the list with state
    frame.render_stateful_widget(list, inner_area, &mut app_state.checkpoints_state);
    
    // Render scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    
    frame.render_stateful_widget(
        scrollbar,
        inner_area,
        &mut app_state.scrollbar_state,
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

fn handle_events(widgets: &WidgetsArea, focused_area: &mut FocusedArea, app_state: &mut AppState) -> std::io::Result<bool> {
    match event::read()? {
        Event::Mouse(mouse_event) => {
            handle_mouse_events(mouse_event, focused_area, widgets);
        }
        Event::Key(key) if key.kind == KeyEventKind::Press =>  {
            return Ok(handle_key_events(key.code, focused_area, app_state));
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

fn handle_key_events(key: KeyCode, focused_area: &mut FocusedArea, app_state: &mut AppState) -> bool {
    match key {
        KeyCode::Char('q') => {
            // Exit the application
            return true;
        }
        KeyCode::Up => {
            if *focused_area == FocusedArea::Checkpoints {
                app_state.previous();
            }
        }
        KeyCode::Down => {
            if *focused_area == FocusedArea::Checkpoints {
                app_state.next();
            }
        }
        _ => {}
    }
    return false;
}

// Application state to manage the list state and scrollbar
struct AppState {
    checkpoints: Vec<CheckpointMeta>,
    checkpoints_state: ListState,
    scrollbar_state: ScrollbarState,
}

impl AppState {
    fn new(checkpoints: Vec<CheckpointMeta>) -> Self {
        let mut state = Self {
            checkpoints,
            checkpoints_state: ListState::default(),
            scrollbar_state: ScrollbarState::default(),
        };
        
        // Initialize with first item selected if list is not empty
        if !state.checkpoints.is_empty() {
            state.checkpoints_state.select(Some(0));
            state.scrollbar_state = state.scrollbar_state.position(0);
        }
        
        state
    }
    
    fn next(&mut self) {
        let i = match self.checkpoints_state.selected() {
            Some(i) => {
                if i >= self.checkpoints.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.checkpoints_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }
    
    fn previous(&mut self) {
        let i = match self.checkpoints_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.checkpoints.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.checkpoints_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }
}
