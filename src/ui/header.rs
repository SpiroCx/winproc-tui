use crate::{
    App,
    app::{AppActivity, SampleFreshness, state::MainMenuSection},
    ui::Theme,
};
use ratatui::{
    layout::{Alignment, Rect},
    prelude::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

const SPINNER: [char; 4] = ['|', '/', '-', '\\'];

pub(crate) fn draw_header(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App, theme: Theme) {
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(theme.panel)),
        area,
    );
    let actions = header_actions(area, app);
    for (action, rect) in &actions {
        let active = action
            .section()
            .is_some_and(|section| app.is_main_menu_open() && app.main_menu_section == section)
            || (*action == HeaderAction::More
                && app.is_main_menu_open()
                && !actions
                    .iter()
                    .any(|(action, _)| action.section() == Some(app.main_menu_section)));
        let selected = match action {
            HeaderAction::Processes => !app.network_browser.visible && !app.file_users.visible,
            HeaderAction::Network => app.network_browser.visible,
            HeaderAction::FileUsers => app.file_users.visible,
            _ => false,
        };
        let hovered = app.header_action_hovered == Some(*action);
        let style = Style::default()
            .fg(if selected {
                theme.focus_border
            } else {
                theme.text
            })
            .bg(if hovered {
                theme.focus_surface
            } else if active {
                theme.table_selection_surface
            } else {
                theme.panel
            })
            .add_modifier(if active || hovered || selected {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });
        let spans = action
            .label()
            .chars()
            .enumerate()
            .map(|(index, ch)| {
                Span::styled(
                    ch.to_string(),
                    if action.access_index() == Some(index) {
                        style.fg(theme.key_hint).add_modifier(Modifier::UNDERLINED)
                    } else {
                        style
                    },
                )
            })
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(Line::from(spans)).style(style), *rect);
    }
    let status_x = actions
        .last()
        .map_or(area.x, |(_, rect)| rect.right().saturating_add(2))
        .min(area.right());
    let status_area = Rect::new(
        status_x,
        area.y,
        area.right().saturating_sub(status_x),
        area.height,
    );
    let mut spans = header_status_spans(app, theme);
    let budget = usize::from(status_area.width);
    let profile = if app.activity() == AppActivity::LogView {
        "PF: --".to_string()
    } else {
        format!(
            "PF: {}{}",
            app.active_investigation_profile
                .as_deref()
                .unwrap_or("none"),
            if app.active_investigation_profile.is_some()
                && app.active_investigation_profile_dirty()
            {
                "*"
            } else {
                ""
            }
        )
    };
    let profile_budget = budget.min(spans.iter().map(Span::width).sum::<usize>() + 30);
    let remaining = profile_budget.saturating_sub(spans.iter().map(Span::width).sum::<usize>());
    if remaining >= 8 {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!(" {} ", truncate_name(&profile, remaining - 4)),
            Style::default()
                .fg(ratatui::style::Color::Black)
                .bg(theme.muted),
        ));
    }
    if let Some(path) = app.active_log_path() {
        append_fitting_label(
            &mut spans,
            &path
                .file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy(),
            budget,
            theme.muted,
        );
    }
    // Clip status by terminal cells, retaining activity before optional names.
    let mut remaining = budget;
    let spans = spans
        .into_iter()
        .map(|span| {
            let text = truncate_cells(&span.content, remaining);
            remaining = remaining.saturating_sub(Span::raw(&text).width());
            Span::styled(text, span.style)
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Right)
            .style(Style::default().bg(theme.panel)),
        status_area,
    );
}

fn append_fitting_label(
    spans: &mut Vec<Span<'static>>,
    text: &str,
    budget: usize,
    color: ratatui::style::Color,
) {
    let remaining = budget.saturating_sub(spans.iter().map(Span::width).sum::<usize>());
    if remaining >= 6 {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            truncate_name(text, remaining - 2),
            Style::default().fg(color),
        ));
    }
}

