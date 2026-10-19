mod constants;
mod trait_def;
mod mapping;
mod loader;

// Re-export all constants (e.g. ENTITY_TYPE_USER)
pub use constants::*;
// Re-export the trait
pub use trait_def::ToCedarEntity;
// Re-export the loader functions
pub use loader::{
    get_user_entities,
    get_role_entities,
    get_group_entities,
    get_dept_entities,
    get_policy_entities,
    find_descendants_entities,
    get_role_models_by_user_uuid
};