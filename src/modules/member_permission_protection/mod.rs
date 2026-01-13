pub mod events;

use crate::modules::{Module, ModuleDefinition};

pub const DEFINITION: ModuleDefinition = ModuleDefinition {
    id: "member_permission_protection",
    name_key: "module-member-permission-protection-name",
    desc_key: "module-member-permission-protection-desc",
};

pub fn module() -> Module {
    Module {
        definition: DEFINITION,
        commands: vec![],
    }
}
