use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

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
pub enum AppSignal {
    Quit,
}

#[derive(Debug)]
pub struct ComponentState {
    pub profile_list_state: ListState,
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
