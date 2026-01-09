pub mod hello;

use crate::{Data, Error};

#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    pub id: &'static str,
    pub name_key: &'static str,
    pub description_key: &'static str,
}

pub struct Module {
    pub definition: ModuleDefinition,
    pub commands: Vec<poise::Command<Data, Error>>,
}

pub fn get_modules() -> Vec<Module> {
    vec![hello::module()]
}

pub fn commands() -> Vec<poise::Command<Data, Error>> {
    let mut all_commands = vec![];

    for mut module in get_modules() {
        let category = module.definition.id;
        for command in &mut module.commands {
            command.category = Some(category.into());
        }
        all_commands.extend(module.commands);
    }

    all_commands.push(crate::services::help::help());
    all_commands
}

pub fn definitions() -> Vec<ModuleDefinition> {
    get_modules().into_iter().map(|m| m.definition).collect()
}
