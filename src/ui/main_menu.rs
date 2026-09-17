use ratatui::{
    layout::Rect,
    prelude::{Modifier, Style},
    text::{Line, Span, Text},
};

use crate::{
    App,
    app::state::{MainMenuAction, MainMenuItem, MainMenuRow, MainMenuSection},
    ui::{Theme, widgets::scrollable_modal::ScrollableModal},
};

const HEADER_HEIGHT: u16 = 1;
const MENU_KEYS: &[(&str, &str)] = &[
    ("↑/↓", "Select"),
    ("←/→", "Menu"),
    ("Enter", "Open"),
    ("Esc", "Close"),
];

pub(crate) fn draw_main_menu(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App, theme: Theme) {
    if app.main_menu_section == MainMenuSection::Help {
        // Help is a direct action: focus the heading without creating an empty dropdown.
        let footer = Rect::new(
            area.x,
            area.bottom().saturating_sub(1),
            area.width,
            area.height.min(1),
        );
        let line = Line::from(super::footer::shortcut_spans(
            &[("←/→", "Menu"), ("Enter", "Help"), ("Esc", "Close")],
            theme,
        ));
        super::footer::register_shortcut_text(
            app,
            footer,
            &Text::from(line.clone()),
            ratatui::layout::Alignment::Left,
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(line).style(Style::default().bg(theme.panel)),
            footer,
        );
        return;
    }
    let content_width = usize::from(main_menu_content_width(app));
    let lines = app
        .main_menu_rows()
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let cursor = if index == app.main_menu_selected {
                "> "
            } else {
                "  "
            };
            let style = if index == app.main_menu_selected {
                Style::default()
                    .fg(theme.text)
                    .bg(theme.table_selection_surface)
                    .add_modifier(Modifier::BOLD)
            } else if app.main_menu_hovered == Some(index) {
                Style::default()
                    .fg(theme.text)
                    .bg(theme.focus_surface)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            let mut line = display_label(app, *row, theme, content_width.saturating_sub(2));
            line.spans.insert(0, Span::raw(cursor));
            line.spans.push(Span::raw(
                " ".repeat(content_width.saturating_sub(line.width())),
            ));
            line.style(style)
        })
        .collect::<Vec<_>>();

    let layout = main_menu_modal(app).render(
        frame,
        anchored_area(area, app),
        Text::from(lines),
        menu_offset(area, app),
        false,
        theme,
    );
    crate::ui::footer::register_shortcut_text(
        app,
        layout.footer,
        &ratatui::text::Text::from(Line::from(super::footer::shortcut_spans(MENU_KEYS, theme))),
        ratatui::layout::Alignment::Left,
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new(Line::from(super::footer::shortcut_spans(
            MENU_KEYS, theme,
        ))),
        layout.footer,
    );
}

pub(crate) fn main_menu_index_at(area: Rect, app: &App, x: u16, y: u16) -> Option<usize> {
    let content = main_menu_modal(app)
        .layout(anchored_area(area, app))
        .content;
    if x < content.x || x >= content.right() || y < content.y || y >= content.bottom() {
        return None;
    }
    let index = menu_offset(area, app) + usize::from(y - content.y);
    (index < app.main_menu_rows().len()).then_some(index)
}

pub(crate) fn main_menu_area(area: Rect, app: &App) -> Rect {
    if app.main_menu_section == MainMenuSection::Help {
        return Rect::default();
    }
    main_menu_modal(app).layout(anchored_area(area, app)).area
}

#[cfg(test)]
pub(crate) fn main_menu_item_area(area: Rect, app: &App, index: usize) -> Option<Rect> {
    let content = main_menu_modal(app)
        .layout(anchored_area(area, app))
        .content;
    let offset = menu_offset(area, app);
    (index >= offset
        && index < app.main_menu_rows().len()
        && index - offset < usize::from(content.height))
    .then(|| {
        Rect::new(
            content.x,
            content.y.saturating_add((index - offset) as u16),
            content.width,
            1,
        )
    })
}

fn main_menu_modal(app: &App) -> ScrollableModal {
    ScrollableModal::new(
        "",
        main_menu_content_width(app),
        app.main_menu_rows().len().min(usize::from(u16::MAX)) as u16,
        1,
    )
    .with_top_left_placement(HEADER_HEIGHT)
}

fn anchored_area(area: Rect, app: &App) -> Rect {
    let width = main_menu_modal(app).layout(area).area.width;
    let actions = super::header::header_actions(crate::ui::screen_layout(area)[0], app);
    let x = actions
        .iter()
        .find(|(action, _)| action.section() == Some(app.main_menu_section))
        .or_else(|| {
            actions
                .iter()
                .find(|(action, _)| *action == super::header::HeaderAction::More)
        })
        .map_or(area.x, |(_, rect)| rect.x)
        .min(area.right().saturating_sub(width));
    Rect::new(x, area.y, area.right().saturating_sub(x), area.height)
}

fn menu_offset(area: Rect, app: &App) -> usize {
    let height = main_menu_modal(app)
        .layout(anchored_area(area, app))
        .content
        .height
        .max(1) as usize;
    app.main_menu_selected.saturating_sub(height - 1)
}

fn main_menu_content_width(app: &App) -> u16 {
    app.main_menu_rows()
        .iter()
        .map(|row| 2 + app.main_menu_row_label(*row).len() + 2 + shortcut(*row).len())
        .max()
        .unwrap_or(0)
        .max(43)
        .min(usize::from(u16::MAX)) as u16
}

fn shortcut(row: MainMenuRow) -> &'static str {
    let MainMenuItem::Action(action) = row.item else {
        let MainMenuItem::Header(action) = row.item else {
            unreachable!()
        };
        return action.shortcut();
    };
    match action {
        MainMenuAction::OpenProfiles => "Ctrl+T",
        MainMenuAction::SaveProfile => "Ctrl+S",
        MainMenuAction::SaveProfileAs => "Ctrl+Shift+S",
        MainMenuAction::OpenColumns => "c (Processes)",
        MainMenuAction::ToggleTrackedOnly => "Shift+T",
        MainMenuAction::ToggleTreeView => "v (Processes)",
        MainMenuAction::ToggleGraphs => "g (Processes)",
        MainMenuAction::ToggleSamples => "v (Graphs)",
        MainMenuAction::ToggleDelta => "d (Graphs)",
        MainMenuAction::CycleGraphLayout => "l (Graphs)",
        MainMenuAction::StartRecording | MainMenuAction::StopRecording => "Ctrl+R",
        MainMenuAction::ReturnToLive => "Ctrl+B",
        MainMenuAction::OpenLog => "Ctrl+L",
        MainMenuAction::QuitImmediately | MainMenuAction::QuitWithConfirmation => "q",
        _ => "",
    }
}

fn display_label(app: &App, row: MainMenuRow, theme: Theme, width: usize) -> Line<'static> {
    let label = app.main_menu_row_label(row);
    let key = shortcut(row);
    let padding = width
        .saturating_sub(Span::raw(&label).width() + key.len())
        .max(2);
    let mut spans = vec![Span::raw(label)];
    if !key.is_empty() {
        spans.push(Span::raw(" ".repeat(padding)));
        let (key, context) = key.split_once(' ').unwrap_or((key, ""));
        spans.push(Span::styled(key, Style::default().fg(theme.key_hint)));
        if !context.is_empty() {
            spans.push(Span::raw(format!(" {context}")));
        }
    }
    Line::from(spans)
}
