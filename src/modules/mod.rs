pub mod hello;

use crate::{Data, Error};

pub fn commands() -> Vec<poise::Command<Data, Error>> {
    let mut commands = vec![];
    commands.extend(hello::commands::commands());
    commands
}
