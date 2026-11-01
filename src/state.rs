use std::{collections::HashMap, sync::mpsc::Sender};

use crossterm::event::{Event, KeyEvent};
use jiff::tz::Dst::No;
use ratatui::{buffer, widgets::ListState};
use serde::{Deserialize, Serialize};

use crate::state::ShortcutEvent::{New, Rename};

#[derive(Debug)]
pub struct AppState {
    pub startup: Startup,
    pub mode: AppMode,
    pub component_state: ComponentState,
    pub current_list_type: ListableComponentType,
    pub current_event: Option<AppEvent>,
    pub buffer: MultiStageBuffer,
    pub emitter: Sender<UIEvent>,
    pub callback_queue: std::collections::VecDeque<HandleEventCallbackCommand>,
}

impl AppState {
    pub fn clear_event(&mut self) {
        self.current_event = None;
    }

    pub fn mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    pub fn send_ui_event(&self, event: UIEvent) {
        self.emitter.send(event);
    }

    pub fn set_app_event(&mut self, event: AppEvent) {
        self.current_event = Some(event);
    }

    pub fn select_idx_from_state(&self, component_type: &ListableComponentType) -> Option<usize> {
        match component_type {
            ListableComponentType::ProfileState => {
                self.component_state.profile_list_state.selected()
            }
            ListableComponentType::ProfileAppsState => {
                self.component_state.profile_apps_state.selected()
            }
        }
    }

    pub fn select_idx_from_current_component(&self) -> Option<usize> {
        self.select_idx_from_state(&self.current_list_type)
    }

    pub fn push_callback_event(&mut self, event: HandleEventCallbackCommand) {
        self.callback_queue.push_back(event);
    }

    pub fn pop_callback_event(&mut self) -> Option<HandleEventCallbackCommand> {
        self.callback_queue.pop_front()
    }
}

#[derive(Debug)]
pub enum AppMode {
    Normal,
    Command,
    BufferWrite,
}
#[derive(Debug)]
pub enum AppSignal {
    Quit,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    KeyboardShortcutEvent(ShortcutEvent),
    CreateNewProfile(EventState),
    RenameProfile(EventState),
    Default,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum ShortcutEvent {
    Rename(EventState),
    New(EventState),
    Delete,
}

impl AppEvent {
    pub fn transition_to(self, new_state: EventState) -> AppEvent {
        match self {
            AppEvent::CreateNewProfile(_) => AppEvent::CreateNewProfile(new_state),
            AppEvent::RenameProfile(_) => AppEvent::RenameProfile(new_state),
            AppEvent::KeyboardShortcutEvent(event) => match event {
                ShortcutEvent::Rename(_) => AppEvent::KeyboardShortcutEvent(Rename(new_state)),
                ShortcutEvent::New(_) => AppEvent::KeyboardShortcutEvent(New(new_state)),
                _ => AppEvent::Default,
            },
            _ => AppEvent::Default,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventState {
    Started,
    WaitingCompletation,
    Completed,
    MultiStage(usize), // stage and input from specified stage
}

pub enum ApplicationAddStage {
    Name = 3,
    Path = 2,
    Args = 1,
    End = 0,
}

impl ApplicationAddStage {
    pub fn from_stage(stage: usize) -> ApplicationAddStage {
        match stage {
            3 => Self::Name,
            2 => Self::Path,
            1 => Self::Args,
            _ => Self::End,
        }
    }
    pub fn to_stage(&self) -> usize {
        match self {
            ApplicationAddStage::Args => 1,
            ApplicationAddStage::Path => 2,
            ApplicationAddStage::Name => 3,
            _ => 0,
        }
    }
    pub fn next(self) -> ApplicationAddStage {
        ApplicationAddStage::from_stage(self.to_stage() - 1)
    }
}

#[derive(Debug, Clone)]
pub enum ListableComponentType {
    ProfileState,
    ProfileAppsState,
}

#[derive(Debug)]
pub struct ComponentState {
    pub profile_list_state: ListState,
    pub profile_apps_state: ListState,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Startup {
    #[serde(rename = "defaultProfile")]
    pub default_profile: String,
    pub profiles: Vec<StartupProfile>,
}

impl Default for Startup {
    fn default() -> Self {
        Self {
            default_profile: String::from("example"),
            profiles: vec![StartupProfile {
                profile: String::from("example"),
                apps: vec![],
            }],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StartupProfile {
    pub profile: String,
    pub apps: Vec<StartupApp>,
}

// copy implicit
// clone explicit
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StartupApp {
    pub name: String,
    pub path: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct MultiStageBuffer {
    buffers: Vec<SharedBuffer>,
    current_stage: usize,
}

impl std::ops::Deref for MultiStageBuffer {
    type Target = SharedBuffer;

    fn deref(&self) -> &Self::Target {
        &self.buffers.get(self.current_stage).expect("Invalid stage")
    }
}

impl std::ops::DerefMut for MultiStageBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffers
            .get_mut(self.current_stage)
            .expect("Invalid stage")
    }
}

impl MultiStageBuffer {
    pub fn new(stage_count: usize) -> MultiStageBuffer {
        let mut buffers = vec![];
        for _ in 0..stage_count {
            buffers.push(SharedBuffer::default());
        }
        Self {
            buffers: buffers,
            current_stage: 0,
        }
    }

    pub fn buffer_from_stage(&self, stage: usize) -> Option<&SharedBuffer> {
        self.buffers.get(stage)
    }

    pub fn next(&mut self) -> Option<usize> {
        let next = self.current_stage + 1;
        if next >= self.buffers.len() {
            return None;
        }
        self.current_stage = next;
        Some(next)
    }
}

#[derive(Debug)]
pub struct SharedBuffer {
    buffer: String,
    cursor_position: usize,
}

impl Default for SharedBuffer {
    fn default() -> Self {
        Self {
            buffer: String::with_capacity(512),
            cursor_position: 0,
        }
    }
}

impl SharedBuffer {
    pub fn push_last(&mut self, ch: impl Into<char>) {
        self.buffer.push(ch.into());
        self.right();
    }

    pub fn insert_under_cursor(&mut self, ch: impl Into<char>) {
        self.buffer.insert(self.cursor_position, ch.into());
        self.right();
    }
    pub fn delete_under_cursor(&mut self) {
        if (self.cursor_position == 0) {
            return;
        }
        let next_cursor = clamp_min(self.cursor_position - 1, 0);
        self.buffer.remove(next_cursor);
        self.left();
    }

    pub fn left(&mut self) -> usize {
        self.cursor_position = match self.cursor_position {
            0 => 0,
            _ => clamp_min(self.cursor_position - 1, 0),
        };
        self.cursor_position
    }
    pub fn right(&mut self) -> usize {
        self.cursor_position = clamp_max(self.cursor_position + 1, self.buffer.len());
        self.cursor_position
    }

    pub fn to_string(&self) -> String {
        self.buffer.clone()
    }

    pub fn position(&self) -> usize {
        self.cursor_position
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor_position = 0;
    }
}

fn clamp_min<T>(val: T, min: T) -> T
where
    T: PartialOrd,
{
    if val.lt(&min) {
        return min;
    }
    return val;
}

fn clamp_max<T>(val: T, max: T) -> T
where
    T: PartialOrd,
{
    if val.gt(&max) {
        return max;
    }
    return val;
}
#[derive(Debug)]
pub enum UIEvent {
    Keyboard(KeyEvent),
    Refresh,
    Kill,
    Startup,
}
#[derive(Debug)]
pub enum HandleEventCallbackCommand {
    DisplayPopup(String), // title
}
