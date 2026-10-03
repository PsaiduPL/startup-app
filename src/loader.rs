use std::{error::Error, fs::OpenOptions, io::BufReader, path::Path};

use ratatui::layout::Flex::Start;
use serde_json::Error as SerdeError;

use crate::Startup;

#[derive(Debug)]
pub enum StartupLoadingError {
    NoHomeVariableDefined,
    ParsingJsonFile(SerdeError),
    ConfigFileIsNotAJson,
}

impl std::fmt::Display for StartupLoadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for StartupLoadingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StartupLoadingError::ParsingJsonFile(err) => Some(err),
            _ => None,
        }
    }
}

const STARTUP_FILE: &str = ".startup.json";

pub fn load_config_app() -> Result<Startup, StartupLoadingError> {
    let home_variable = std::env::home_dir().ok_or(StartupLoadingError::NoHomeVariableDefined)?;

    let config_path = home_variable.join(STARTUP_FILE);
    handle_config_path_error(validate_config_path(&config_path))?;

    parse_from_file(&config_path).map_err(StartupLoadingError::ParsingJsonFile)
}

pub fn load_with_home_override(home_path: &str) -> Result<Startup, StartupLoadingError> {
    unsafe {
        std::env::set_var("HOME", String::from(home_path));
    }
    load_config_app()
}

enum ConfigPathValidationError {
    NoConfigFileFound,
    ConfigFileIsNotAJson,
}

fn validate_config_path(config_path: &Path) -> Result<(), ConfigPathValidationError> {
    if config_path.try_exists().is_ok_and(|exists| !exists) {
        return Err(ConfigPathValidationError::NoConfigFileFound);
    }

    if config_path.extension().filter(|e| *e == "json").is_none() {
        return Err(ConfigPathValidationError::ConfigFileIsNotAJson);
    }
    Ok(())
}

fn handle_config_path_error(
    error: Result<(), ConfigPathValidationError>,
) -> Result<(), StartupLoadingError> {
    let Err(error) = error else { return Ok(()) };
    match error {
        ConfigPathValidationError::NoConfigFileFound => {
            let path = std::env::home_dir().unwrap().join(STARTUP_FILE);
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap();
            let default_app = Startup::default();
            serde_json::to_writer_pretty(file, &default_app).unwrap();
        }
        ConfigPathValidationError::ConfigFileIsNotAJson => {
            return Err(StartupLoadingError::ConfigFileIsNotAJson);
        }
    };
    Ok(())
}

fn parse_from_file(file_path: &Path) -> Result<Startup, SerdeError> {
    let file = OpenOptions::new()
        .read(true)
        .open(file_path)
        .expect("Failed to open file");
    let startup: Result<Startup, SerdeError> = serde_json::from_reader(BufReader::new(file));
    startup
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    const HOME_OVERRIDE: &str = "/root/project/rust/startup-app/test-resources";

    #[test]
    fn should_load_config() {
        let res = load_with_home_override(HOME_OVERRIDE);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.default_profile, "work");
    }
}
