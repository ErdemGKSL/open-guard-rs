use crate::modules::{Module, ModuleDefinition};

pub mod tracking;

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "sticky_roles",
            name_key: "module-sticky-roles-name",
            desc_key: "module-sticky-roles-desc",
        },
        commands: vec![],
        event_handlers: vec![],
    }
}
