pub mod hello;

use crate::{Data, Error};

pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![hello::hello()]
}
