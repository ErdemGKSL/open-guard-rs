use crate::modules::{Module, ModuleDefinition};

pub mod events;

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "sticky_roles",
            name_key: "module-sticky-roles-name",
            desc_key: "module-sticky-roles-desc",
        },
        commands: vec![],
    }
}
