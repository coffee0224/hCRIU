use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;
use rust_criu::Criu;
use which::which;

use hcriu::utils::{CheckpointMeta, get_all_checkpoints, set_hcriu_dir, get_hcriu_dir};
use hcriu::restore::handle_restore;

fn find_criu_path() -> Option<String> {
  which("criu").ok().map(|p| p.to_string_lossy().into_owned())
}


/**
 * Get information about all processes in the system
 * 
 * Returns a vector of ProcessInfo structs containing PID, name, and command line
 */
fn get_all_processes() -> Vec<ProcessInfo> {
    let mut processes = Vec::new();
    if let Ok(all_procs) = procfs::process::all_processes() {
        for proc_result in all_procs {
            if let Ok(process) = proc_result {
                if let Ok(status) = process.status() {
                    let name = status.name;
                    let cmd = process.cmdline().unwrap_or_default().join(" ");
                    processes.push(ProcessInfo {
                        pid: process.pid,
                        name,
                        cmd,
                    });
                }
            }
        }
    }
    processes
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let widgets = WidgetsArea::new(&terminal.get_frame());
    
    // Initialize hcriu directory
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let hcriu_dir = home_dir.join(".hcriu");
    set_hcriu_dir(hcriu_dir);
    
    // Initialize app state
    let checkpoints = get_all_checkpoints();
    let mut app_state = AppState::new(checkpoints);
    
    // Initial process list load
    app_state.processes = get_all_processes();
    if !app_state.processes.is_empty() {
        app_state.processes_state.select(Some(0));
        app_state.processes_scrollbar_state = app_state.processes_scrollbar_state.position(0);
    }
    
    loop {
        // Update process list if interval has elapsed
        if app_state.last_update.elapsed() >= app_state.update_interval {
            app_state.processes = get_all_processes();
            app_state.last_update = Instant::now();
        }
        
        terminal.draw(|f| draw(f, &widgets, &mut app_state)).expect("failed to draw frame");
        if handle_events(&widgets, &mut app_state)? {
            break;
        }
    }
    ratatui::restore();
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
            ListItem::new(format!("{} {} {} {}", 
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
    
    // Render processes list with scrollbar
    let processes_block = Block::default()
        .title("Processes")
        .borders(Borders::ALL)
        .border_style(if app_state.focused_area == FocusedArea::Processes { focused_border_style } else { default_border_style });
    
    let processes_inner_area = processes_block.inner(processes_area);
    frame.render_widget(processes_block, processes_area);
    
    // Create list items from processes
    let process_items: Vec<ListItem> = app_state.processes
        .iter()
        .map(|process| {
            ListItem::new(format!("{:<6} {:<15} {}", 
                process.pid,
                process.name,
                process.cmd,
            ))
        })
        .collect();
    
    // Create the list widget for processes
    let processes_list = List::new(process_items)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(" ");
    
    // Render the processes list with state
    frame.render_stateful_widget(processes_list, processes_inner_area, &mut app_state.processes_state);
    
    // Render processes scrollbar
    let processes_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    
    frame.render_stateful_widget(
        processes_scrollbar,
        processes_inner_area,
        &mut app_state.processes_scrollbar_state,
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
    let popup_block = Block::default()
        .title("Menu")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White));
    
    frame.render_widget(&popup_block, popup_area);
    
    // Create menu items based on popup type
    let menu_items = match app_state.popup_type {
        PopupType::Checkpoint => vec![
            "r restore checkpoint to process",
            "d delete checkpoint",
        ],
        PopupType::Process => vec![
            "s take a snapshot and stop",
            "l take a snapshot and leave running",
            "p take snapshots periodly",
        ],
    };
    
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
        Event::Key(key) => {
            return Ok(handle_key_events(key.code, key.modifiers, app_state));
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

fn handle_key_events(key: KeyCode, _modifiers: KeyModifiers, app_state: &mut AppState) -> bool {
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
                app_state.popup_type = PopupType::Checkpoint;
                app_state.show_popup = true;
                app_state.popup_state.select(Some(0));
            } else if app_state.focused_area == FocusedArea::Processes && !app_state.processes.is_empty() {
                app_state.popup_type = PopupType::Process;
                app_state.show_popup = true;
                app_state.popup_state.select(Some(0));
            }
        }
        KeyCode::Up => {
            match app_state.focused_area {
                FocusedArea::Checkpoints => app_state.checkpoints_previous(),
                FocusedArea::Processes => app_state.processes_previous(),
                _ => {}
            }
        }
        KeyCode::Down => {
            match app_state.focused_area {
                FocusedArea::Checkpoints => app_state.checkpoints_next(),
                FocusedArea::Processes => app_state.processes_next(),
                _ => {}
            }
        }
        KeyCode::Tab => {
            // Cycle through areas: Checkpoints -> Processes -> Tasks -> Checkpoints
            match app_state.focused_area {
                FocusedArea::Checkpoints => app_state.set_focused_area(FocusedArea::Processes),
                FocusedArea::Processes => app_state.set_focused_area(FocusedArea::Tasks),
                FocusedArea::Tasks => app_state.set_focused_area(FocusedArea::Checkpoints),
                _ => app_state.set_focused_area(FocusedArea::Checkpoints),
            }
        }
        KeyCode::BackTab => {
            // Reverse cycle: Checkpoints -> Tasks -> Processes -> Checkpoints
            match app_state.focused_area {
                FocusedArea::Checkpoints => app_state.set_focused_area(FocusedArea::Tasks),
                FocusedArea::Tasks => app_state.set_focused_area(FocusedArea::Processes),
                FocusedArea::Processes => app_state.set_focused_area(FocusedArea::Checkpoints),
                _ => app_state.set_focused_area(FocusedArea::Checkpoints),
            }
        }
        _ => {}
    }
    return false;
}