fn truncate_name(text: &str, width: usize) -> String {
    if Span::raw(text).width() <= width {
        return text.to_string();
    }
    if width < 5 {
        return truncate_cells(text, width);
    }
    let tail_budget = ((width - 1) / 2).max(4);
    let mut tail = String::new();
    let mut used = 0;
    for ch in text.chars().rev() {
        let cells = Span::raw(ch.to_string()).width();
        if used + cells > tail_budget {
            break;
        }
        tail.insert(0, ch);
        used += cells;
    }
    format!("{}{}", truncate_cells(text, width - used), tail)
}

fn truncate_cells(text: &str, width: usize) -> String {
    if Span::raw(text).width() <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let cells = Span::raw(ch.to_string()).width();
        if used + cells > width - 1 {
            break;
        }
        out.push(ch);
        used += cells;
    }
    out.push('…');
    out
}

fn header_status_spans(app: &App, theme: Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    let activity = app.activity();
    spans.push(mode_span(
        activity_label(activity),
        match activity {
            AppActivity::Live => theme.active_series,
            AppActivity::Recording => theme.danger,
            AppActivity::LogView => theme.warning,
        },
        theme,
    ));
    match activity {
        AppActivity::Live => {}
        AppActivity::Recording => {
            if let Some(interval_seconds) = app
                .active_recording_interval_seconds()
                .filter(|interval| *interval > 1)
            {
                append_recording_interval(&mut spans, interval_seconds, theme);
            }
            {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    SPINNER[app.recording_spinner_index % SPINNER.len()].to_string(),
                    Style::default()
                        .fg(theme.danger)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }
        AppActivity::LogView => {
            if let Some(interval_seconds) = app.log_view_interval_seconds {
                append_recording_interval(&mut spans, interval_seconds, theme);
            }
        }
    }

    if let Some(SampleFreshness::Stale { age_seconds }) = app.sample_freshness() {
        spans.push(stale_span(age_seconds, theme));
    }
    if app.is_display_paused() && activity != AppActivity::LogView {
        spans.push(Span::raw("  "));
        spans.push(mode_span("DISPLAY PAUSED", theme.warning, theme));
    }
    spans
}

fn activity_label(activity: AppActivity) -> &'static str {
    match activity {
        AppActivity::Live => "LIVE",
        AppActivity::Recording => "REC",
        AppActivity::LogView => "LOG",
    }
}

fn append_recording_interval(spans: &mut Vec<Span<'static>>, seconds: u64, theme: Theme) {
    spans.push(Span::raw(" · "));
    spans.push(Span::styled(
        if seconds > 1 {
            format!("{seconds}s AVG")
        } else {
            "1s".to_string()
        },
        Style::default()
            .fg(theme.muted)
            .add_modifier(Modifier::BOLD),
    ));
}

fn stale_span(age_seconds: u64, theme: Theme) -> Span<'static> {
    Span::styled(
        format!(" · STALE {age_seconds}s"),
        Style::default()
            .fg(theme.warning)
            .add_modifier(Modifier::BOLD),
    )
}

