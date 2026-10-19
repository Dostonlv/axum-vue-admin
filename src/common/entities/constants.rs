// Entity type names
pub const ENTITY_TYPE_USER: &str = "User";
pub const ENTITY_TYPE_ROLE: &str = "Role";
pub const ENTITY_TYPE_GROUP: &str = "Group";
pub const ENTITY_TYPE_DEPARTMENT: &str = "Department";
pub const ENTITY_TYPE_POLICY: &str = "Policy";

// Attribute names
// Constants used by Cedar
const  APPLICATION_ENTITY_UID: &str = r#"Application::"VueAxumAdmin""#;

pub const  ENTITY_TYPE_ROBOT: &str = "Robot";
pub const  ENTITY_TYPE_ROBOT_ACCOUNT: &str = "RobotAccount";

pub const  ENTITY_ATTR_NAME: &str = "name";
pub const ENTITY_ATTR_OWNERS: &str = "owners";

// Cache key prefixes
pub const USER_ENTITIES_CACHE_PREFIX: &str = "authz:user_entities";
pub const ROLE_ENTITIES_CACHE_PREFIX: &str = "authz:role_entities";

pub const DEPT_ENTITIES_CACHE_PREFIX: &str = "authz:dept_entities";

pub const GROUP_ENTITIES_CACHE_PREFIX: &str = "authz:group_entities";

pub const POLICY_ENTITIES_CACHE_PREFIX: &str = "authz:policy_entities";
pub const POLICIES_AND_TEMPLATES_CACHE_KEY: &str = "authz:policies_and_templates";
pub const TEMPLATE_LINKS_CACHE_KEY: &str = "authz:template_links";
