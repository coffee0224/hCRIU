use std::{io::stdout, ops::ControlFlow, time::Duration};
use std::path::PathBuf;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind, MouseButton, MouseEvent};
use crossterm::execute;
use ratatui::{
    layout::{Constraint, Layout, Position, Rect, Alignment, Direction},
    widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, Paragraph, Clear},
    style::{Style, Color, Modifier},
    text::{Line, Span},
    Frame,
};
use rust_criu::Criu;
use which::which;

use hcriu::utils::{CheckpointMeta, get_all_checkpoints, set_hcriu_dir, get_hcriu_dir};
use hcriu::restore::handle_restore;

fn find_criu_path() -> Option<String> {
  which("criu").ok().map(|p| p.to_string_lossy().into_owned())
}

struct RestorePopup {
    checkpoint_id: String,
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    execute!(stdout(), EnableMouseCapture)?;
    let widgets = WidgetsArea::new(&terminal.get_frame());
    
    // Initialize hcriu directory
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let hcriu_dir = home_dir.join(".hcriu");
    set_hcriu_dir(hcriu_dir);
    
    // Initialize app state
    let checkpoints = get_all_checkpoints();
    let mut app_state = AppState::new(checkpoints);
    
    loop {
        terminal.draw(|f| draw(f, &widgets, &mut app_state)).expect("failed to draw frame");
        if handle_events(&widgets, &mut app_state)? {
            break;
        }
    }
    ratatui::restore();
    if let Err(err) = execute!(stdout(), DisableMouseCapture) {
        eprintln!("Error disabling mouse capture: {err}");
    }
    Ok(())
}


fn draw(frame: &mut Frame, widgets: &WidgetsArea, app_state: &mut AppState) {
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
            .border_style(if app_state.focused_area == FocusedArea::Title { focused_border_style } else { default_border_style }),
        title_area,
    );
    
    // Render checkpoints list with scrollbar
    let checkpoints_block = Block::default()
        .title("Checkpoints")
        .borders(Borders::ALL)
        .border_style(if app_state.focused_area == FocusedArea::Checkpoints { focused_border_style } else { default_border_style });
    
    let inner_area = checkpoints_block.inner(checkpoints_area);
    frame.render_widget(checkpoints_block, checkpoints_area);
    
    // Create list items from checkpoints
    let items: Vec<ListItem> = app_state.checkpoints
        .iter()
        .map(|checkpoint| {
            ListItem::new(format!("{}  {} {} {}", 
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
            .border_style(if app_state.focused_area == FocusedArea::Tasks { focused_border_style } else { default_border_style }),
        tasks_area,
    );
    frame.render_widget(
        Block::default()
            .title("Processes")
            .borders(Borders::ALL)
            .border_style(if app_state.focused_area == FocusedArea::Processes { focused_border_style } else { default_border_style }),
        processes_area,
    );
    frame.render_widget(
        Block::default()
            .borders(Borders::empty()),
        status_area,
    );
    
    // Render popup if it's active
    if app_state.show_popup {
        draw_popup(frame, app_state);
    }
}

fn draw_popup(frame: &mut Frame, app_state: &mut AppState) {
    let area = frame.area();
    
    // Calculate popup size and position
    let popup_width = 40;
    let popup_height = 6;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);
    
    // Render popup background
    frame.render_widget(Clear, popup_area);
    
    // Render popup block
    let popup_block = Block::default()
        .title("Menu")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White));
    
    frame.render_widget(&popup_block, popup_area);
    
    // Create menu items
    let menu_items = vec![
        "r restore checkpoint to process",
        "d delete checkpoint",
    ];
    
    // Create list items
    let items: Vec<ListItem> = menu_items
        .iter()
        .map(|item| ListItem::new(*item))
        .collect();
    
    // Create list widget
    let inner_area = popup_block.inner(popup_area);
    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(" ");
    
    // Render list with state
    frame.render_stateful_widget(list, inner_area, &mut app_state.popup_state);
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

