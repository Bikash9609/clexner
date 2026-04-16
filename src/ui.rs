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
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const UI_MIN_FILE_SIZE_BYTES: u64 = 1_000_000;

#[derive(Clone)]
struct RowEntry {
    text: String,
    category: String,
    path: Option<PathBuf>,
    size_bytes: u64,
    essential_warning: bool,
    is_header: bool,
    status: RowStatus,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum UiMode {
    Selecting,
    Confirming,
}

#[derive(Clone, PartialEq, Eq)]
enum RowStatus {
    Idle,
    Queued,
    Deleting,
    Deleted,
    Failed,
}

enum DeleteRequest {
    Delete(PathBuf),
}

enum DeleteEvent {
    Started(PathBuf),
    Done(PathBuf),
    Failed(PathBuf),
}

pub fn run_tui(targets: &[CacheTarget]) -> Result<()> {
    if targets.is_empty() {
        return Ok(());
    }

    let mut rows = build_rows(targets);
    if rows.is_empty() {
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut idx = 0usize;
    let mut selected = HashSet::<usize>::new();
    let mut scroll_offset = 0usize;
    let mut mode = UiMode::Selecting;
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut spinner_idx = 0usize;
    let (delete_req_tx, delete_req_rx) = mpsc::channel::<DeleteRequest>();
    let (delete_evt_tx, delete_evt_rx) = mpsc::channel::<DeleteEvent>();
    let delete_worker = thread::spawn(move || {
        for req in delete_req_rx {
            match req {
                DeleteRequest::Delete(path) => {
                    let _ = delete_evt_tx.send(DeleteEvent::Started(path.clone()));
                    let result = if path.is_file() {
                        fs::remove_file(&path)
                    } else {
                        fs::remove_dir_all(&path)
                    };
                    match result {
                        Ok(_) => {
                            let _ = delete_evt_tx.send(DeleteEvent::Done(path));
                        }
                        Err(_) => {
                            let _ = delete_evt_tx.send(DeleteEvent::Failed(path));
                        }
                    }
                }
            }
        }
    });

    loop {
        while let Ok(evt) = delete_evt_rx.try_recv() {
            match evt {
                DeleteEvent::Started(path) => {
                    if let Some(row) = rows
                        .iter_mut()
                        .find(|r| !r.is_header && r.path.as_ref().is_some_and(|p| p == &path))
                    {
                        row.status = RowStatus::Deleting;
                    }
                }
                DeleteEvent::Done(path) => {
                    if let Some(row) = rows
                        .iter_mut()
                        .find(|r| !r.is_header && r.path.as_ref().is_some_and(|p| p == &path))
                    {
                        row.status = RowStatus::Deleted;
                    }
                }
                DeleteEvent::Failed(path) => {
                    if let Some(row) = rows
                        .iter_mut()
                        .find(|r| !r.is_header && r.path.as_ref().is_some_and(|p| p == &path))
                    {
                        row.status = RowStatus::Failed;
                    }
                }
            }
        }

        let size = terminal.size()?;
        let list_height = size.height.saturating_sub(2 + 6 + 3 + 2) as usize;
        let visible_rows = list_height.max(1);
        if idx < scroll_offset {
            scroll_offset = idx;
        } else if idx >= scroll_offset + visible_rows {
            scroll_offset = idx + 1 - visible_rows;
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(6),
                    Constraint::Length(4),
                    Constraint::Length(3),
                ])
                .split(f.size());

            let total_reclaimable: u64 = rows
                .iter()
                .filter(|r| !r.is_header)
                .map(|r| r.size_bytes)
                .sum();
            let selected_reclaimable: u64 = selected
                .iter()
                .filter_map(|i| rows.get(*i))
                .filter(|r| !r.is_header && r.status == RowStatus::Idle)
                .map(|r| r.size_bytes)
                .sum();
            let selected_items = selected
                .iter()
                .filter_map(|i| rows.get(*i))
                .filter(|r| !r.is_header && r.status == RowStatus::Idle)
                .count();
            let deleting_items = rows
                .iter()
                .filter(|r| !r.is_header && (r.status == RowStatus::Queued || r.status == RowStatus::Deleting))
                .count();
            let deleted_items = rows
                .iter()
                .filter(|r| !r.is_header && r.status == RowStatus::Deleted)
                .count();
            let phase = if mode == UiMode::Selecting {
                "Preview mode - no files deleted yet"
            } else {
                "Confirming deletion - y to enqueue"
            };

            let end = (scroll_offset + visible_rows).min(rows.len());
            let items: Vec<ListItem> = rows
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(end.saturating_sub(scroll_offset))
                .map(|(i, row)| {
                    let mark = if row.is_header {
                        let group_idxs: Vec<usize> = rows
                            .iter()
                            .enumerate()
                            .filter(|(_, r)| !r.is_header && r.category == row.category && r.status == RowStatus::Idle)
                            .map(|(idx, _)| idx)
                            .collect();
                        if group_idxs.is_empty() {
                            "[ ]"
                        } else {
                            let selected_count =
                                group_idxs.iter().filter(|idx| selected.contains(idx)).count();
                            if selected_count == 0 {
                                "[ ]"
                            } else if selected_count == group_idxs.len() {
                                "[x]"
                            } else {
                                "[~]"
                            }
                        }
                    } else if selected.contains(&i) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    let cursor = if i == idx { ">" } else { " " };
                    let size = if row.is_header {
                        "".to_string()
                    } else {
                        format!("{:<10}", format_size(row.size_bytes, DECIMAL))
                    };
                    let status_tag = if !row.is_header {
                        if row.status == RowStatus::Queued {
                            " [DELETING queued]".to_string()
                        } else if row.status == RowStatus::Deleting {
                            if i == idx {
                                format!(" [DELETING {}]", spinner_frames[spinner_idx])
                            } else {
                                " [DELETING]".to_string()
                            }
                        } else if row.status == RowStatus::Deleted {
                            " [DELETED]".to_string()
                        } else if row.status == RowStatus::Failed {
                            " [FAILED]".to_string()
                        } else if row.essential_warning {
                            " [rebuild]".to_string()
                        } else {
                            " [safe]".to_string()
                        }
                    } else {
                        "".to_string()
                    };
                    let risk_tag = if !row.is_header && row.status == RowStatus::Idle {
                        if row.essential_warning {
                            " [rebuild]"
                        } else {
                            " [safe]"
                        }
                    } else {
                        ""
                    };
                    let content = if row.is_header {
                        format!("{cursor} {mark} {}", row.text)
                    } else {
                        format!("{cursor}     {mark} {:<10} {}{}", size, row.text, status_tag)
                    };
                    let style = if row.is_header {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else if row.status == RowStatus::Deleting || row.status == RowStatus::Queued {
                        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
                    } else if row.status == RowStatus::Deleted {
                        Style::default().fg(Color::Green)
                    } else if row.status == RowStatus::Failed {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    } else if i == idx {
                        let pulse = if spinner_idx % 2 == 0 {
                            Color::Yellow
                        } else {
                            Color::White
                        };
                        Style::default().fg(pulse).add_modifier(Modifier::BOLD)
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
                    .title(format!(
                        "cachectl {} - ready for cleanup",
                        spinner_frames[spinner_idx]
                    ))
                    .borders(Borders::ALL),
            );
            f.render_widget(list, chunks[0]);

            let current = &rows[idx];
            let selected_state = if current.is_header {
                "header row (group)"
            } else if selected.contains(&idx) {
                "selected for deletion"
            } else {
                "not selected"
            };
            let item_kind = if current.path.is_some() { "item" } else { "group" };
            let (purpose, safe_delete_note) = target_info(&current.category, current.essential_warning);
            let current_path = current
                .path
                .as_ref()
                .map(|p| truncate_middle(&p.display().to_string(), 80))
                .unwrap_or_else(|| current.text.clone());
            let info_text = format!(
                "Current {item_kind}: {}\nIdentifier: {}\nUse: {}\nSafety: {} ({})",
                current_path, current.category, purpose, safe_delete_note, selected_state
            );
            let info = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title("Current Item Info"));
            f.render_widget(info, chunks[1]);

            let summary_text = format!(
                "Total reclaimable: {}\nSelected: {} ({selected_items} items)\nDeleting: {deleting_items}  Deleted: {deleted_items}\n{phase}",
                format_size(total_reclaimable, DECIMAL),
                format_size(selected_reclaimable, DECIMAL),
            );
            let summary = Paragraph::new(summary_text)
                .block(Block::default().borders(Borders::ALL).title("Impact Preview"));
            f.render_widget(summary, chunks[2]);

            let help_text = if mode == UiMode::Selecting {
                "Up/Down/MouseWheel: scroll  Space: toggle row  g: category  a: all  Enter: delete selected  q: quit  (dirs + files >= 1 MB)"
            } else {
                "Confirm cleanup: y = enqueue delete  n = back  q/Esc = cancel"
            };
            let help = Paragraph::new(help_text)
                .style(Style::default().add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(help, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(120))? {
            match event::read()? {
            Event::Key(k) => match mode {
                UiMode::Selecting => match k.code {
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
                            let category = rows[idx].category.clone();
                            let group_idxs: Vec<usize> = rows
                                .iter()
                                .enumerate()
                                .filter(|(_, r)| !r.is_header && r.category == category && r.status == RowStatus::Idle)
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
                        } else if rows[idx].status != RowStatus::Idle {
                        } else if selected.contains(&idx) {
                            selected.remove(&idx);
                        } else {
                            selected.insert(idx);
                        }
                    }
                    KeyCode::Char('a') => {
                        let all_idxs: Vec<usize> = rows
                            .iter()
                            .enumerate()
                            .filter(|(_, r)| !r.is_header && r.status == RowStatus::Idle)
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
                            .filter(|(_, r)| !r.is_header && r.category == category && r.status == RowStatus::Idle)
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
                    KeyCode::Enter => {
                        if !selected.is_empty() {
                            mode = UiMode::Confirming;
                        }
                    }
                    KeyCode::Char('q') => {
                        selected.clear();
                        break;
                    }
                    _ => {}
                },
                UiMode::Confirming => match k.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        let mut queued = Vec::new();
                        for selected_idx in selected.iter().copied() {
                            if let Some(row) = rows.get_mut(selected_idx) {
                                if !row.is_header && row.status == RowStatus::Idle {
                                    row.status = RowStatus::Queued;
                                    if let Some(path) = row.path.clone() {
                                        queued.push(path);
                                    }
                                }
                            }
                        }
                        for path in queued {
                            let _ = delete_req_tx.send(DeleteRequest::Delete(path));
                        }
                        selected.clear();
                        mode = UiMode::Selecting;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        mode = UiMode::Selecting;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        mode = UiMode::Selecting;
                    }
                    _ => {}
                },
            },
            Event::Mouse(m) => match m.kind {
                MouseEventKind::ScrollDown => {
                    if mode == UiMode::Selecting && idx + 1 < rows.len() {
                        idx += 1;
                    }
                }
                MouseEventKind::ScrollUp => {
                    if mode == UiMode::Selecting {
                        idx = idx.saturating_sub(1);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        }
        spinner_idx = (spinner_idx + 1) % spinner_frames.len();
    }

    drop(delete_req_tx);
    let _ = delete_worker.join();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}

fn build_rows(targets: &[CacheTarget]) -> Vec<RowEntry> {
    let mut rows = Vec::new();
    for target in targets {
        if use_grouped_rows_for_target(&target.id) {
            if target.size_bytes == 0 || !target.path.exists() {
                continue;
            }
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
                status: RowStatus::Idle,
            });
            rows.push(RowEntry {
                text: truncate_middle(&target.path.display().to_string(), 86),
                category: target.id.clone(),
                path: Some(target.path.clone()),
                size_bytes: target.size_bytes,
                essential_warning: grouped_target_is_caution(&target.id),
                is_header: false,
                status: RowStatus::Idle,
            });
            continue;
        }

        let entries: Vec<DeletionEntry> =
            scanner::collect_deletion_entries_compact(target, UI_MIN_FILE_SIZE_BYTES);
        if entries.is_empty() {
            continue;
        }

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
            status: RowStatus::Idle,
        });
        for e in entries {
            rows.push(RowEntry {
                text: truncate_middle(&e.path.display().to_string(), 86),
                category: e.category,
                path: Some(e.path),
                size_bytes: e.size_bytes,
                essential_warning: e.is_essential_warning,
                is_header: false,
                status: RowStatus::Idle,
            });
        }
    }
    rows
}

fn use_grouped_rows_for_target(target_id: &str) -> bool {
    matches!(
        target_id,
        "go_build_cache"
            | "composer_cache"
            | "nuget_packages"
            | "nuget_http_cache"
            | "nuget_plugins_cache"
            | "gradle_caches"
            | "gradle_wrapper_dists"
            | "xcode_derived_data"
            | "swiftpm_cache"
            | "pub_cache"
            | "cabal_cache"
            | "stack_cache"
            | "hex_cache"
            | "ivy_cache"
            | "coursier_cache"
            | "sbt_boot_cache"
            | "pipenv_cache"
            | "huggingface_hub_cache"
            | "kube_cache"
            | "helm_cache"
    )
}

fn grouped_target_is_caution(target_id: &str) -> bool {
    matches!(
        target_id,
        "nuget_packages"
            | "gradle_caches"
            | "gradle_wrapper_dists"
            | "xcode_derived_data"
            | "swiftpm_cache"
            | "pub_cache"
            | "cabal_cache"
            | "stack_cache"
            | "ivy_cache"
            | "coursier_cache"
            | "sbt_boot_cache"
            | "huggingface_hub_cache"
    )
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

fn target_info(target_id: &str, essential_warning: bool) -> (&'static str, &'static str) {
    let purpose = match target_id {
        "npm_cache" => "NPM package download/build cache",
        "yarn_cache" => "Yarn package cache",
        "pnpm_store" => "pnpm shared store",
        "uv_cache" => "uv package/cache data",
        "pip_cache" => "pip wheel and package cache",
        "poetry_cache" => "Poetry dependency cache",
        "pipx_cache" => "pipx package cache",
        "cargo_registry" => "Cargo registry index and crates cache",
        "cargo_git" => "Cargo git checkout cache",
        "rustup_downloads" => "Rustup downloaded installer/cache files",
        "rustup_tmp" => "Rustup temporary files",
        "docker_cache" => "Docker local cache/state files",
        "wasp_cache" => "Wasp framework cache data",
        "venv_dirs" => "Project Python virtual environments (.venv)",
        "go_build_cache" => "Go build artifact cache",
        "composer_cache" => "Composer package download cache",
        "nuget_packages" => "NuGet global package cache",
        "nuget_http_cache" => "NuGet HTTP response cache",
        "nuget_plugins_cache" => "NuGet plugins cache",
        "gradle_caches" => "Gradle shared dependency/build cache",
        "gradle_wrapper_dists" => "Gradle wrapper downloaded distributions",
        "xcode_derived_data" => "Xcode DerivedData build cache",
        "swiftpm_cache" => "Swift package manager cache",
        "pub_cache" => "Dart/Flutter pub package cache",
        "cabal_cache" => "Cabal package cache",
        "stack_cache" => "Stack package cache",
        "hex_cache" => "Hex package cache",
        "ivy_cache" => "Ivy dependency cache",
        "coursier_cache" => "Coursier dependency cache",
        "sbt_boot_cache" => "sbt launcher/bootstrap cache",
        "pipenv_cache" => "Pipenv package cache",
        "huggingface_hub_cache" => "Hugging Face hub cache",
        "kube_cache" => "kubectl local discovery/http cache",
        "helm_cache" => "Helm chart/index cache",
        _ => "Tool cache data",
    };

    let safe_delete_note = if essential_warning {
        "caution: safe but may force heavy re-download/rebuild"
    } else {
        "generally safe to delete; tool recreates when needed"
    };

    (purpose, safe_delete_note)
}
