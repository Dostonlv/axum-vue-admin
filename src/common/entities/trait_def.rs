use std::collections::{HashSet, HashMap};
use std::str::FromStr;
use cedar_policy::{Entity, EntityId, EntityTypeName, EntityUid, RestrictedExpression};
use crate::errors::app_error::AppError;

pub trait ToCedarEntity {
    /// Cedar type name (e.g. "User")
    fn entity_type() -> &'static str;

    /// Unique entity ID (usually a UUID)
    fn entity_id(&self) -> String;

    /// Entity attributes
    fn attributes(&self) -> HashMap<String, RestrictedExpression>;

    /// Parent entity UIDs (empty by default)
    fn parents(&self) -> HashSet<EntityUid> {
        HashSet::new()
    }

    /// Core conversion: build a cedar_policy::Entity
    fn to_cedar_entity(&self) -> Result<Entity, AppError> {
        let uid = self.get_uid()?;
        Ok(Entity::new(uid, self.attributes(), self.parents())?)
    }

    /// Shortcut for the entity's EntityUid
    fn get_uid(&self) -> Result<EntityUid, AppError> {
        let type_name = EntityTypeName::from_str(Self::entity_type())?;
        let id = EntityId::from_str(&self.entity_id())?;
        Ok(EntityUid::from_type_name_and_id(type_name, id))
    }
}