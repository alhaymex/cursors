use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, BrowserView, VisibleItem};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(2)])
        .split(area);

    draw_body(frame, chunks[0], app);
    draw_footer(frame, chunks[1], app);

    if app.show_help {
        draw_help(frame, centered_rect(74, 16, area), app);
    }
    if app.pending_action.is_some() {
        draw_confirmation(frame, centered_rect(68, 5, area), app);
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let status_style = if app.status.is_error {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default().fg(Color::LightGreen)
    };
    let view = match app.view {
        BrowserView::Themes => "themes",
        BrowserView::Backups => "backups",
    };
    let search_state = if app.filter_mode {
        format!("searching {}", app.filter)
    } else if app.filter.is_empty() {
        "/ search".to_string()
    } else {
        format!("search {}", app.filter)
    };

    let action_line = match app.view {
        BrowserView::Backups => Line::from(vec![
            Span::styled(&app.status.text, status_style),
            gap(),
            key_inline("j/k"),
            muted(" move "),
            key_inline("enter/u"),
            muted(" restore "),
            key_inline("/"),
            muted(" search "),
            key_inline("h/esc"),
            muted(" back "),
            key_inline("?"),
            muted(" help "),
            key_inline("q"),
            muted(" quit"),
        ]),
        BrowserView::Themes => Line::from(vec![
            Span::styled(&app.status.text, status_style),
            gap(),
            key_inline("j/k"),
            muted(" move "),
            key_inline("enter"),
            muted(" apply "),
            key_inline("space"),
            muted(" dir "),
            key_inline("b"),
            muted(" backups "),
            key_inline("/"),
            muted(" search "),
            key_inline("?"),
            muted(" help "),
            key_inline("q"),
            muted(" quit"),
        ]),
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "cursors",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            muted("  "),
            label("current"),
            value(app.current.theme.clone()),
            muted(format!(" @ {}", app.current.size)),
            gap(),
            label("size"),
            value(app.size.to_string()),
            gap(),
            label("view"),
            value(view),
            gap(),
            muted(format!("{} themes", app.themes.len())),
            gap(),
            muted(search_state),
        ]),
        action_line,
    ];

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &mut App) {
    if area.width >= 96 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(48), Constraint::Length(36)])
            .split(area);
        draw_list(frame, chunks[0], app);
        draw_details(frame, chunks[1], app);
    } else {
        draw_list(frame, area, app);
    }
}

