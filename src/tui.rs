use crate::utils::{CheckpointMeta, get_all_checkpoints};
use ratatui::{
  DefaultTerminal, Frame, Terminal,
  backend::CrosstermBackend,
  crossterm::event::{self, Event, KeyCode, KeyEventKind},
  layout::{Constraint, Layout, Position},
  style::{Color, Modifier, Style, Stylize},
  text::{Line, Span, Text},
  widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};
use std::{io, vec};

fn get_checkoutpoints_list(checkpoints: Vec<&CheckpointMeta>) -> Vec<ListItem> {
  checkpoints
    .iter()
    .map(|c| {
      ListItem::from(Span::raw(format!(
        "{}: {} ({} - {})",
        c.checkpoint_id, c.tag, c.pid, c.dump_time
      )))
    })
    .collect()
}

fn merge_checkpoints_message<'a>(
  checkpoints: Vec<&'a CheckpointMeta>,
  mut parts: std::str::SplitWhitespace<'_>,
) -> Vec<ListItem<'a>> {
  let tag = match parts.next() {
    Some(tag) => tag.to_string(),
    None => {
      return vec![ListItem::new(Line::from(Span::raw(
        "Please provide a tag for the merge command.",
      )))];
    }
  };
  let keep_daily = true; // default to true
  let keep_hourly = false; // default to false

  let mut filtered_checkpoints: Vec<&CheckpointMeta> = checkpoints
    .iter()
    .copied()
    .filter(|c| c.tag == tag)
    .collect();

  filtered_checkpoints.sort_by(|a, b| a.dump_time.cmp(&b.dump_time));

  // filter checkpoints by time
  let keep_checkpoints: Vec<&CheckpointMeta> = if keep_daily {
    // keep the latest checkpoint of each day
    let mut daily_checkpoints = Vec::new();
    let mut current_day = String::new();
    for checkpoint in filtered_checkpoints.iter().rev() {
      let day = checkpoint.dump_time.split(' ').next().unwrap();
      if day != current_day {
        daily_checkpoints.push(*checkpoint);
        current_day = day.to_string();
      }
    }
    daily_checkpoints
  } else if keep_hourly {
    // keep the latest checkpoint of each hour
    let mut hourly_checkpoints = Vec::new();
    let mut current_hour = String::new();
    for checkpoint in filtered_checkpoints.iter().rev() {
      let hour = checkpoint
        .dump_time
        .split(' ')
        .nth(1)
        .unwrap()
        .split(':')
        .next()
        .unwrap();
      if hour != current_hour {
        hourly_checkpoints.push(*checkpoint);
        current_hour = hour.to_string();
      }
    }
    hourly_checkpoints
  } else {
    // keep only the latest checkpoint
    filtered_checkpoints
      .iter()
      .max_by_key(|c| &c.dump_time)
      .map(|c| vec![*c])
      .unwrap_or_default()
  };

  if keep_checkpoints.is_empty() {
    return vec![ListItem::new(Line::from(Span::raw(
      "No checkpoints found for the given tag and pid.",
    )))];
  }

  let merged_checkpoints: Vec<&CheckpointMeta> = checkpoints
    .into_iter()
    .filter(|c| !keep_checkpoints.contains(c))
    .collect();

  let mut items = vec![];
  if let Some("dry-run") = parts.next() {
    items.extend(vec![ListItem::new(Line::from(Span::raw(
      "The following checkpoints will be merged:",
    )))]);
    items.extend(get_checkoutpoints_list(merged_checkpoints));
    items.extend(vec![ListItem::new(Line::from(Span::raw(
      "The following checkpoints will be kept:",
    )))]);
    items.extend(get_checkoutpoints_list(keep_checkpoints));
    return items;
  }

  merged_checkpoints.iter().for_each(|c| {
    // delete the checkpoint
    let checkpoint_dir = crate::utils::get_hcriu_dir().join(c.checkpoint_id.clone());
    std::fs::remove_dir_all(&checkpoint_dir).unwrap();
    items.extend(vec![ListItem::new(Line::from(Span::raw(format!(
      "Deleted checkpoint {}",
      c.checkpoint_id
    ))))]);
  });
  items.extend(vec![ListItem::new(Line::from(Span::raw(format!(
    "Merged {:?} checkpoints",
    merged_checkpoints.len()
  ))))]);
  items
}

