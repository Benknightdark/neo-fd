use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use regex::Regex;
use scanner_core::Scanner;
use std::io;
use std::path::PathBuf;

enum AppState {
    SelectingRegex,
    InputtingPath,
    Finished,
}

struct App {
    state: AppState,
    regex_items: Vec<(&'static str, &'static str)>,
    selected_index: usize,
    custom_regex: String,
    path_input: String,
    error_message: Option<String>,

    // Results to pass to scanner
    final_patterns: Vec<(String, Regex)>,
    final_path: PathBuf,
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::SelectingRegex,
            regex_items: vec![
                ("身分證字號", r"[A-Za-z][12]\d{8}"),
                ("台灣十大姓氏", r"[陳林黃張李王吳劉蔡楊][\u4e00-\u9fa5]{2}"),
            ],
            selected_index: 0,
            custom_regex: String::new(),
            path_input: String::new(),
            error_message: None,
            final_patterns: Vec::new(),
            final_path: PathBuf::new(),
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    } else if matches!(app.state, AppState::Finished) {
        // Start scanning
        let scanner = Scanner::new(app.final_patterns, |res| {
            println!(
                "[{}:{}] {}: {}",
                res.path, res.line_num, res.pattern_name, res.matched_text
            );
        });
        println!("正在掃描目錄: {:?}", app.final_path);
        scanner.scan_dir(&app.final_path);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.state {
                AppState::SelectingRegex => match key.code {
                    KeyCode::Up => {
                        app.error_message = None; // 切換時清除錯誤
                        if app.selected_index > 0 {
                            app.selected_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        app.error_message = None; // 切換時清除錯誤
                        if app.selected_index < app.regex_items.len() {
                            app.selected_index += 1;
                        }
                    }
                    KeyCode::Char(c) => {
                        app.error_message = None;
                        app.custom_regex.push(c);
                        app.selected_index = app.regex_items.len(); // Move focus to input
                    }
                    KeyCode::Backspace => {
                        app.error_message = None;
                        app.custom_regex.pop();
                    }
                    KeyCode::Enter => {
                        app.error_message = None; // 先清空上次錯誤
                        if app.selected_index < app.regex_items.len() {
                            let (name, pattern) = app.regex_items[app.selected_index];
                            if let Ok(re) = Regex::new(pattern) {
                                app.final_patterns.push((name.to_string(), re));
                                app.state = AppState::InputtingPath;
                            }
                        } else if !app.custom_regex.is_empty() {
                            match Regex::new(&app.custom_regex) {
                                Ok(re) => {
                                    app.final_patterns.push(("自定義".to_string(), re));
                                    app.state = AppState::InputtingPath;
                                }
                                Err(err) => {
                                    app.error_message = Some(format!("無效的 Regex: {}", err));
                                }
                            }
                        }
                    }
                    KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                AppState::InputtingPath => match key.code {
                    KeyCode::Char(c) => {
                        app.path_input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.path_input.pop();
                    }
                    KeyCode::Enter => {
                        if app.path_input.trim().is_empty() {
                            #[cfg(target_os = "windows")]
                            {
                                app.final_path = PathBuf::from("C:\\");
                            }
                            #[cfg(not(target_os = "windows"))]
                            {
                                app.final_path = PathBuf::from("/");
                            }
                        } else {
                            app.final_path = PathBuf::from(&app.path_input);
                        }
                        app.state = AppState::Finished;
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        app.state = AppState::SelectingRegex;
                        app.final_patterns.clear();
                    }
                    _ => {}
                },
                AppState::Finished => return Ok(()),
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(5), // 稍微加高以容納可能的錯誤訊息
            ]
            .as_ref(),
        )
        .split(f.area());

    match app.state {
        AppState::SelectingRegex => {
            let title = Paragraph::new("請選擇掃描模式 (使用方向鍵選取，或直接輸入自定義 Regex)")
                .block(Block::default().borders(Borders::ALL).title("Regex 掃描器"));
            f.render_widget(title, chunks[0]);

            let items: Vec<ListItem> = app
                .regex_items
                .iter()
                .enumerate()
                .map(|(i, (name, _))| {
                    let style = if i == app.selected_index {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(name.to_string()).style(style)
                })
                .collect();

            let list =
                List::new(items).block(Block::default().borders(Borders::ALL).title("內建模式"));
            f.render_widget(list, chunks[1]);

            let input_style = if app.selected_index == app.regex_items.len() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let mut input_lines = vec![Line::from(vec![Span::styled(
                app.custom_regex.as_str(),
                input_style,
            )])];

            // 繪製錯誤訊息
            if let Some(ref err_msg) = app.error_message {
                input_lines.push(Line::from(vec![Span::styled(
                    err_msg.as_str(),
                    Style::default().fg(Color::Red),
                )]));
            }

            let input = Paragraph::new(input_lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("自定義 Regex 輸入 (Enter 執行)"),
            );
            f.render_widget(input, chunks[2]);
        }
        AppState::InputtingPath => {
            let title = Paragraph::new("請輸入要掃描的目錄路徑 (直接按 Enter 掃描全機)")
                .block(Block::default().borders(Borders::ALL).title("路徑設定"));
            f.render_widget(title, chunks[0]);

            let path_display = if app.path_input.is_empty() {
                "(預設為全機掃描 / )"
            } else {
                app.path_input.as_str()
            };
            let input = Paragraph::new(path_display)
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("目標目錄"));
            f.render_widget(input, chunks[1]);

            let hint =
                Paragraph::new("Esc: 返回模式選擇 | Enter: 開始掃描").block(Block::default());
            f.render_widget(hint, chunks[2]);
        }
        _ => {}
    }
}
