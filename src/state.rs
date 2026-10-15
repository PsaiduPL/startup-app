use crossterm::event::Event;
use ratatui::{buffer, widgets::ListState};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AppState {
    pub startup: Startup,
    pub mode: AppMode,
    pub component_state: ComponentState,
    pub current_event: Option<AppEvent>,
    pub shared_buffer: SharedBuffer,
}

impl AppState {
    pub fn clear_event(&mut self) {
        self.current_event = None;
    }

    pub fn mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    pub fn clear_buffer(&mut self) {
        self.shared_buffer.clear();
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
    CreateNewProfile(EventState),
    RenameProfile(EventState),
    DeleteCurrentProfile,
    Default,
}
impl AppEvent {
    pub fn transition_to(self, new_state: EventState) -> AppEvent {
        match self {
            AppEvent::CreateNewProfile(_) => AppEvent::CreateNewProfile(new_state),
            AppEvent::RenameProfile(_) => AppEvent::RenameProfile(new_state),
            _ => AppEvent::Default,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventState {
    Started,
    WaitingCompletation,
    Completed,
}

#[derive(Debug)]
pub struct ComponentState {
    pub profile_list_state: ListState,
    // pub profile_apps_state: ListState,
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