pub fn show_checkpoints_tui(checkpoints: Vec<&CheckpointMeta>) {
  let mut stdout = io::stdout();
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

pub fn interactive_tui() -> Result<(), io::Error> {
  let terminal = ratatui::init();
  let app_result = App::new().run(terminal);
  ratatui::restore();
  app_result
}

/// App holds the state of the application
struct App {
  /// Current value of the input box
  input: String,
  /// Position of cursor in the editor area.
  character_index: usize,
  /// Current input mode
  input_mode: InputMode,
  /// History of recorded messages
  message: String,
}

enum InputMode {
  Normal,
  Editing,
}

impl App {
  const fn new() -> Self {
    Self {
      input: String::new(),
      input_mode: InputMode::Normal,
      message: String::new(),
      character_index: 0,
    }
  }

  fn move_cursor_left(&mut self) {
    let cursor_moved_left = self.character_index.saturating_sub(1);
    self.character_index = self.clamp_cursor(cursor_moved_left);
  }

  fn move_cursor_right(&mut self) {
    let cursor_moved_right = self.character_index.saturating_add(1);
    self.character_index = self.clamp_cursor(cursor_moved_right);
  }

  fn enter_char(&mut self, new_char: char) {
    let index = self.byte_index();
    self.input.insert(index, new_char);
    self.move_cursor_right();
  }

  /// Returns the byte index based on the character position.
  ///
  /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
  /// the byte index based on the index of the character.
  fn byte_index(&self) -> usize {
    self
      .input
      .char_indices()
      .map(|(i, _)| i)
      .nth(self.character_index)
      .unwrap_or(self.input.len())
  }

  fn delete_char(&mut self) {
    let is_not_cursor_leftmost = self.character_index != 0;
    if is_not_cursor_leftmost {
      // Method "remove" is not used on the saved text for deleting the selected char.
      // Reason: Using remove on String works on bytes instead of the chars.
      // Using remove would require special care because of char boundaries.

      let current_index = self.character_index;
      let from_left_to_current_index = current_index - 1;

      // Getting all characters before the selected character.
      let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
      // Getting all characters after selected character.
      let after_char_to_delete = self.input.chars().skip(current_index);

      // Put all characters together except the selected one.
      // By leaving the selected one out, it is forgotten and therefore deleted.
      self.input = before_char_to_delete.chain(after_char_to_delete).collect();
      self.move_cursor_left();
    }
  }

  fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
    new_cursor_pos.clamp(0, self.input.chars().count())
  }

  fn reset_cursor(&mut self) {
    self.character_index = 0;
  }

  fn submit_message(&mut self) {
    self.message = self.input.clone();
    self.input.clear();
    self.reset_cursor();
  }

  fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), io::Error> {
    loop {
      terminal.draw(|frame| self.draw(frame))?;

      if let Event::Key(key) = event::read()? {
        match self.input_mode {
          InputMode::Normal => match key.code {
            KeyCode::Char('e') => {
              self.input_mode = InputMode::Editing;
            }
            KeyCode::Char('q') => {
              return Ok(());
            }
            _ => {}
          },
          InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Enter => self.submit_message(),
            KeyCode::Char(to_insert) => self.enter_char(to_insert),
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Esc => self.input_mode = InputMode::Normal,
            _ => {}
          },
          InputMode::Editing => {}
        }
      }
    }
  }

  fn draw(&self, frame: &mut Frame) {
    let vertical = Layout::vertical([
      Constraint::Length(1),
      Constraint::Length(3),
      Constraint::Min(1),
    ]);
    let [help_area, input_area, messages_area] = vertical.areas(frame.area());

    let (msg, style) = match self.input_mode {
      InputMode::Normal => (
        vec![
          "Press ".into(),
          "q".bold(),
          " to exit, ".into(),
          "e".bold(),
          " to start editing.".bold(),
        ],
        Style::default().add_modifier(Modifier::RAPID_BLINK),
      ),
      InputMode::Editing => (
        vec![
          "Press ".into(),
          "Esc".bold(),
          " to stop editing, ".into(),
          "Enter".bold(),
          " to submit the command. ".into(),
          "Commands: ".into(),
          "list, dump, merge.".bold(),
        ],
        Style::default(),
      ),
    };
    let text = Text::from(Line::from(msg)).patch_style(style);
    let help_message = Paragraph::new(text);
    frame.render_widget(help_message, help_area);

    let input = Paragraph::new(self.input.as_str())
      .style(match self.input_mode {
        InputMode::Normal => Style::default(),
        InputMode::Editing => Style::default().fg(Color::Yellow),
      })
      .block(Block::bordered().title("Input"));
    frame.render_widget(input, input_area);
    match self.input_mode {
      // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
      InputMode::Normal => {}

      // Make the cursor visible and ask ratatui to put it at the specified coordinates after
      // rendering
      #[allow(clippy::cast_possible_truncation)]
      InputMode::Editing => frame.set_cursor_position(Position::new(
        // Draw the cursor at the current position in the input field.
        // This position is can be controlled via the left and right arrow key
        input_area.x + self.character_index as u16 + 1,
        // Move one line down, from the border to the input line
        input_area.y + 1,
      )),
    }

    let mut parts = self.message.split_whitespace();
    // Store all checkpoints in a variable so references live long enough
    let all_checkpoints = get_all_checkpoints();
    let checkpoints: Vec<&CheckpointMeta> = all_checkpoints.iter().collect();
    let message: Vec<ListItem> = match parts.next() {
      Some("list") => get_checkoutpoints_list(checkpoints.clone()),
      Some("merge") => merge_checkpoints_message(checkpoints.clone(), parts),
      _ => vec![],
    };
    let message = List::new(message).block(Block::bordered().title("Output"));
    frame.render_widget(message, messages_area);
  }
}
