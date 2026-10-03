use std::{io, ops::Deref, process::Command};

use crate::{
    Startup, StartupApp, StartupProfile,
    enginge::CommandExecStatus::{Failure, Success},
};

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

fn fire_app(StartupApp { name, path, args }: &StartupApp) -> CommandExecStatus {
    let spawn_handle = Command::new(path)
        .args(args)
        .spawn()
        .map_err(|e| Failure(e));

    match spawn_handle {
        Ok(_) => Success,
        Err(err) => err,
    }
}
