use crate::cache_targets::CacheTarget;
use crate::scanner::{self, DeletionEntry};
use anyhow::Result;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use humansize::{format_size, DECIMAL};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

#[derive(Clone)]
struct RowEntry {
    text: String,
    category: String,
    path: Option<PathBuf>,
    size_bytes: u64,
    essential_warning: bool,
    is_header: bool,
}

pub fn run_tui(targets: &[CacheTarget]) -> Result<Vec<PathBuf>> {
    if targets.is_empty() {
        return Ok(vec![]);
    }

    let rows = build_rows(targets);
    if rows.is_empty() {
        return Ok(vec![]);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut idx = 0usize;
    let mut selected = HashSet::<usize>::new();
    let mut scroll_offset = 0usize;

    loop {
        let size = terminal.size()?;
        let list_height = size.height.saturating_sub(2 + 3 + 2) as usize;
        let visible_rows = list_height.max(1);
        if idx < scroll_offset {
            scroll_offset = idx;
        } else if idx >= scroll_offset + visible_rows {
            scroll_offset = idx + 1 - visible_rows;
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(f.size());

            let end = (scroll_offset + visible_rows).min(rows.len());
            let items: Vec<ListItem> = rows
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(end.saturating_sub(scroll_offset))
                .map(|(i, row)| {
                    let mark = if row.is_header {
                        " - "
                    } else if selected.contains(&i) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    let cursor = if i == idx { ">" } else { " " };
                    let prefix = format!("{cursor} {mark} ");
                    let size = if row.is_header {
                        "".to_string()
                    } else {
                        format!("{:<10}", format_size(row.size_bytes, DECIMAL))
                    };
                    let warn = if row.essential_warning && !row.is_header {
                        " ⚠️"
                    } else {
                        ""
                    };
                    let content = format!("{prefix}{:<10} {}{}", size, row.text, warn);
                    let style = if row.is_header {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else if row.essential_warning {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(vec![Span::styled(content, style)]))
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title("cachectl - nested deletion view")
                    .borders(Borders::ALL),
            );
            f.render_widget(list, chunks[0]);

            let help = Paragraph::new("Up/Down/MouseWheel: scroll  Space: toggle row  g: category  a: all  Enter: confirm  q: cancel")
                .style(Style::default().add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(help, chunks[1]);
        })?;

        match event::read()? {
            Event::Key(k) => match k.code {
                KeyCode::Up => {
                    idx = idx.saturating_sub(1);
                }
                KeyCode::Down => {
                    if idx + 1 < rows.len() {
                        idx += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    if rows[idx].is_header {
                        continue;
                    }
                    if selected.contains(&idx) {
                        selected.remove(&idx);
                    } else {
                        selected.insert(idx);
                    }
                }
                KeyCode::Char('a') => {
                    let all_idxs: Vec<usize> = rows
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| !r.is_header)
                        .map(|(i, _)| i)
                        .collect();
                    if selected.len() == all_idxs.len() {
                        selected.clear();
                    } else {
                        selected = all_idxs.into_iter().collect();
                    }
                }
                KeyCode::Char('g') => {
                    let category = rows[idx].category.clone();
                    let group_idxs: Vec<usize> = rows
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| !r.is_header && r.category == category)
                        .map(|(i, _)| i)
                        .collect();
                    let all_selected = group_idxs.iter().all(|i| selected.contains(i));
                    for id in group_idxs {
                        if all_selected {
                            selected.remove(&id);
                        } else {
                            selected.insert(id);
                        }
                    }
                }
                KeyCode::Enter => break,
                KeyCode::Char('q') => {
                    selected.clear();
                    break;
                }
                _ => {}
            },
            Event::Mouse(m) => match m.kind {
                MouseEventKind::ScrollDown => {
                    if idx + 1 < rows.len() {
                        idx += 1;
                    }
                }
                MouseEventKind::ScrollUp => {
                    idx = idx.saturating_sub(1);
                }
                _ => {}
            },
            _ => {}
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(selected
        .into_iter()
        .filter_map(|i| rows.get(i).and_then(|r| r.path.clone()))
        .collect())
}

fn build_rows(targets: &[CacheTarget]) -> Vec<RowEntry> {
    let mut rows = Vec::new();
    for target in targets {
        let header = format!(
            "{} ({}) - {}",
            target.label,
            target.id,
            format_size(target.size_bytes, DECIMAL)
        );
        rows.push(RowEntry {
            text: header,
            category: target.id.clone(),
            path: None,
            size_bytes: target.size_bytes,
            essential_warning: false,
            is_header: true,
        });
        let entries: Vec<DeletionEntry> = scanner::collect_deletion_entries(target);
        for e in entries {
            rows.push(RowEntry {
                text: truncate_middle(&e.path.display().to_string(), 86),
                category: e.category,
                path: Some(e.path),
                size_bytes: e.size_bytes,
                essential_warning: e.is_essential_warning,
                is_header: false,
            });
        }
    }
    rows
}

fn truncate_middle(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        return input.to_string();
    }
    let keep = max_len.saturating_sub(3);
    let left = keep / 2;
    let right = keep.saturating_sub(left);
    format!("{}...{}", &input[..left], &input[input.len() - right..])
}
