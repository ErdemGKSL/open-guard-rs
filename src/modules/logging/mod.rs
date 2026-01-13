pub mod events;

use crate::modules::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "logging",
            name_key: "module-logging-name",
            desc_key: "module-logging-desc",
        },
        commands: vec![],
        event_handlers: vec![events::handler],
    }
}
