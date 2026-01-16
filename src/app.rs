use ansi_to_tui::IntoText as _;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;
use ratatui::{
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};

use crate::config::Config;
use crate::fuzzy::FuzzyMatcher;
use crate::git::{ChangedFile, FileStatus, Repository};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    FileList,
    DiffView,
}

pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub files: Vec<ChangedFile>,
    pub file_paths: Vec<String>,
    pub filtered_indices: Vec<usize>,
    pub list_state: ListState,
    pub search_mode: bool,
    pub search_query: String,
    pub fuzzy_matcher: FuzzyMatcher,
    pub diff_content: Vec<u8>,
    pub diff_lines: Vec<Line<'static>>,
    pub diff_scroll: u16,
    pub selected_file: Option<String>,
    pub config: Config,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load();
        let repository = Repository::open_current_dir()?;
        let files = repository.get_changed_files()?;
        let file_paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
        let filtered_indices: Vec<usize> = (0..files.len()).collect();

        let mut list_state = ListState::default();
        if !files.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            running: true,
            screen: Screen::FileList,
            files,
            file_paths,
            filtered_indices,
            list_state,
            search_mode: false,
            search_query: String::new(),
            fuzzy_matcher: FuzzyMatcher::new(),
            diff_content: Vec::new(),
            diff_lines: Vec::new(),
            diff_scroll: 0,
            selected_file: None,
            config,
        })
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::FileList => self.draw_file_list(frame),
            Screen::DiffView => self.draw_diff_view(frame),
        }
    }

    fn draw_file_list(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let constraints = if self.search_mode {
            vec![
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ]
        } else {
            vec![Constraint::Min(1), Constraint::Length(1)]
        };

        let chunks = Layout::vertical(constraints).split(area);

        let (list_area, help_area) = if self.search_mode {
            // Draw search input
            let search_block = Block::default().title(" Search ").borders(Borders::ALL);
            let search_input = Paragraph::new(self.search_query.as_str()).block(search_block);
            frame.render_widget(search_input, chunks[0]);

            // Set cursor position
            frame.set_cursor_position(Position::new(
                chunks[0].x + self.search_query.len() as u16 + 1,
                chunks[0].y + 1,
            ));

            (chunks[1], chunks[2])
        } else {
            (chunks[0], chunks[1])
        };

        // Build list items from filtered indices
        let items: Vec<ListItem> = self
            .filtered_indices
            .iter()
            .filter_map(|&idx| self.files.get(idx))
            .map(|file| {
                let status_char = match file.status {
                    FileStatus::Modified => ("M", Color::Yellow),
                    FileStatus::Added => ("A", Color::Green),
                    FileStatus::Deleted => ("D", Color::Red),
                    FileStatus::Renamed => ("R", Color::Cyan),
                    FileStatus::Untracked => ("?", Color::Gray),
                };
                let line = Line::from(vec![
                    Span::styled(
                        format!("{} ", status_char.0),
                        Style::default().fg(status_char.1),
                    ),
                    Span::raw(&file.path),
                ]);
                ListItem::new(line)
            })
            .collect();

        let title = format!(
            " Changed Files ({}/{}) ",
            self.filtered_indices.len(),
            self.files.len()
        );
        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, list_area, &mut self.list_state);

        let help_text = if self.search_mode {
            " Type to search | Enter: select | Esc: cancel "
        } else {
            " j/k: move | Enter: view diff | /: search | q: quit "
        };
        let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, help_area);
    }

    fn draw_diff_view(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);

        let title = format!(
            " {} ",
            self.selected_file.as_deref().unwrap_or("Diff")
        );

        let visible_height = chunks[0].height.saturating_sub(2) as usize;
        let visible_lines: Vec<Line> = self
            .diff_lines
            .iter()
            .skip(self.diff_scroll as usize)
            .take(visible_height)
            .cloned()
            .collect();

        let diff = Paragraph::new(visible_lines)
            .block(Block::default().title(title).borders(Borders::ALL));

        frame.render_widget(diff, chunks[0]);

        let total_lines = self.diff_lines.len();
        let current_line = self.diff_scroll as usize + 1;
        let help = Paragraph::new(format!(
            " j/k: scroll | q: back | Line {}/{} ",
            current_line.min(total_lines),
            total_lines
        ))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[1]);
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            match self.screen {
                Screen::FileList => self.handle_file_list_keys(key.code),
                Screen::DiffView => self.handle_diff_view_keys(key.code),
            }
        }
        Ok(())
    }

    fn handle_file_list_keys(&mut self, code: KeyCode) {
        if self.search_mode {
            match code {
                KeyCode::Esc => {
                    self.search_mode = false;
                    self.search_query.clear();
                    self.update_filter();
                }
                KeyCode::Enter => {
                    self.search_mode = false;
                    if !self.filtered_indices.is_empty() {
                        self.open_diff();
                    }
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.update_filter();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.update_filter();
                }
                KeyCode::Down => self.select_next(),
                KeyCode::Up => self.select_previous(),
                _ => {}
            }
        } else {
            match code {
                KeyCode::Char('q') => self.running = false,
                KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('/') => {
                    self.search_mode = true;
                }
                KeyCode::Enter => self.open_diff(),
                _ => {}
            }
        }
    }

    fn handle_diff_view_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.screen = Screen::FileList;
                self.diff_scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let max_scroll = self.diff_lines.len().saturating_sub(1);
                self.diff_scroll = (self.diff_scroll + 1).min(max_scroll as u16);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.diff_scroll = self.diff_scroll.saturating_sub(1);
            }
            KeyCode::Char('d') | KeyCode::PageDown => {
                let max_scroll = self.diff_lines.len().saturating_sub(1);
                self.diff_scroll = (self.diff_scroll + 20).min(max_scroll as u16);
            }
            KeyCode::Char('u') | KeyCode::PageUp => {
                self.diff_scroll = self.diff_scroll.saturating_sub(20);
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.diff_scroll = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.diff_scroll = self.diff_lines.len().saturating_sub(1) as u16;
            }
            _ => {}
        }
    }

    fn select_next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.filtered_indices.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn update_filter(&mut self) {
        self.filtered_indices = self.fuzzy_matcher.filter(&self.file_paths, &self.search_query);
        // Reset selection to first item if there are results
        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    fn open_diff(&mut self) {
        if let Some(list_idx) = self.list_state.selected() {
            if let Some(&file_idx) = self.filtered_indices.get(list_idx) {
                if let Some(file) = self.files.get(file_idx) {
                    self.selected_file = Some(file.path.clone());
                    // Get terminal width (subtract 2 for border)
                    let width = terminal::size().map(|(w, _)| w.saturating_sub(2)).unwrap_or(80);
                    self.diff_content = crate::git::get_diff(&file.path, width, &self.config.diff);

                    // Parse ANSI escape sequences into styled lines
                    self.diff_lines = match self.diff_content.as_slice().into_text() {
                        Ok(text) => text
                            .lines
                            .into_iter()
                            .map(|line| {
                                Line::from(
                                    line.spans
                                        .into_iter()
                                        .map(|span| {
                                            Span::styled(span.content.to_string(), span.style)
                                        })
                                        .collect::<Vec<_>>(),
                                )
                            })
                            .collect(),
                        Err(_) => {
                            // Fallback: plain text without ANSI parsing
                            String::from_utf8_lossy(&self.diff_content)
                                .lines()
                                .map(|s| Line::raw(s.to_string()))
                                .collect()
                        }
                    };

                    self.diff_scroll = 0;
                    self.screen = Screen::DiffView;
                }
            }
        }
    }
}