fn draw_list(frame: &mut Frame, area: Rect, app: &mut App) {
    if matches!(app.view, BrowserView::Backups) {
        draw_backup_list(frame, area, app);
        return;
    }

    let items = app.visible_items();
    let rows = items
        .iter()
        .map(|item| row_for_item(app, *item))
        .collect::<Vec<_>>();

    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new("No cursor themes match the current search.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let mut state = ListState::default();
    state.select(Some(app.selected.min(rows.len() - 1)));

    let list = List::new(rows)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(45, 52, 64))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_backup_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let items = app.visible_backups();
    let rows = items
        .iter()
        .filter_map(|index| app.backups.get(*index))
        .map(|backup| {
            ListItem::new(Line::from(vec![
                Span::styled("~ ", Style::default().fg(Color::LightBlue)),
                Span::styled(
                    backup.theme.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                muted(format!("    size {}", backup.size)),
                muted(format!("    {} files", backup.files.len())),
                muted(format!("    {}", backup.id)),
            ]))
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new("No backups found.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let mut state = ListState::default();
    state.select(Some(app.selected.min(rows.len() - 1)));
    let list = List::new(rows)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(45, 52, 64))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn row_for_item(app: &App, item: VisibleItem) -> ListItem<'static> {
    match item {
        VisibleItem::Theme(index) => {
            let theme = &app.themes[index];
            let current = if app.current.theme == theme.name {
                "* "
            } else {
                "  "
            };
            ListItem::new(Line::from(vec![
                Span::styled(current, Style::default().fg(Color::Green)),
                Span::styled(
                    theme.name.clone(),
                    Style::default().fg(if app.current.theme == theme.name {
                        Color::LightGreen
                    } else {
                        Color::White
                    }),
                ),
            ]))
        }
    }
}

fn draw_details(frame: &mut Frame, area: Rect, app: &App) {
    if matches!(app.view, BrowserView::Backups) {
        draw_backup_details(frame, area, app);
        return;
    }

    let content = match app.selected_item() {
        Some(item) => {
            if let Some(theme) = app.theme_for_item(item) {
                vec![
                    Line::from(vec![label("theme"), value(theme.name.clone())]),
                    Line::raw(""),
                    Line::from(vec![
                        label("family guess"),
                        value(crate::theme::family_name(&theme.name)),
                    ]),
                    Line::raw(""),
                    Line::from(vec![label("source"), value(theme.source_kind.label())]),
                    Line::raw(""),
                    Line::from(vec![
                        label("path"),
                        value(theme.source_dir.display().to_string()),
                    ]),
                    Line::raw(""),
                    Line::from(vec![label("apply size"), value(app.size.to_string())]),
                    Line::raw(""),
                    Line::styled(
                        "Enter applies. Space opens the theme directory.",
                        Style::default().fg(Color::Gray),
                    ),
                ]
            } else {
                vec![Line::raw("No selection")]
            }
        }
        None => vec![Line::raw("No selection")],
    };

    frame.render_widget(Paragraph::new(content).wrap(Wrap { trim: true }), area);
}

fn draw_backup_details(frame: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(backup) = app.selected_backup() {
        let mut lines = vec![
            Line::from(vec![label("backup"), value(backup.id.clone())]),
            Line::raw(""),
            Line::from(vec![label("theme"), value(backup.theme.clone())]),
            Line::raw(""),
            Line::from(vec![label("size"), value(backup.size.to_string())]),
            Line::raw(""),
            Line::from(vec![label("files"), value(backup.files.len().to_string())]),
            Line::raw(""),
        ];
        for file in &backup.files {
            let state = if file.existed { "" } else { " (missing)" };
            lines.push(Line::from(vec![
                muted(format!("{} -> ", file.target)),
                value(format!("{}{}", file.original_path.display(), state)),
            ]));
        }
        lines.push(Line::raw(""));
        lines.push(Line::styled(
            "Press Enter or u to restore this backup.",
            Style::default().fg(Color::Gray),
        ));
        lines
    } else {
        vec![Line::raw("No backup selected")]
    };

    frame.render_widget(Paragraph::new(content).wrap(Wrap { trim: true }), area);
}

fn draw_help(frame: &mut Frame, area: Rect, app: &App) {
    let lines = if matches!(app.view, BrowserView::Backups) {
        vec![
            Line::from(vec![key("j/down"), Span::raw(" move down")]),
            Line::from(vec![key("k/up"), Span::raw(" move up")]),
            Line::from(vec![key("enter/u"), Span::raw(" restore selected backup")]),
            Line::from(vec![key("/"), Span::raw(" search backups")]),
            Line::from(vec![key("h/backspace"), Span::raw(" return to themes")]),
            Line::from(vec![
                key("esc"),
                Span::raw(" clear search or return to themes"),
            ]),
            Line::from(vec![key("q"), Span::raw(" quit")]),
        ]
    } else {
        vec![
            Line::from(vec![key("j/down"), Span::raw(" move down")]),
            Line::from(vec![key("k/up"), Span::raw(" move up")]),
            Line::from(vec![key("enter"), Span::raw(" apply selected theme")]),
            Line::from(vec![key("/"), Span::raw(" search themes")]),
            Line::from(vec![key("esc"), Span::raw(" clear search")]),
            Line::from(vec![key("+/-"), Span::raw(" change cursor size")]),
            Line::from(vec![key("d"), Span::raw(" reset size to 24")]),
            Line::from(vec![key("r"), Span::raw(" rescan themes")]),
            Line::from(vec![key("b"), Span::raw(" open backup page")]),
            Line::from(vec![
                key("space"),
                Span::raw(" open selected theme directory"),
            ]),
            Line::from(vec![key("q"), Span::raw(" quit")]),
        ]
    };

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(Color::Black)),
        area,
    );
}

fn draw_confirmation(frame: &mut Frame, area: Rect, app: &App) {
    let message = match &app.pending_action {
        Some(crate::app::PendingAction::ApplyTheme { theme_index, size }) => app
            .themes
            .get(*theme_index)
            .map(|theme| format!("Apply {} @ {}?", theme.name, size))
            .unwrap_or_else(|| "Apply selected theme?".to_string()),
        Some(crate::app::PendingAction::RestoreBackup { backup_index }) => app
            .backups
            .get(*backup_index)
            .map(|backup| format!("Restore {} @ {}?", backup.theme, backup.size))
            .unwrap_or_else(|| "Restore selected backup?".to_string()),
        None => return,
    };

    let lines = vec![
        Line::styled(
            message,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        Line::from(vec![
            key_inline("y/enter"),
            muted(" confirm    "),
            key_inline("n/esc"),
            muted(" cancel"),
        ]),
    ];

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(Color::Black)),
        area,
    );
}

fn label(text: impl Into<String>) -> Span<'static> {
    Span::styled(
        format!("{}: ", text.into()),
        Style::default().fg(Color::Gray),
    )
}

fn value(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(Color::White))
}

fn muted(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(Color::DarkGray))
}

fn key(text: impl Into<String>) -> Span<'static> {
    Span::styled(
        format!("{:<12}", text.into()),
        Style::default()
            .fg(Color::LightBlue)
            .add_modifier(Modifier::BOLD),
    )
}

fn key_inline(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(Color::LightBlue))
}

fn gap() -> Span<'static> {
    Span::raw("    ")
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}
