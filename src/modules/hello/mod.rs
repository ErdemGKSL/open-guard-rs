pub mod commands;
pub mod events;

use super::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "hello",
            name_key: "module-hello-name",
            description_key: "module-hello-description",
        },
        commands: commands::commands(),
    }
}
