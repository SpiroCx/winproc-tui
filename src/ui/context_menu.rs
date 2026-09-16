use crate::{App, app::context_menu::ContextMenu};
use ratatui::{
    layout::{Position, Rect},
    prelude::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub(crate) struct MenuLayout {
    pub(crate) area: Rect,
    content: Rect,
    offset: usize,
}
pub(crate) fn layout(screen: Rect, menu: &ContextMenu) -> MenuLayout {
    let width = (Line::from(menu.title.as_str()).width().max(
        menu.items
            .iter()
            .map(|i| Line::from(i.label.as_str()).width() + 2 + if i.enabled { 0 } else { 14 })
            .max()
            .unwrap_or(0),
    ) + 2)
        .min(88)
        .min(screen.width as usize) as u16;
    let height = (menu.items.len() as u16 + 2).min(screen.height);
    let area = Rect::new(
        menu.anchor
            .x
            .min(screen.right().saturating_sub(width))
            .max(screen.x),
        menu.anchor
            .y
            .min(screen.bottom().saturating_sub(height))
            .max(screen.y),
        width,
        height,
    );
    let content = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    let offset = menu
        .selected
        .saturating_add(1)
        .saturating_sub(content.height as usize);
    MenuLayout {
        area,
        content,
        offset,
    }
}
pub(crate) fn item_at(screen: Rect, menu: &ContextMenu, x: u16, y: u16) -> Option<usize> {
    let layout = layout(screen, menu);
    layout
        .content
        .contains(Position::new(x, y))
        .then(|| layout.offset + (y - layout.content.y) as usize)
        .filter(|i| *i < menu.items.len())
}
pub(crate) fn draw(frame: &mut ratatui::Frame<'_>, app: &App) {
    let Some(menu) = app.context_menu.as_ref() else {
        return;
    };
    let layout = layout(frame.area(), menu);
    let theme = app.theme();
    let rows = menu
        .items
        .iter()
        .enumerate()
        .skip(layout.offset)
        .take(layout.content.height as usize)
        .map(|(index, item)| {
            let selected = index == menu.selected;
            let hovered = menu.hovered == Some(index);
            let style = Style::default()
                .fg(if item.enabled {
                    theme.text
                } else {
                    theme.muted
                })
                .bg(if hovered {
                    theme.focus_surface
                } else if selected {
                    theme.table_selection_surface
                } else {
                    theme.panel_alt
                })
                .add_modifier(if selected || hovered {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });
            Line::styled(
                format!(
                    "{} {}{}",
                    if selected { ">" } else { " " },
                    item.label,
                    if item.enabled { "" } else { " (unavailable)" }
                ),
                style,
            )
        })
        .collect::<Vec<_>>();
    frame.render_widget(Clear, layout.area);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(menu.title.as_str())
            .border_style(Style::default().fg(theme.focus_border))
            .style(Style::default().bg(theme.panel_alt)),
        layout.area,
    );
    frame.render_widget(Paragraph::new(Text::from(rows)), layout.content);
}