fn handle_events(widgets: &WidgetsArea, app_state: &mut AppState) -> std::io::Result<bool> {
    match event::read()? {
        Event::Mouse(mouse_event) => {
            handle_mouse_events(mouse_event, app_state, widgets);
        }
        Event::Key(key) if key.kind == KeyEventKind::Press =>  {
            return Ok(handle_key_events(key.code, app_state));
        },
        _ => {}
    }
    Ok(false)
}

fn handle_mouse_events(
    mouse_event: MouseEvent,
    app_state: &mut AppState,
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
                app_state.set_focused_area(FocusedArea::Checkpoints);
            } else if tasks_area.contains(mouse_pos) {
                app_state.set_focused_area(FocusedArea::Tasks);
            } else if processes_area.contains(mouse_pos) {
                app_state.set_focused_area(FocusedArea::Processes);
            }
        }
        _ => {}
    }
}

fn handle_key_events(key: KeyCode, app_state: &mut AppState) -> bool {
    // If popup is active, handle popup navigation
    if app_state.show_popup {
        match key {
            KeyCode::Esc => {
                app_state.show_popup = false;
            }
            KeyCode::Up => {
                app_state.popup_previous();
            }
            KeyCode::Down => {
                app_state.popup_next();
            }
            KeyCode::Enter => {
                handle_popup_action(app_state);
                app_state.show_popup = false;
            }
            _ => {}
        }
        return false;
    }
    
    // Normal key handling
    match key {
        KeyCode::Char('q') => {
            // Exit the application
            return true;
        }
        KeyCode::Char('x') => {
            if app_state.focused_area == FocusedArea::Checkpoints && !app_state.checkpoints.is_empty() {
                app_state.show_popup = true;
                app_state.popup_state.select(Some(0));
            }
        }
        KeyCode::Up => {
            if app_state.focused_area == FocusedArea::Checkpoints {
                app_state.previous();
            }
        }
        KeyCode::Down => {
            if app_state.focused_area == FocusedArea::Checkpoints {
                app_state.next();
            }
        }
        _ => {}
    }
    return false;
}

fn handle_popup_action(app_state: &mut AppState) {
    if let Some(selected_checkpoint_idx) = app_state.checkpoints_state.selected() {
        if selected_checkpoint_idx < app_state.checkpoints.len() {
            let checkpoint = &app_state.checkpoints[selected_checkpoint_idx];
            
            match app_state.popup_state.selected() {
                Some(0) => {
                    let path = match find_criu_path() {
      Some(path) => path,
      None => {
        eprintln!("criu not found in PATH, please specify --criu-path");
        std::process::exit(1);
      }
    };
                    let mut criu = Criu::new_with_criu_path(path).unwrap();
                    handle_restore(&mut criu, checkpoint.checkpoint_id.clone());

                }
                Some(1) => {
                    let checkpoint_dir = get_hcriu_dir().join(checkpoint.checkpoint_id.clone());
                    std::fs::remove_dir_all(&checkpoint_dir).unwrap();
                }
                _ => {}
            }
        }
    }
}

// Application state to manage the list state and scrollbar
struct AppState {
    checkpoints: Vec<CheckpointMeta>,
    checkpoints_state: ListState,
    scrollbar_state: ScrollbarState,
    show_popup: bool,
    popup_state: ListState,
    focused_area: FocusedArea,
}

impl AppState {
    fn new(checkpoints: Vec<CheckpointMeta>) -> Self {
        let mut state = Self {
            checkpoints,
            checkpoints_state: ListState::default(),
            scrollbar_state: ScrollbarState::default(),
            show_popup: false,
            popup_state: ListState::default(),
            focused_area: FocusedArea::Checkpoints,
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
    
    fn popup_next(&mut self) {
        let i = match self.popup_state.selected() {
            Some(i) => {
                if i >= 1 { // Only 2 options in popup
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.popup_state.select(Some(i));
    }
    
    fn popup_previous(&mut self) {
        let i = match self.popup_state.selected() {
            Some(i) => {
                if i == 0 {
                    1 // Only 2 options in popup
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.popup_state.select(Some(i));
    }
    
    fn set_focused_area(&mut self, area: FocusedArea) {
        self.focused_area = area;
    }
}
