use ratatui::{
    layout::{Alignment, Rect},
    prelude::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::ui::Theme;
use crate::ui::widgets::block::semantic_modal_title;

pub(crate) fn centered_dialog_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

pub(crate) fn warning_dialog<'a>(
    title: &'static str,
    message: &'static str,
    detail: &'static str,
    shortcuts: Line<'a>,
    theme: Theme,
    hit: (&crate::App, Rect),
) -> Paragraph<'a> {
    let content = hit.1.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    if content.height > 3 {
        crate::ui::footer::register_shortcut_text(
            hit.0,
            Rect::new(content.x, content.y + 3, content.width, 1),
            &Text::from(shortcuts.clone()),
            Alignment::Center,
        );
    }
    let lines = Text::from(vec![
        Line::from(Span::styled(message, Style::default().fg(theme.text))),
        Line::from(Span::styled(detail, Style::default().fg(theme.text))),
        Line::from(""),
        shortcuts,
    ]);

    Paragraph::new(lines)
        .block(warning_block(title, theme))
        .alignment(Alignment::Center)
}

pub(crate) fn warning_message_dialog<'a>(
    title: &'static str,
    message: &'static str,
    shortcuts: Line<'a>,
    theme: Theme,
    hit: (&crate::App, Rect),
) -> Paragraph<'a> {
    let content = hit.1.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    if content.height > 2 {
        crate::ui::footer::register_shortcut_text(
            hit.0,
            Rect::new(content.x, content.y + 2, content.width, 1),
            &Text::from(shortcuts.clone()),
            Alignment::Center,
        );
    }
    let lines = Text::from(vec![
        Line::from(Span::styled(message, Style::default().fg(theme.text))),
        Line::from(""),
        shortcuts,
    ]);

    Paragraph::new(lines)
        .block(warning_block(title, theme))
        .alignment(Alignment::Center)
}

pub(crate) fn warning_block(title: &'static str, theme: Theme) -> Block<'static> {
    Block::default()
        .title(semantic_modal_title(title, theme.warning, theme))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.panel_alt))
}
