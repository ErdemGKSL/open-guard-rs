pub mod bot_adding_protection;
pub mod channel_permission_protection;
pub mod channel_protection;
pub mod logging;
pub mod member_permission_protection;
pub mod moderation_protection;
pub mod role_permission_protection;
pub mod role_protection;
pub mod sticky_roles;

use poise::serenity_prelude as serenity;

pub type EventHandler = for<'a> fn(
    &'a serenity::Context,
    &'a serenity::FullEvent,
    &'a crate::Data,
) -> poise::BoxFuture<'a, Result<(), crate::Error>>;

#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    pub id: &'static str,
    pub name_key: &'static str,
    pub desc_key: &'static str,
}

pub struct Module {
    pub definition: ModuleDefinition,
    pub commands: Vec<poise::Command<crate::Data, crate::Error>>,
    pub event_handlers: Vec<EventHandler>,
}

pub fn get_modules() -> Vec<Module> {
    vec![
        channel_protection::module(),
        channel_permission_protection::module(),
        role_protection::module(),
        role_permission_protection::module(),
        member_permission_protection::module(),
        crate::modules::bot_adding_protection::module(),
        crate::modules::moderation_protection::module(),
        crate::modules::logging::module(),
        crate::modules::sticky_roles::module(),
    ]
}

pub fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    let mut all_commands = vec![];

    for mut module in get_modules() {
        let category = module.definition.id;
        for command in &mut module.commands {
            command.category = Some(category.into());
        }
        all_commands.extend(module.commands);
    }

    all_commands.push(crate::services::config::config());
    all_commands.push(crate::services::help::help());
    all_commands.push(crate::services::status::status());
    all_commands.push(crate::services::setup::setup());
    all_commands
}

pub fn definitions() -> Vec<ModuleDefinition> {
    get_modules().into_iter().map(|m| m.definition).collect()
}
