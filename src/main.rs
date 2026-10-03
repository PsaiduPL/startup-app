mod engine;
mod loader;
mod state;
mod ui;
mod utils;
use std::collections::HashMap;
use std::fmt::Error;

use crate::{
    loader::load_with_home_override,
    state::{AppMode, AppSignal, AppState, ComponentState},
    ui::render,
};
use crossterm::event::{self, KeyCode, KeyEvent};
use jiff::tz::Dst::No;
use ratatui::widgets::ListState;

use crate::state::Startup;

fn load_startup() -> Startup {
    #[cfg(debug_assertions)]
    {
        load_with_home_override("/root/project/rust/startup-app/test-resources").unwrap()
    }

    #[cfg(not(debug_assertions))]
    {
        load_config_app().unwrap()
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let startup = load_startup();
    let mut list_state = ListState::default().with_selected(Some(0));
    let mut app_state = AppState {
        startup: startup,
        mode: AppMode::Normal,
        component_state: ComponentState {
            profile_list_state: list_state,
        },
    };

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &mut app_state))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                let signal = match app_state.mode {
                    AppMode::Normal => handle_normal_mode(key, &mut app_state),
                    AppMode::Command => handle_command_event(key, &mut app_state),
                };
                match signal {
                    Some(AppSignal::Quit) => break Ok(()),
                    _ => {}
                }
            }
        }
    })
}

fn handle_normal_mode(key_event: KeyEvent, app_state: &mut AppState) -> Option<AppSignal> {
    match key_event.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app_state.component_state.profile_list_state.select_next();
            {}
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app_state
                .component_state
                .profile_list_state
                .select_previous();
        }
        KeyCode::Char('q') | KeyCode::Esc => return Some(AppSignal::Quit),
        KeyCode::Char(':') => app_state.mode = AppMode::Command,
        KeyCode::Enter => {
            let Some(current_profile) = app_state.component_state.profile_list_state.selected()
            else {
                return None;
            };
            let profile = app_state.startup.profiles[current_profile].clone();

            std::thread::spawn(move || {
                engine::fire_profile(&profile);
            });
        }
        _ => {}
    };
    None
}
fn handle_command_event(key_event: KeyEvent, app_state: &mut AppState) -> Option<AppSignal> {
    match key_event.code {
        KeyCode::Char('p') => {
            app_state.startup.profiles[0].profile = String::from("XD");
        }
        _ => {}
    };

    app_state.mode = AppMode::Normal;
    None
}

fn fire_profile(app_state: &mut AppState) {}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
}
