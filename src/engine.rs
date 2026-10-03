use std::{
    io::{self, Stderr},
    ops::Deref,
    process::{Command, Stdio},
};

use crate::state::{StartupApp, StartupProfile};

pub enum CommandExecStatus {
    Success,
    Failure(io::Error),
}

pub fn fire_profile(profile: &StartupProfile) -> Vec<(StartupApp, CommandExecStatus)> {
    profile
        .apps
        .iter()
        .map(|e| (e.clone(), fire_app(e)))
        .collect()
}

fn fire_app(
    StartupApp {
        name: _,
        path: path,
        args: args,
    }: &StartupApp,
) -> CommandExecStatus {
    let spawn_handle = Command::new(path)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| CommandExecStatus::Failure(e));

    match spawn_handle {
        Ok(_) => CommandExecStatus::Success,
        Err(err) => err,
    }
}
