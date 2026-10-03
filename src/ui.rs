use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect, Spacing},
    macros::{span, text},
    style::{Color, Modifier, Style, Stylize},
    symbols::line::THICK_TOP_LEFT,
    text::{self, Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Padding, Paragraph, Widget},
};

use crate::{Startup, StartupApp, utils::formatted_time};

#[derive(Debug)]
pub struct AppState {
    pub startup: Startup,
    pub mode: AppMode,
    pub component_state: ComponentState,
}

#[derive(Debug)]
pub enum AppMode {
    Normal,
    Command,
}

#[derive(Debug)]
pub struct ComponentState {
    pub profile_list_state: ListState,
}

pub fn render(frame: &mut Frame, app_state: &mut AppState) {
    let main_layout = Layout::default()
        .spacing(Spacing::Overlap(1))
        .constraints([Constraint::Length(4), Constraint::Fill(5)]);

    let [title_area, app_area] = main_layout.areas(frame.area());
    render_title_area(frame, title_area);
    render_app_area(frame, app_state, app_area);
}

// title layout

fn render_title_area(frame: &mut Frame, title_area: Rect) {
    let constraints = Constraint::from_fills([1, 1, 1]);
    let [_first, second, third] = title_area.layout(&Layout::horizontal(constraints));
    let text = text![
        Line::from("Startup App TUI").bold(),
        Line::from("Click q to exit").yellow()
    ];
    let title = Paragraph::new(text)
        .alignment(ratatui::layout::HorizontalAlignment::Center)
        .block(Block::bordered().merge_borders(ratatui::symbols::merge::MergeStrategy::Exact));
    frame.render_widget(title, second);
    frame.render_widget(
        Paragraph::new(format!("{:}", formatted_time()))
            .light_yellow()
            .alignment(ratatui::layout::HorizontalAlignment::Right)
            .block(Block::new().padding(Padding {
                top: 1,
                right: 2,
                left: 0,
                bottom: 0,
            })),
        third,
    );
}

// main layout
fn render_app_area(frame: &mut Frame, app_state: &mut AppState, app_area: Rect) {
    let constraints = [Constraint::Percentage(50), Constraint::Percentage(50)];

    let [profile_area, startup_apps_area] = Layout::horizontal(constraints)
        .spacing(Spacing::Overlap(1))
        .areas(app_area);

    render_list(frame, profile_area, app_state);
    render_child_list(frame, startup_apps_area, app_state);
}

pub fn render_list(frame: &mut Frame, area: Rect, app_state: &mut AppState) {
    let items: Vec<String> = app_state
        .startup
        .profiles
        .iter()
        .map(|p| p.profile.clone())
        .collect();
    let list = List::new(items)
        .style(Color::White)
        .highlight_style(Style::new().bg(Color::LightYellow).fg(Color::Black))
        .highlight_symbol("> ")
        .block(Block::bordered().merge_borders(ratatui::symbols::merge::MergeStrategy::Exact));

    frame.render_stateful_widget(
        list,
        area,
        &mut app_state.component_state.profile_list_state,
    );
}

pub fn render_child_list(frame: &mut Frame, area: Rect, app_state: &mut AppState) {
    let selected = app_state.component_state.profile_list_state.selected();
    if selected.is_none() {
        return;
    }
    let selected = selected.unwrap();
    let Some(items) = app_state.startup.profiles.get(selected) else {
        return;
    };

    let items: Vec<Text> = items
        .apps
        .iter()
        .map(|app| {
            text![
                Line::from(app.name.clone()).bold(),
                Line::from_iter([
                    span!("─".repeat(2)),
                    span!(" "),
                    span!(Modifier::BOLD; app.path.clone()),
                    span!(" "),
                    span!(app.args.join(" "))
                ])
                .cyan(),
            ]
        })
        .collect();

    let list = List::new(items)
        .style(Color::White)
        .yellow()
        .block(Block::bordered().merge_borders(ratatui::symbols::merge::MergeStrategy::Exact));

    frame.render_widget(list, area);
}
