mod components;
mod engine;
mod loader;
mod state;
mod ui;
mod utils;
use crate::{
    loader::{load_config_app, load_with_home_override},
    state::{AppMode, AppSignal, AppState, ComponentState, SharedBuffer},
    ui::render,
};
use crossterm::event::{self, KeyCode, KeyEvent};
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
    let list_state = ListState::default().with_selected(Some(0));
    let mut app_state = AppState {
        startup: startup,
        mode: AppMode::Normal,
        component_state: ComponentState {
            profile_list_state: list_state,
        },
        current_event: None,
        shared_buffer: SharedBuffer::default(),
    };

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &mut app_state))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                let signal = match app_state.mode {
                    AppMode::BufferWrite => handle_buffer_write(key, &mut app_state),
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

fn handle_buffer_write(key: KeyEvent, app_state: &mut AppState) -> Option<AppSignal> {
    match key.code {
        KeyCode::Char(any) => app_state.shared_buffer.insert_under_cursor(any),
        KeyCode::Esc => {
            app_state.mode = AppMode::Normal;
            app_state.current_event = None;
        }
        KeyCode::Backspace => {
            app_state.shared_buffer.delete_under_cursor();
        }
        KeyCode::Left => {
            app_state.shared_buffer.left();
        }
        KeyCode::Right => {
            app_state.shared_buffer.right();
        }
        KeyCode::Enter => {
            if let Some(event) = app_state.current_event.clone() {
                app_state.current_event =
                    Some(event.transition_to(state::EventState::WaitingCompletation));
            };
        }
        _ => {}
    };
    None
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
            let current_profile = app_state.component_state.profile_list_state.selected()?;

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
        KeyCode::Char('n') => {
            app_state.current_event = Some(state::AppEvent::CreateNewProfile(
                state::EventState::Started,
            ))
        }
        KeyCode::Char('d') => app_state.current_event = Some(state::AppEvent::DeleteCurrentProfile),
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
