use ratatui::{
    Frame,
    layout::{Constraint, Flex::Start, Layout, Rect, Spacing},
    macros::{line, span, text},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Clear, List, Padding, Paragraph},
};

use crate::{
    state::{AppEvent, AppState, EventState, StartupProfile},
    utils::formatted_time,
};

pub fn render(frame: &mut Frame, app_state: &mut AppState) {
    let main_layout = Layout::default()
        .spacing(Spacing::Overlap(1))
        .constraints([Constraint::Length(4), Constraint::Fill(5)]);

    let [title_area, app_area] = main_layout.areas(frame.area());
    render_title_area(frame, app_state, title_area);
    render_app_area(frame, app_state, app_area);

    if let Some(event) = &app_state.current_event {
        handle_event(frame, app_state, event.clone());
    }
}

// title layout

fn render_title_area(frame: &mut Frame, app_state: &mut AppState, title_area: Rect) {
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
        Paragraph::new(text![formatted_time(), app_mode_to_string(&app_state)])
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
fn app_mode_to_string(app_state: &AppState) -> String {
    String::from(match app_state.mode {
        crate::state::AppMode::BufferWrite => "Buffer",
        crate::state::AppMode::Command => "Command",
        crate::state::AppMode::Normal => "Normal",
    })
}

// main layout
fn render_app_area(frame: &mut Frame, app_state: &mut AppState, app_area: Rect) {
    let constraints = [Constraint::Percentage(20), Constraint::Percentage(80)];

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

fn handle_event(frame: &mut Frame, app_state: &mut AppState, app_event: AppEvent) {
    match app_event {
        AppEvent::CreateNewProfile(state) => {
            match state {
                EventState::WaitingCompletation => {
                    app_state.startup.profiles.push(StartupProfile {
                        profile: app_state.shared_buffer.to_string(),
                        apps: vec![],
                    });
                    app_state.clear_event();
                    app_state.clear_buffer();
                    app_state.mode(crate::state::AppMode::Normal);
                    return;
                }
                _ => {}
            };
            app_state.mode = crate::state::AppMode::BufferWrite;
            let popup_block = Block::bordered().title("Popup");
            let centered_area = frame
                .area()
                .centered(Constraint::Percentage(60), Constraint::Percentage(20));
            // clears out any background in the area before rendering the popup
            frame.render_widget(Clear, centered_area);
            let text: String = app_state.shared_buffer.to_string();

            let pivot = app_state.shared_buffer.position();
            let (left, cursor, right) = split_text_by_cursor(&text, pivot);
            let text = line![left, cursor.bold(), right];
            let paragraph = Paragraph::new(text).block(popup_block);
            frame.render_widget(paragraph, centered_area);
        }
        AppEvent::RenameProfile(state) => {}
        AppEvent::DeleteCurrentProfile => {
            let Some(selected) = app_state.component_state.profile_list_state.selected() else {
                return;
            };
            app_state.startup.profiles.remove(selected);
            app_state.clear_event();
            app_state.mode(crate::state::AppMode::Normal);
        }
        _ => {}
    };
}

fn split_text_by_cursor(str: &str, cursor: usize) -> (&str, &str, &str) {
    let cursor = match cursor {
        0 => 1,
        _ => cursor,
    };
    let Some((left, right)) = str.split_at_checked(cursor - 1) else {
        return (str, "", "");
    };
    match right.len() {
        1 => (left, right, ""),
        _ => {
            let Some((cursor, new_right)) = right.split_at_checked(1) else {
                return (left, right, "");
            };
            (left, cursor, new_right)
        }
    }
}