fn handle_popup_action(app_state: &mut AppState) {
    match app_state.popup_type {
        PopupType::Checkpoint => {
            if let Some(selected_checkpoint_idx) = app_state.checkpoints_state.selected() {
                if selected_checkpoint_idx < app_state.checkpoints.len() {
                    let checkpoint = &app_state.checkpoints[selected_checkpoint_idx];
                    
                    match app_state.popup_state.selected() {
                        Some(0) => {
                            // Restore checkpoint
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
                            // Delete checkpoint
                            let checkpoint_dir = get_hcriu_dir().join(checkpoint.checkpoint_id.clone());
                            std::fs::remove_dir_all(&checkpoint_dir).unwrap();
                        }
                        _ => {}
                    }
                }
            }
        },
        PopupType::Process => {
            if let Some(selected_process_idx) = app_state.processes_state.selected() {
                if selected_process_idx < app_state.processes.len() {
                    let process = &app_state.processes[selected_process_idx];
                    
                    match app_state.popup_state.selected() {
                        Some(0) => {
                            // Take a snapshot and stop
                            println!("Taking snapshot of process {} and stopping it", process.pid);
                            // TODO: Implement actual snapshot and stop functionality
                        }
                        Some(1) => {
                            // Take a snapshot and leave running
                            println!("Taking snapshot of process {} and leaving it running", process.pid);
                            // TODO: Implement actual snapshot functionality
                        }
                        Some(2) => {
                            // Take snapshots periodically
                            println!("Taking periodic snapshots of process {}", process.pid);
                            // TODO: Implement periodic snapshot functionality
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// Process information structure
struct ProcessInfo {
    pid: i32,
    name: String,
    cmd: String,
}

// Popup menu type
#[derive(PartialEq, Debug, Copy, Clone)]
enum PopupType {
    Checkpoint,
    Process,
}

// Application state to manage the list state and scrollbar
struct AppState {
    focused_area: FocusedArea,
    last_update: Instant,
    update_interval: Duration,
    // checkpoint widget
    checkpoints: Vec<CheckpointMeta>,
    checkpoints_state: ListState,
    scrollbar_state: ScrollbarState,

    // process widget
    processes: Vec<ProcessInfo>,
    processes_state: ListState,
    processes_scrollbar_state: ScrollbarState,

    // popup widget
    show_popup: bool,
    popup_state: ListState,
    popup_type: PopupType,
}

impl AppState {
    fn new(checkpoints: Vec<CheckpointMeta>) -> Self {
        let mut state = Self {
            checkpoints,
            checkpoints_state: ListState::default(),
            scrollbar_state: ScrollbarState::default(),
            processes: Vec::new(),
            processes_state: ListState::default(),
            processes_scrollbar_state: ScrollbarState::default(),
            show_popup: false,
            popup_state: ListState::default(),
            popup_type: PopupType::Checkpoint,
            focused_area: FocusedArea::Checkpoints,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1),
        };
        
        // Initialize with first item selected if list is not empty
        if !state.checkpoints.is_empty() {
            state.checkpoints_state.select(Some(0));
            state.scrollbar_state = state.scrollbar_state.position(0);
        }
        
        state
    }
    
    fn checkpoints_next(&mut self) {
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
    
    fn checkpoints_previous(&mut self) {
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
    
    fn processes_next(&mut self) {
        let i = match self.processes_state.selected() {
            Some(i) => {
                if i >= self.processes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.processes_state.select(Some(i));
        self.processes_scrollbar_state = self.processes_scrollbar_state.position(i);
    }
    
    fn processes_previous(&mut self) {
        let i = match self.processes_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.processes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.processes_state.select(Some(i));
        self.processes_scrollbar_state = self.processes_scrollbar_state.position(i);
    }
}
