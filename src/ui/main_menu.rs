use ratatui::{
    layout::Rect,
    prelude::{Modifier, Style},
    text::{Line, Span, Text},
};

use crate::{
    App,
    app::state::{MainMenuAction, MainMenuItem, MainMenuRow},
    ui::{Theme, widgets::scrollable_modal::ScrollableModal},
};

const HEADER_HEIGHT: u16 = 1;
const MENU_KEYS: &[(&str, &str)] = &[
    ("↑/↓", "Select"),
    ("←/→", "Expand"),
    ("Enter", "Open"),
    ("Esc", "Close"),
];

pub(crate) fn draw_main_menu(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App, theme: Theme) {
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
            let indent = "  ".repeat(usize::from(row.depth));
            let label = format!("{cursor}{indent}{}", display_label(app, *row));
            Line::from(Span::styled(format!("{label:<content_width$}"), style))
        })
        .collect::<Vec<_>>();

    let layout = main_menu_modal(app).render(frame, area, Text::from(lines), 0, false, theme);
    frame.render_widget(
        ratatui::widgets::Paragraph::new(Line::from(super::footer::shortcut_spans(
            MENU_KEYS, theme,
        ))),
        layout.footer,
    );
}

pub(crate) fn main_menu_index_at(area: Rect, app: &App, x: u16, y: u16) -> Option<usize> {
    let content = main_menu_modal(app).layout(area).content;
    if x < content.x || x >= content.right() || y < content.y || y >= content.bottom() {
        return None;
    }
    let index = usize::from(y - content.y);
    (index < app.main_menu_rows().len()).then_some(index)
}

#[cfg(test)]
pub(crate) fn main_menu_area(area: Rect, app: &App) -> Rect {
    main_menu_modal(app).area(area)
}

#[cfg(test)]
pub(crate) fn main_menu_item_area(area: Rect, app: &App, index: usize) -> Option<Rect> {
    let content = main_menu_modal(app).layout(area).content;
    (index < app.main_menu_rows().len() && index < usize::from(content.height)).then(|| {
        Rect::new(
            content.x,
            content.y.saturating_add(index as u16),
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

fn main_menu_content_width(app: &App) -> u16 {
    let activity = app.activity();
    activity
        .main_menu_items()
        .iter()
        .flat_map(|item| {
            let root = MainMenuRow {
                item: *item,
                depth: 0,
            };
            let children = match item {
                MainMenuItem::Section(section) => section.actions(activity),
                MainMenuItem::Action(_) => &[],
            };
            std::iter::once(root).chain(children.iter().copied().map(|action| MainMenuRow {
                item: MainMenuItem::Action(action),
                depth: 1,
            }))
        })
        .map(|row| 2 + usize::from(row.depth) * 2 + display_label(app, row).chars().count())
        .max()
        .unwrap_or_default()
        .max(52)
        .min(usize::from(u16::MAX)) as u16
}

fn display_label(app: &App, row: MainMenuRow) -> String {
    let mut label = app.main_menu_row_label(row);
    let MainMenuItem::Action(action) = row.item else {
        return label;
    };
    let key = match action {
        MainMenuAction::ProcessInfo => "Enter (Processes)",
        MainMenuAction::ProcessFiles => "f (Processes)",
        MainMenuAction::OpenProfiles => "Ctrl+T",
        MainMenuAction::SaveProfile => "Ctrl+S",
        MainMenuAction::SaveProfileAs => "Ctrl+Shift+S",
        MainMenuAction::OpenColumns => "c (Processes)",
        MainMenuAction::ToggleTrackedOnly => "Shift+T",
        MainMenuAction::ToggleTreeView => "v (Processes)",
        MainMenuAction::StartRecording | MainMenuAction::StopRecording => "Ctrl+R",
        MainMenuAction::ReturnToLive => "Ctrl+B",
        MainMenuAction::OpenLog => "Ctrl+L",
        MainMenuAction::Help => "F1",
        MainMenuAction::QuitImmediately | MainMenuAction::QuitWithConfirmation => "q",
        _ => "",
    };
    if matches!(
        action,
        MainMenuAction::ProcessInfo | MainMenuAction::ProcessFiles
    ) {
        if let Some(process) = app.selected_visible_process() {
            let name: String = process.name.chars().take(24).collect();
            label.push_str(&format!(" · PID {} {name}", process.pid));
        } else {
            label.push_str(" · no process selected");
        }
    }
    if !key.is_empty() {
        label.push_str(&format!("  {key}"));
    }
    label
}
