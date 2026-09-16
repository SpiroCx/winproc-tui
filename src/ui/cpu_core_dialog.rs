use ratatui::{
    layout::Rect,
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::{
    App,
    app::AppActivity,
    model::{CpuCoreKind, CpuLogicalProcessorSample},
    ui::{Theme, footer::shortcut_spans, widgets::scrollable_modal::ScrollableModal},
};

const FOOTER_ITEMS: [(&str, &str); 4] = [
    ("↑/↓", "Rows"),
    ("PgUp/PgDn", "Page"),
    ("Home/End", "Edge"),
    ("Enter/Esc", "Close"),
];
const FOOTER_HEIGHT: u16 = 1;
const MAX_COLUMNS: usize = 4;
const COLUMN_GAP: &str = "   ";

pub(crate) fn draw_cpu_core_dialog(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    app: &App,
    theme: Theme,
) {
    let (modal, columns) = cpu_core_dialog_grid(area, app);
    let lines = cpu_core_dialog_lines(app, columns, theme);
    let layout = modal.render(
        frame,
        area,
        Text::from(lines),
        app.cpu_core_scroll.offset,
        false,
        theme,
    );
    if !layout.footer.is_empty() {
        crate::ui::footer::register_shortcut_text(
            app,
            layout.footer,
            &ratatui::text::Text::from(Line::from(shortcut_spans(&FOOTER_ITEMS, theme))),
            ratatui::layout::Alignment::Left,
        );
        frame.render_widget(
            Paragraph::new(Line::from(shortcut_spans(&FOOTER_ITEMS, theme))),
            layout.footer,
        );
    }
}

pub(crate) fn cpu_core_dialog_page_size_for_screen(area: Rect, app: &App) -> usize {
    cpu_core_dialog_grid(area, app).0.page_size(area)
}

pub(crate) fn cpu_core_dialog_content_area(area: Rect, app: &App) -> Rect {
    cpu_core_dialog_grid(area, app).0.layout(area).content
}

pub(crate) fn cpu_core_dialog_scrollbar_area(
    area: Rect,
    app: &App,
    page_size: usize,
) -> Option<Rect> {
    cpu_core_dialog_grid(area, app)
        .0
        .scrollbar_area(area, page_size)
}

pub(crate) fn cpu_core_dialog_total_rows(area: Rect, app: &App) -> usize {
    cpu_core_dialog_grid(area, app).0.content_height as usize
}

fn cpu_core_dialog_lines(app: &App, columns: usize, theme: Theme) -> Vec<Line<'static>> {
    let cores = &app.display_snapshot().cpu_logical_processors;
    if cores.is_empty() {
        let message = if app.activity() == AppActivity::LogView {
            "Per-core usage is not recorded in logs."
        } else {
            "Per-core usage is unavailable."
        };
        return vec![Line::from(Span::styled(
            message,
            ratatui::style::Style::default().fg(theme.muted),
        ))];
    }

    let index_width = cores.len().saturating_sub(1).to_string().len().max(1);
    cores
        .chunks(columns)
        .enumerate()
        .map(|(row, cores)| {
            let mut spans = Vec::new();
            for (column, core) in cores.iter().enumerate() {
                if column > 0 {
                    spans.push(Span::raw(COLUMN_GAP));
                }
                spans
                    .extend(cpu_core_line(row * columns + column, index_width, *core, theme).spans);
            }
            Line::from(spans)
        })
        .collect()
}

fn cpu_core_line(
    index: usize,
    index_width: usize,
    core: CpuLogicalProcessorSample,
    theme: Theme,
) -> Line<'static> {
    let kind = match core.kind {
        Some(CpuCoreKind::Performance) => "P",
        Some(CpuCoreKind::Efficiency) => "E",
        None => "-",
    };
    Line::from(vec![
        Span::styled(
            format!("CPU {index:>index_width$}"),
            ratatui::style::Style::default().fg(theme.text),
        ),
        Span::styled(
            format!(" ({kind})  "),
            ratatui::style::Style::default().fg(theme.muted),
        ),
        Span::styled(
            format!("{:>3}%", core.usage_percent.min(100)),
            ratatui::style::Style::default().fg(theme.text),
        ),
    ])
}

fn cpu_core_dialog_grid(area: Rect, app: &App) -> (ScrollableModal, usize) {
    let core_count = app.display_snapshot().cpu_logical_processors.len();
    let index_width = core_count.saturating_sub(1).to_string().len();
    let cell_width = format!("CPU {:>index_width$} (-)  100%", 0).len();
    let preferred_columns = core_count.clamp(1, MAX_COLUMNS);
    let grid_width = preferred_columns * cell_width + (preferred_columns - 1) * COLUMN_GAP.len();
    let mut modal = ScrollableModal::new(
        "PER-CORE CPU USAGE",
        grid_width.max(footer_width()).max(39) as u16,
        1,
        FOOTER_HEIGHT,
    );
    let available_width = modal.layout(area).content.width as usize;
    let columns = ((available_width + COLUMN_GAP.len()) / (cell_width + COLUMN_GAP.len()))
        .clamp(1, preferred_columns);
    modal.content_height = core_count.div_ceil(columns).clamp(1, u16::MAX as usize) as u16;
    (modal, columns)
}

fn footer_width() -> usize {
    FOOTER_ITEMS
        .iter()
        .enumerate()
        .map(|(index, (key, label))| {
            usize::from(index > 0) * 2 + key.chars().count() + 1 + label.chars().count()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_cpu_row_uses_index_kind_and_percent() {
        let line = cpu_core_line(
            7,
            2,
            CpuLogicalProcessorSample {
                usage_percent: 42,
                kind: Some(CpuCoreKind::Efficiency),
            },
            crate::ui::THEMES[0],
        );
        let text = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(text, "CPU  7 (E)   42%");
    }
}
