pub mod commands;

use crate::modules::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "config",
            name_key: "module-config-name",
            description_key: "module-config-desc",
        },
        commands: vec![commands::config()],
    }
}
