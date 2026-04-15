use std::io::{Stdout, Write, stdout};

use anyhow::{Result, anyhow};
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Stylize;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Padding, Paragraph};
use ratatui::{Terminal, TerminalOptions, Viewport};

use crate::config::{CommandConfig, ProjectConfig};
use crate::scan::Dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootOption {
    pub label: String,
    pub value: String,
    pub is_manual: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WorkspaceChoice {
    CurrentDir,
    Project,
    BrowseRoot,
    NewRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceResult {
    pub choice: WorkspaceChoice,
    pub project_path: String,
}

pub fn print_banner(writer: &mut dyn Write, version: &str) -> Result<()> {
    const BANNER_ART: &str = "  _______ ____  _____   _____ ______ \n |  _____/ __ \\|  __ \\ / ____|  ____|\n | |__  | |  | | |__) | |  __| |__   \n |  __| | |  | |  _  /| | |_ |  __|  \n | |    | |__| | | \\ \\| |__| | |____ \n |_|     \\____/|_|  \\_\\\\_____|______|";

    let colors = ["Cyan", "Cyan", "Blue", "Blue", "Magenta", "Magenta"];
    for (index, line) in BANNER_ART.lines().enumerate() {
        let rendered = match colors.get(index).copied().unwrap_or("Magenta") {
            "Cyan" => line.cyan().bold().to_string(),
            "Blue" => line.blue().bold().to_string(),
            _ => line.magenta().bold().to_string(),
        };
        writeln!(writer, "{rendered}")?;
    }

    let mut info = "  Forge — 快速启动你的 AI 编程助手"
        .dark_grey()
        .italic()
        .to_string();
    if !version.is_empty() && version != "dev" {
        info.push_str("  ");
        info.push_str(&format!("[v{version}]").cyan().bold().to_string());
    }
    writeln!(writer, "{info}")?;
    writeln!(writer, "{}", "─".repeat(42).dark_grey())?;
    writeln!(writer)?;
    Ok(())
}

pub fn step_line(step: &str, title: &str) -> String {
    format!("{}  {}", step.dark_grey().bold(), title.dark_grey())
}

pub fn selected_line(label: &str) -> String {
    format!("  ✓ 已选择: {label}").green().bold().to_string()
}

pub fn subtitle_line(value: &str) -> String {
    format!("    {value}").dark_grey().to_string()
}

pub fn select_workspace(
    saved_root: &str,
    projects: &[ProjectConfig],
) -> Result<(WorkspaceResult, bool)> {
    let cwd = std::env::current_dir()
        .map_err(|err| anyhow!("获取当前目录失败：{err}"))?
        .to_string_lossy()
        .to_string();

    let mut options = vec![WorkspaceOption {
        label: "当前目录".to_string(),
        desc: cwd.clone(),
        choice: WorkspaceChoice::CurrentDir,
        path: cwd,
    }];

    for project in projects {
        options.push(WorkspaceOption {
            label: project.name.clone(),
            desc: project.path.clone(),
            choice: WorkspaceChoice::Project,
            path: project.path.clone(),
        });
    }

    if !saved_root.trim().is_empty() {
        options.push(WorkspaceOption {
            label: "从根目录选择子文件夹…".to_string(),
            desc: saved_root.to_string(),
            choice: WorkspaceChoice::BrowseRoot,
            path: String::new(),
        });
    }

    options.push(WorkspaceOption {
        label: "选择新的根目录…".to_string(),
        desc: String::new(),
        choice: WorkspaceChoice::NewRoot,
        path: String::new(),
    });

    let mut state = SimpleListState::new(
        "选择工作区",
        options
            .iter()
            .map(|option| DisplayRow {
                label: option.label.clone(),
                desc: option.desc.clone(),
            })
            .collect(),
        HelpLine,
    );

    let selected = run_terminal_loop(
        session_mode(false, state.inline_height()),
        &mut state,
        |terminal, state| {
            terminal.draw(|frame| render_simple_list(frame, state))?;
            Ok(())
        },
        |state, key| state.handle_key(key),
    )?;

    match selected {
        TerminalOutcome::Canceled => Ok((
            WorkspaceResult {
                choice: WorkspaceChoice::CurrentDir,
                project_path: String::new(),
            },
            true,
        )),
        TerminalOutcome::Selected(index) => {
            let option = &options[index];
            Ok((
                WorkspaceResult {
                    choice: option.choice,
                    project_path: option.path.clone(),
                },
                false,
            ))
        }
        TerminalOutcome::Text(_) => Err(anyhow!("工作区选择状态异常")),
    }
}

pub fn select_root(
    _label: &str,
    options: &[RootOption],
    default_input: &str,
) -> Result<(String, bool)> {
    let root_options = if options.is_empty() {
        vec![RootOption {
            label: "手动输入路径...".to_string(),
            value: String::new(),
            is_manual: true,
        }]
    } else {
        options.to_vec()
    };

    let mut state = RootSelectState::new(root_options.clone(), default_input);
    let outcome = run_terminal_loop(
        session_mode(false, state.inline_height()),
        &mut state,
        |terminal, state| {
            terminal.draw(|frame| render_root_select(frame, state))?;
            Ok(())
        },
        |state, key| state.handle_key(key),
    )?;

    match outcome {
        TerminalOutcome::Canceled => Ok((String::new(), true)),
        TerminalOutcome::Text(value) => Ok((value, false)),
        TerminalOutcome::Selected(index) => {
            let option = &root_options[index];
            Ok((option.value.clone(), false))
        }
    }
}

pub fn select_dir(dirs: &[Dir]) -> Result<(Dir, bool)> {
    let mut state = DirSelectState::new(dirs.to_vec());
    let outcome = run_terminal_loop(
        session_mode(true, 0),
        &mut state,
        |terminal, state| {
            terminal.draw(|frame| render_dir_select(frame, state))?;
            Ok(())
        },
        |state, key| state.handle_key(key),
    )?;

    match outcome {
        TerminalOutcome::Canceled => Ok((
            Dir {
                name: String::new(),
                path: String::new(),
            },
            true,
        )),
        TerminalOutcome::Selected(index) => Ok((dirs[index].clone(), false)),
        TerminalOutcome::Text(_) => Err(anyhow!("目录选择状态异常")),
    }
}

pub fn select_command(commands: &[CommandConfig]) -> Result<(CommandConfig, bool)> {
    let commands = if commands.is_empty() {
        crate::config::default_commands()
    } else {
        commands.to_vec()
    };

    let rows = commands
        .iter()
        .map(|command| DisplayRow {
            label: command.name.clone(),
            desc: if command.args.is_empty() {
                command.command.clone()
            } else {
                format!("{} {}", command.command, command.args.join(" "))
            },
        })
        .collect();

    let mut state = SimpleListState::new("选择启动模式", rows, HelpLine);
    let outcome = run_terminal_loop(
        session_mode(false, state.inline_height()),
        &mut state,
        |terminal, state| {
            terminal.draw(|frame| render_simple_list(frame, state))?;
            Ok(())
        },
        |state, key| state.handle_key(key),
    )?;

    match outcome {
        TerminalOutcome::Canceled => Ok((
            CommandConfig {
                name: String::new(),
                command: String::new(),
                args: Vec::new(),
            },
            true,
        )),
        TerminalOutcome::Selected(index) => Ok((commands[index].clone(), false)),
        TerminalOutcome::Text(_) => Err(anyhow!("命令选择状态异常")),
    }
}

fn run_terminal_loop<S, FDraw, FKey>(
    session_mode: SessionMode,
    state: &mut S,
    mut draw: FDraw,
    mut handle_key: FKey,
) -> Result<TerminalOutcome>
where
    FDraw: FnMut(&mut Terminal<CrosstermBackend<Stdout>>, &S) -> Result<()>,
    FKey: FnMut(&mut S, KeyEvent) -> Option<TerminalOutcome>,
{
    let mut session = TerminalSession::new(session_mode)?;

    loop {
        draw(session.terminal_mut(), state)?;
        match event::read()? {
            Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                if let Some(outcome) = handle_key(state, key) {
                    return Ok(outcome);
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    mode: SessionMode,
}

impl TerminalSession {
    fn new(mode: SessionMode) -> Result<Self> {
        enable_raw_mode()?;
        let mut term_stdout = stdout();
        match mode {
            SessionMode::AltScreen => {
                crossterm::execute!(term_stdout, EnterAlternateScreen, cursor::Hide)?;
                let backend = CrosstermBackend::new(term_stdout);
                let terminal = Terminal::new(backend)?;
                Ok(Self { terminal, mode })
            }
            SessionMode::Inline(height) => {
                crossterm::execute!(term_stdout, cursor::Hide)?;
                let backend = CrosstermBackend::new(term_stdout);
                match Terminal::with_options(
                    backend,
                    TerminalOptions {
                        viewport: Viewport::Inline(height),
                    },
                ) {
                    Ok(terminal) => Ok(Self { terminal, mode }),
                    Err(_) => {
                        let mut fallback_stdout = stdout();
                        crossterm::execute!(fallback_stdout, EnterAlternateScreen)?;
                        let backend = CrosstermBackend::new(fallback_stdout);
                        let terminal = Terminal::new(backend)?;
                        Ok(Self {
                            terminal,
                            mode: SessionMode::AltScreen,
                        })
                    }
                }
            }
        }
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if matches!(self.mode, SessionMode::Inline(_)) {
            let _ = self.terminal.clear();
        }
        let _ = disable_raw_mode();
        let _ = self.terminal.show_cursor();
        if matches!(self.mode, SessionMode::AltScreen) {
            let _ = crossterm::execute!(
                self.terminal.backend_mut(),
                LeaveAlternateScreen,
                cursor::Show
            );
        } else {
            let _ = crossterm::execute!(self.terminal.backend_mut(), cursor::Show);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SessionMode {
    AltScreen,
    Inline(u16),
}

fn session_mode(use_alt_screen: bool, inline_height: u16) -> SessionMode {
    if use_alt_screen {
        SessionMode::AltScreen
    } else {
        SessionMode::Inline(inline_height.max(1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TerminalOutcome {
    Selected(usize),
    Text(String),
    Canceled,
}

#[derive(Debug, Clone)]
struct DisplayRow {
    label: String,
    desc: String,
}

impl DisplayRow {
    fn line_count(&self) -> u16 {
        1
    }
}

#[derive(Debug, Clone, Default)]
struct HelpLine;

impl HelpLine {
    fn text(&self) -> &'static str {
        "[↑↓] 选择  [Enter] 确认  [Esc] 取消"
    }
}

#[derive(Debug, Clone)]
struct SimpleListState {
    title: &'static str,
    rows: Vec<DisplayRow>,
    selected: usize,
    help: HelpLine,
}

impl SimpleListState {
    fn new(title: &'static str, rows: Vec<DisplayRow>, help: HelpLine) -> Self {
        Self {
            title,
            rows,
            selected: 0,
            help,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<TerminalOutcome> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => {
                if self.selected + 1 < self.rows.len() {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => return Some(TerminalOutcome::Selected(self.selected)),
            KeyCode::Esc => return Some(TerminalOutcome::Canceled),
            KeyCode::Char('q') if key.modifiers.is_empty() => {
                return Some(TerminalOutcome::Canceled);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(TerminalOutcome::Canceled);
            }
            _ => {}
        }
        None
    }

    fn inline_height(&self) -> u16 {
        inline_panel_total_height(self.rows.iter().map(DisplayRow::line_count).sum::<u16>() + 4)
    }
}

#[derive(Debug, Clone)]
struct WorkspaceOption {
    label: String,
    desc: String,
    choice: WorkspaceChoice,
    path: String,
}

#[derive(Debug, Clone)]
struct RootSelectState {
    options: Vec<RootOption>,
    selected: usize,
    phase: RootPhase,
    input: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum RootPhase {
    Select,
    Input,
}

impl RootSelectState {
    fn new(options: Vec<RootOption>, default_input: &str) -> Self {
        Self {
            options,
            selected: 0,
            phase: RootPhase::Select,
            input: default_input.to_string(),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<TerminalOutcome> {
        match self.phase {
            RootPhase::Select => self.handle_select_key(key),
            RootPhase::Input => self.handle_input_key(key),
        }
    }

    fn handle_select_key(&mut self, key: KeyEvent) -> Option<TerminalOutcome> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => {
                if self.selected + 1 < self.options.len() {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                if self.options[self.selected].is_manual {
                    self.phase = RootPhase::Input;
                } else {
                    return Some(TerminalOutcome::Selected(self.selected));
                }
            }
            KeyCode::Esc | KeyCode::Char('q') if key.modifiers.is_empty() => {
                return Some(TerminalOutcome::Canceled);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(TerminalOutcome::Canceled);
            }
            _ => {}
        }
        None
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> Option<TerminalOutcome> {
        match key.code {
            KeyCode::Enter => {
                let value = self.input.trim().to_string();
                if !value.is_empty() {
                    return Some(TerminalOutcome::Text(value));
                }
            }
            KeyCode::Esc => self.phase = RootPhase::Select,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(TerminalOutcome::Canceled);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.push(ch);
            }
            _ => {}
        }
        None
    }

    fn inline_height(&self) -> u16 {
        let select_height = inline_panel_total_height(self.options.len() as u16 + 4);
        let input_height = inline_panel_total_height(5);
        select_height.max(input_height)
    }
}

#[derive(Debug, Clone)]
struct DirSelectState {
    dirs: Vec<Dir>,
    selected: usize,
    filter: String,
}

impl DirSelectState {
    fn new(dirs: Vec<Dir>) -> Self {
        Self {
            dirs,
            selected: 0,
            filter: String::new(),
        }
    }

    fn filtered_indices(&self) -> Vec<usize> {
        if self.filter.is_empty() {
            return (0..self.dirs.len()).collect();
        }

        let filter = self.filter.to_lowercase();
        self.dirs
            .iter()
            .enumerate()
            .filter(|(_, dir)| {
                dir.name.to_lowercase().contains(&filter)
                    || dir.path.to_lowercase().contains(&filter)
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<TerminalOutcome> {
        let filtered = self.filtered_indices();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => {
                if self.selected + 1 < filtered.len() {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(index) = filtered.get(self.selected) {
                    return Some(TerminalOutcome::Selected(*index));
                }
            }
            KeyCode::Esc => {
                if self.filter.is_empty() {
                    return Some(TerminalOutcome::Canceled);
                }
                self.filter.clear();
                self.selected = 0;
            }
            KeyCode::Char('q') if key.modifiers.is_empty() && self.filter.is_empty() => {
                return Some(TerminalOutcome::Canceled);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(TerminalOutcome::Canceled);
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.selected = 0;
            }
            KeyCode::Char(ch)
                if !(key.modifiers.contains(KeyModifiers::CONTROL)
                    || (ch == 'q' && self.filter.is_empty())) =>
            {
                self.filter.push(ch);
                self.selected = 0;
            }
            _ => {}
        }

        let filtered = self.filtered_indices();
        if self.selected >= filtered.len() && !filtered.is_empty() {
            self.selected = filtered.len() - 1;
        }
        None
    }
}

fn render_simple_list(frame: &mut ratatui::Frame<'_>, state: &SimpleListState) {
    let lines = simple_list_lines(state);
    let area = inline_panel_area(frame.area(), max_line_width(&lines), lines.len() as u16, 32);
    let block = inline_panel_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_root_select(frame: &mut ratatui::Frame<'_>, state: &RootSelectState) {
    match state.phase {
        RootPhase::Select => {
            let rows = state
                .options
                .iter()
                .map(|option| DisplayRow {
                    label: option.label.clone(),
                    desc: option.value.clone(),
                })
                .collect::<Vec<_>>();
            let list_state = SimpleListState {
                title: "选择根目录",
                rows,
                selected: state.selected,
                help: HelpLine,
            };
            render_simple_list(frame, &list_state);
        }
        RootPhase::Input => {
            let lines = root_input_lines(state);
            let area =
                inline_panel_area(frame.area(), max_line_width(&lines), lines.len() as u16, 50);
            let block = inline_panel_block();
            let inner = block.inner(area);
            frame.render_widget(block, area);
            frame.render_widget(Paragraph::new(lines), inner);

            let cursor_x = inner.x + 2 + Line::from(state.input.as_str()).width() as u16;
            let cursor_y = inner.y + 2;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn render_dir_select(frame: &mut ratatui::Frame<'_>, state: &DirSelectState) {
    let area = centered_area(frame.area(), 90, 80);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(area);

    let filter_block = panel_block("选择项目目录");
    let filter_inner = filter_block.inner(chunks[0]);
    frame.render_widget(filter_block, chunks[0]);
    frame.render_widget(
        Paragraph::new(format!("搜索: {}", state.filter)).style(style_normal()),
        filter_inner,
    );

    let filtered = state.filtered_indices();
    let items = if filtered.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "没有匹配结果",
            style_dim(),
        )))]
    } else {
        filtered
            .iter()
            .map(|index| {
                let dir = &state.dirs[*index];
                ListItem::new(vec![
                    Line::from(dir.name.clone()),
                    Line::from(Span::styled(dir.path.clone(), style_dim())),
                ])
            })
            .collect::<Vec<_>>()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(style_selected())
        .highlight_symbol("▌ ");
    let mut list_state = ListState::default();
    if !filtered.is_empty() {
        list_state.select(Some(state.selected));
    }
    frame.render_stateful_widget(list, chunks[1], &mut list_state);
    frame.render_widget(
        Paragraph::new("[↑↓] 选择  [Enter] 确认  [Esc] 清空/取消").style(style_dim()),
        chunks[2],
    );
}

fn panel_block(title: &str) -> Block<'_> {
    Block::default()
        .title(Line::from(Span::styled(title.to_string(), style_title())))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
}

fn inline_panel_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(style_dim())
        .padding(Padding::new(2, 2, 1, 1))
}

fn simple_list_lines(state: &SimpleListState) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(state.rows.len() + 4);
    lines.push(Line::styled(state.title.to_string(), style_title()));
    lines.push(Line::default());

    for (index, row) in state.rows.iter().enumerate() {
        lines.push(simple_list_row(row, index == state.selected));
    }

    lines.push(Line::default());
    lines.push(Line::styled(state.help.text(), style_dim()));
    lines
}

fn root_input_lines(state: &RootSelectState) -> Vec<Line<'static>> {
    vec![
        Line::styled("输入根目录路径", style_title()),
        Line::default(),
        root_input_line(&state.input),
        Line::default(),
        Line::styled("[Enter] 确认  [Esc] 返回选择", style_dim()),
    ]
}

fn root_input_line(input: &str) -> Line<'static> {
    if input.is_empty() {
        Line::from(vec![
            Span::styled("> ", style_selected()),
            Span::styled("请输入根目录路径，例如 ~/Projects", style_dim()),
        ])
    } else {
        Line::from(vec![
            Span::styled("> ", style_selected()),
            Span::styled(input.to_string(), style_normal()),
        ])
    }
}

fn simple_list_row(row: &DisplayRow, selected: bool) -> Line<'static> {
    if selected {
        let mut spans = vec![
            Span::styled("▌ ", style_cursor()),
            Span::styled(row.label.clone(), style_selected()),
            Span::styled(" ›", style_cursor()),
        ];
        if !row.desc.is_empty() {
            spans.push(Span::styled(format!("  {}", row.desc), style_dim()));
        }
        Line::from(spans)
    } else {
        let mut spans = vec![
            Span::raw("  "),
            Span::styled(row.label.clone(), style_normal()),
        ];
        if !row.desc.is_empty() {
            spans.push(Span::styled(format!("  {}", row.desc), style_dim()));
        }
        Line::from(spans)
    }
}

fn inline_panel_area(
    area: Rect,
    content_width: u16,
    content_height: u16,
    min_content_width: u16,
) -> Rect {
    let left_margin = if area.width > 4 { 2 } else { 0 };
    let right_margin = if area.width > 4 { 2 } else { 0 };
    let top_margin = if area.height > 1 { 1 } else { 0 };
    let max_width = area.width.saturating_sub(left_margin + right_margin).max(1);
    let width = content_width
        .max(min_content_width)
        .saturating_add(6)
        .min(max_width)
        .max(1);
    let height = content_height
        .saturating_add(4)
        .min(area.height.saturating_sub(top_margin).max(1))
        .max(1);

    Rect::new(area.x + left_margin, area.y + top_margin, width, height)
}

fn inline_panel_total_height(content_height: u16) -> u16 {
    content_height.saturating_add(5)
}

fn max_line_width(lines: &[Line<'_>]) -> u16 {
    lines
        .iter()
        .map(|line| line.width() as u16)
        .max()
        .unwrap_or(0)
}

fn style_title() -> Style {
    Style::default()
        .fg(Color::Blue)
        .add_modifier(Modifier::BOLD)
}

fn style_dim() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn style_normal() -> Style {
    Style::default().fg(Color::White)
}

fn style_selected() -> Style {
    Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::BOLD)
}

fn style_cursor() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

fn centered_area(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_list_state_navigation_and_cancel() {
        let mut state = SimpleListState::new(
            "测试",
            vec![
                DisplayRow {
                    label: "a".to_string(),
                    desc: String::new(),
                },
                DisplayRow {
                    label: "b".to_string(),
                    desc: String::new(),
                },
            ],
            HelpLine,
        );

        assert_eq!(state.handle_key(KeyEvent::from(KeyCode::Down)), None);
        assert_eq!(state.selected, 1);
        assert_eq!(
            state.handle_key(KeyEvent::from(KeyCode::Enter)),
            Some(TerminalOutcome::Selected(1))
        );
        assert_eq!(
            state.handle_key(KeyEvent::from(KeyCode::Esc)),
            Some(TerminalOutcome::Canceled)
        );
    }

    #[test]
    fn test_root_select_state_enters_manual_input_and_returns() {
        let mut state = RootSelectState::new(
            vec![
                RootOption {
                    label: "当前目录".to_string(),
                    value: ".".to_string(),
                    is_manual: false,
                },
                RootOption {
                    label: "手动输入路径...".to_string(),
                    value: String::new(),
                    is_manual: true,
                },
            ],
            "~/Projects",
        );

        state.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(state.selected, 1);
        assert_eq!(state.handle_key(KeyEvent::from(KeyCode::Enter)), None);
        assert_eq!(state.phase, RootPhase::Input);
        assert_eq!(state.handle_key(KeyEvent::from(KeyCode::Esc)), None);
        assert_eq!(state.phase, RootPhase::Select);
    }

    #[test]
    fn test_dir_select_state_filters_and_clears() {
        let mut state = DirSelectState::new(vec![
            Dir {
                name: "alpha".to_string(),
                path: "/a".to_string(),
            },
            Dir {
                name: "beta".to_string(),
                path: "/b".to_string(),
            },
        ]);

        state.handle_key(KeyEvent::from(KeyCode::Char('b')));
        assert_eq!(state.filter, "b");
        assert_eq!(state.filtered_indices(), vec![1]);
        assert_eq!(state.handle_key(KeyEvent::from(KeyCode::Esc)), None);
        assert!(state.filter.is_empty());
        assert_eq!(
            state.handle_key(KeyEvent::from(KeyCode::Esc)),
            Some(TerminalOutcome::Canceled)
        );
    }

    #[test]
    fn test_session_mode_uses_inline_viewport_for_non_alt_ui() {
        assert_eq!(session_mode(false, 12), SessionMode::Inline(12));
    }

    #[test]
    fn test_session_mode_uses_alt_screen_for_fullscreen_ui() {
        assert_eq!(session_mode(true, 12), SessionMode::AltScreen);
    }

    #[test]
    fn test_simple_list_inline_height_counts_rows_and_help() {
        let state = SimpleListState::new(
            "测试",
            vec![
                DisplayRow {
                    label: "a".to_string(),
                    desc: String::new(),
                },
                DisplayRow {
                    label: "b".to_string(),
                    desc: "/tmp/b".to_string(),
                },
            ],
            HelpLine,
        );

        assert_eq!(state.inline_height(), 11);
    }

    #[test]
    fn test_inline_panel_area_is_top_aligned() {
        let area = inline_panel_area(Rect::new(0, 0, 100, 24), 24, 8, 0);

        assert_eq!(area.x, 2);
        assert_eq!(area.y, 1);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 12);
    }

    #[test]
    fn test_inline_panel_area_uses_compact_width() {
        let state = SimpleListState::new(
            "测试",
            vec![DisplayRow {
                label: "当前目录".to_string(),
                desc: "/tmp/demo".to_string(),
            }],
            HelpLine,
        );

        let lines = simple_list_lines(&state);
        let area = inline_panel_area(
            Rect::new(0, 0, 120, 20),
            max_line_width(&lines),
            lines.len() as u16,
            32,
        );

        assert!(area.width < 60);
        assert_eq!(area.x, 2);
    }

    #[test]
    fn test_simple_list_rows_render_desc_on_same_line() {
        let row = DisplayRow {
            label: "当前目录".to_string(),
            desc: "/tmp/demo".to_string(),
        };

        assert_eq!(simple_list_row(&row, false).width(), 21);
        assert_eq!(simple_list_row(&row, true).width(), 23);
    }
}
