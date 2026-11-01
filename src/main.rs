mod components;
mod engine;
mod loader;
mod state;
mod ui;
mod utils;
use std::{
    collections::VecDeque,
    sync::mpsc::{self, Sender},
};

use crate::{
    loader::{load_config_app, load_with_home_override},
    state::{
        AppEvent::{self, Down, Left, Right, Up},
        AppMode, AppSignal, AppState, ComponentState,
        EventState::Started,
        MultiStageBuffer, SharedBuffer, UIEvent,
    },
    ui::render,
};
use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::state::Startup;

fn load_startup() -> Startup {
    #[cfg(debug_assertions)]
    {
        load_with_home_override("/Users/psaidu/coding/rust/startup-app/test-resources").unwrap()
    }

    #[cfg(not(debug_assertions))]
    {
        load_config_app().unwrap()
    }
}
//TODO figure out how to handle popup as last operation, maybe callback event are okay or queue

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let startup = load_startup();
    let list_state = ListState::default().with_selected(Some(0));
    let app_state = ListState::default().with_selected(None);
    let (emitter, sink) = mpsc::channel::<UIEvent>();
    let mut app_state = AppState {
        startup: startup,
        mode: AppMode::Normal,
        component_state: ComponentState {
            profile_list_state: list_state,
            profile_apps_state: app_state,
        },
        current_list_type: state::ListableComponentType::ProfileState,
        current_event: None,
        buffer: MultiStageBuffer::new(1),
        emitter: emitter.clone(),
        callback_queue: VecDeque::with_capacity(20),
    };

    handle_keyboard_events_off_thread(emitter.clone());
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &mut app_state))?;
            if let Ok(event) = sink.recv() {
                match event {
                    UIEvent::Keyboard(key) => {
                        let signal = match app_state.mode {
                            AppMode::BufferWrite => handle_buffer_write(key, &mut app_state),
                            AppMode::Normal => handle_normal_mode(key, &mut app_state),
                            AppMode::Command => handle_command_event(key, &mut app_state),
                        };
                        match signal {
                            Some(AppSignal::Quit) => return Ok(()),
                            _ => {}
                        }
                    }
                    UIEvent::Kill => return Ok(()),
                    _ => {}
                }
            }
        }
    })
}

fn handle_buffer_write(key: KeyEvent, app_state: &mut AppState) -> Option<AppSignal> {
    match key.code {
        KeyCode::Char(any) => app_state.buffer.insert_under_cursor(any),
        KeyCode::Esc => {
            app_state.mode = AppMode::Normal;
            app_state.current_event = None;
        }
        KeyCode::Backspace => {
            app_state.buffer.delete_under_cursor();
        }
        KeyCode::Left => {
            app_state.buffer.left();
        }
        KeyCode::Right => {
            app_state.buffer.right();
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
            app_state.set_app_event(Down);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app_state.set_app_event(Up);
        }
        KeyCode::Left => {
            app_state.set_app_event(Left);
        }
        KeyCode::Right => {
            app_state.set_app_event(Right);
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
            app_state.mode(AppMode::BufferWrite);
            app_state.set_app_event(AppEvent::KeyboardShortcutEvent(state::ShortcutEvent::New(
                Started,
            )));
            return None;
        }
        KeyCode::Char('d') => {
            app_state.set_app_event(AppEvent::KeyboardShortcutEvent(
                state::ShortcutEvent::Delete,
            ));
        }
        KeyCode::Char('r') => {
            app_state.mode(AppMode::BufferWrite);
            app_state.set_app_event(AppEvent::KeyboardShortcutEvent(
                state::ShortcutEvent::Rename(Started),
            ));
            return None;
        }
        _ => {}
    };

    app_state.mode = AppMode::Normal;
    None
}

fn handle_keyboard_events_off_thread(emitter: Sender<UIEvent>) {
    std::thread::spawn(move || {
        let Err(_) = std::panic::catch_unwind(|| {
            emitter.send(UIEvent::Startup).unwrap();
            loop {
                if let event = event::read().unwrap() {
                    match event {
                        event::Event::Key(key) => {
                            if key.is_press() {
                                emitter.send(UIEvent::Keyboard(key));
                            }
                        }
                        event::Event::Resize(_, _) => {
                            emitter.send(UIEvent::Refresh);
                        }
                        _ => {}
                    }
                }
            }
        });
        emitter.send(UIEvent::Kill);
    });
}

fn fire_profile(app_state: &mut AppState) {}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
}
