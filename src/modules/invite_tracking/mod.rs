pub mod events;
pub mod tracking;
pub mod stats;
pub mod commands;

use crate::modules::{Module, ModuleDefinition};

pub const DEFINITION: ModuleDefinition = ModuleDefinition {
    id: "invite_tracking",
    name_key: "module-invite-tracking-name",
    desc_key: "module-invite-tracking-desc",
};

pub fn module() -> Module {
    Module {
        definition: DEFINITION,
        commands: commands::commands(),
        event_handlers: vec![events::handler],
    }
}