fn mode_span(label: &'static str, color: ratatui::prelude::Color, theme: Theme) -> Span<'static> {
    Span::styled(
        format!(" {label} "),
        Style::default()
            .fg(theme.background)
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeaderAction {
    Session,
    Profile,
    View,
    Tools,
    Settings,
    Help,
    Processes,
    Network,
    FileUsers,
    More,
}
impl HeaderAction {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Session => " Session ▾ ",
            Self::Profile => " Profile ▾ ",
            Self::View => " View ▾ ",
            Self::Tools => " Tools ▾ ",
            Self::Settings => " Settings ▾ ",
            Self::Help => " Help ",
            Self::Processes => "[Processes]",
            Self::Network => "[Network]",
            Self::FileUsers => "[Find by file]",
            Self::More => " More ▾ ",
        }
    }
    pub(crate) fn menu_label(self) -> &'static str {
        match self {
            Self::FileUsers => "Find processes by file",
            Self::Session => "Session",
            Self::Profile => "Profile",
            Self::View => "View",
            Self::Tools => "Tools",
            Self::Settings => "Settings",
            Self::Help => "Help",
            Self::Processes => "Processes",
            Self::Network => "Network endpoints",
            Self::More => "More",
        }
    }
    pub(crate) fn section(self) -> Option<MainMenuSection> {
        match self {
            Self::Session => Some(MainMenuSection::Session),
            Self::Profile => Some(MainMenuSection::Profile),
            Self::View => Some(MainMenuSection::View),
            Self::Tools => Some(MainMenuSection::Tools),
            Self::Settings => Some(MainMenuSection::Settings),
            Self::More => Some(MainMenuSection::More),
            _ => None,
        }
    }
    fn access_index(self) -> Option<usize> {
        match self {
            Self::Session | Self::Profile | Self::View => Some(1),
            Self::Tools => Some(2),
            Self::Settings => Some(3),
            _ => None,
        }
    }
    pub(crate) fn shortcut(self) -> &'static str {
        match self {
            Self::Session => "Alt+S",
            Self::Profile => "Alt+P",
            Self::View => "Alt+V",
            Self::Tools => "Alt+O",
            Self::Settings => "Alt+T",
            Self::Help => "F1",
            Self::Processes => "F2",
            Self::Network => "F3",
            Self::FileUsers => "F4",
            Self::More => "",
        }
    }
}

fn available_actions(app: &App) -> Vec<HeaderAction> {
    use HeaderAction::*;
    [Session, Profile, View, Tools, Settings, Help]
        .into_iter()
        .filter(|action| app.activity() != AppActivity::LogView || *action != Tools)
        .collect()
}

pub(crate) fn header_actions(area: Rect, app: &App) -> Vec<(HeaderAction, Rect)> {
    if area.is_empty() {
        return Vec::new();
    }
    // Width depends only on terminal geometry, never profile names or activity badges.
    let reserved = (area.width / 3)
        .clamp(40, 48)
        .min(area.width.saturating_sub(12));
    let end = area.right().saturating_sub(reserved);
    let available = available_actions(app);
    let total = available
        .iter()
        .map(|action| Span::raw(action.label()).width() as u16 + 1)
        .sum::<u16>();
    let overflow = total > end.saturating_sub(area.x);
    let more_width = Span::raw(HeaderAction::More.label()).width() as u16;
    let limit = if overflow {
        end.saturating_sub(more_width + 1)
    } else {
        end
    };
    let mut x = area.x;
    let mut actions = Vec::new();
    for action in available {
        let width = Span::raw(action.label()).width() as u16;
        if x.saturating_add(width) > limit && action != HeaderAction::Session {
            break;
        }
        let width = width.min(area.right().saturating_sub(x));
        actions.push((action, Rect::new(x, area.y, width, 1)));
        x = x.saturating_add(width + 1);
    }
    if overflow && x + more_width + 8 <= area.right() {
        actions.push((HeaderAction::More, Rect::new(x, area.y, more_width, 1)));
    }
    actions
}

pub(crate) fn overflow_actions(screen: Rect, app: &App) -> Vec<HeaderAction> {
    let visible = header_actions(crate::ui::screen_layout(screen)[0], app);
    available_actions(app)
        .into_iter()
        .filter(|action| !visible.iter().any(|(shown, _)| shown == action))
        .collect()
}

pub(crate) fn header_action_at(screen: Rect, app: &App, x: u16, y: u16) -> Option<HeaderAction> {
    header_actions(crate::ui::screen_layout(screen)[0], app)
        .into_iter()
        .find(|(_, rect)| rect.contains(ratatui::layout::Position::new(x, y)))
        .map(|(action, _)| action)
}
