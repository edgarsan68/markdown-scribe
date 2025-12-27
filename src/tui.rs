use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::fs::read_to_string;
use std::io::{self, Stdout};

use crate::render_markdown_to_html;

pub fn run_editor(file: Option<String>) -> Result<()> {
    let mut terminal = setup_terminal()?;

    let mut text = String::new();
    if let Some(f) = file {
        text = read_to_string(&f).context("Failed to load file for editing")?;
    }

    let mut cursor_pos = text.len(); // Simple cursor at end

    loop {
        terminal.draw(|f| draw_ui(f, &text, &render_markdown_to_html(&text).unwrap_or_default()))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        text.insert(cursor_pos, '\n');
                        cursor_pos += 1;
                    }
                    KeyCode::Char(c) => {
                        text.insert(cursor_pos, c);
                        cursor_pos += 1;
                    }
                    KeyCode::Backspace => {
                        if cursor_pos > 0 {
                            text.remove(cursor_pos - 1);
                            cursor_pos -= 1;
                        }
                    }
                    KeyCode::Left => if cursor_pos > 0 { cursor_pos -= 1; }
                    KeyCode::Right => if cursor_pos < text.len() { cursor_pos += 1; }
                    _ => {}
                }
            }
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(terminal.show_cursor()?)
}

fn draw_ui(f: &mut Frame, text: &str, preview: &str) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    let editor_block = Block::default().title("Editor").borders(Borders::ALL);
    let preview_block = Block::default().title("Preview").borders(Borders::ALL);

    f.render_widget(Paragraph::new(text).block(editor_block), chunks[0]);
    f.render_widget(Paragraph::new(preview).block(preview_block), chunks[1]);
}