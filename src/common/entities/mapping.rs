use std::collections::HashMap;
use cedar_policy::RestrictedExpression;
use crate::entity::{cedar_policy_set, departments, roles, user_groups};
use super::trait_def::ToCedarEntity;
use super::constants::*;

// Role mapping
impl ToCedarEntity for roles::Model {
    fn entity_type() -> &'static str { ENTITY_TYPE_ROLE }
    fn entity_id(&self) -> String { self.role_uuid.to_string() }
    fn attributes(&self) -> HashMap<String, RestrictedExpression> {
        let mut attrs = HashMap::new();
        attrs.insert(ENTITY_ATTR_NAME.to_string(), RestrictedExpression::new_string(self.role_name.clone()));
        attrs
    }
}

// Group mapping
impl ToCedarEntity for user_groups::Model {
    fn entity_type() -> &'static str { ENTITY_TYPE_GROUP }
    fn entity_id(&self) -> String { self.user_group_uuid.clone() }
    fn attributes(&self) -> HashMap<String, RestrictedExpression> {
        let mut attrs = HashMap::new();
        attrs.insert(ENTITY_ATTR_NAME.to_string(), RestrictedExpression::new_string(self.name.clone()));
        attrs
    }
}

// Department mapping
impl ToCedarEntity for departments::Model {
    fn entity_type() -> &'static str { ENTITY_TYPE_DEPARTMENT }
    fn entity_id(&self) -> String { self.dept_uuid.clone() }
    fn attributes(&self) -> HashMap<String, RestrictedExpression> {
        let mut attrs = HashMap::new();
        attrs.insert(ENTITY_ATTR_NAME.to_string(), RestrictedExpression::new_string(self.name.clone()));
        attrs
    }
}

// Policy mapping
impl ToCedarEntity for cedar_policy_set::Model {
    fn entity_type() -> &'static str { ENTITY_TYPE_POLICY }
    fn entity_id(&self) -> String { self.policy_uuid.to_string() }
    fn attributes(&self) -> HashMap<String, RestrictedExpression> {
        let mut attrs = HashMap::new();
        attrs.insert(ENTITY_ATTR_NAME.to_string(), RestrictedExpression::new_string(self.annotation.clone()));
        attrs
    }
}