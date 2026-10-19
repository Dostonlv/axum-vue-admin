use std::collections::{HashSet, HashMap, VecDeque};
use std::str::FromStr;
use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EntityUid, RestrictedExpression, Schema};
use futures_util::TryFutureExt;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QuerySelect, RelationTrait, JoinType, Condition, QueryTrait};
use crate::entity::{cedar_policy_set, departments, group_roles, roles, user_group_members, user_groups, user_roles, users};
use crate::errors::app_error::AppError;
use crate::not_found;
use crate::services::cache::CacheService;

use super::constants::*;
use super::trait_def::ToCedarEntity;


/// Convert a set of ToCedarEntity models into a Cedar Entities object
fn models_to_entities<T: ToCedarEntity>(
    models: Vec<T>,
    schema: Option<&Schema>
) -> Result<Entities, AppError> {
    let mut entities = HashSet::new();
    for model in models {
        entities.insert(model.to_cedar_entity()?);
    }
    Ok(Entities::from_entities(entities, schema)?)
}

/// Wrap a HashSet<Entity> into an Entities object
fn wrap_entities(
    set: HashSet<Entity>,
    schema: Option<&Schema>
) -> Result<Entities, AppError> {
    Ok(Entities::from_entities(set, schema)?)
}

// --- User Entities ---
pub async fn get_user_entities(
    db: &DatabaseConnection,
    cache: &CacheService,
    user_uuid: String,
    schema: &Schema,
) -> Result<Entities, AppError> {
    let cache_key = format!("{}:{}", USER_ENTITIES_CACHE_PREFIX, user_uuid);
    if let Some(entities) = cache.get_entities(cache_key.as_str()).await? {
        return Ok(entities);
    }

    // Fetch the base user
    let user_info = users::Entity::find()
        .filter(users::Column::UserUuid.eq(user_uuid.clone()))
        .one(db)
        .await?
        .ok_or(not_found!("User {} not found", user_uuid))?;

    // Fetch roles, groups and departments concurrently
    let (roles, groups, department) = tokio::try_join!(
        get_role_models_by_user_uuid(db, user_uuid.clone()),

        async {
            user_groups::Entity::find()
            .join(JoinType::InnerJoin, user_groups::Relation::UserGroupMembers.def())
            .filter(user_group_members::Column::UserId.eq(user_info.user_id))
            .all(db)
            .await
            .map_err(AppError::from)
        },

        async {
            departments::Entity::find()
            .join(JoinType::InnerJoin, departments::Relation::Users.def())
            .filter(users::Column::UserId.eq(user_info.user_id))
            .one(db)
            .await
            .map_err(AppError::from)
        }
    )?;

    let mut entities_set = HashSet::new();
    let mut user_parents = HashSet::new();

    // Departments (and their children)
    if let Some(dept) = department {
        let descendant_entities = find_descendants_entities(db, dept.dept_id).await?;
        for e in descendant_entities.iter() {
            user_parents.insert(e.uid());
            entities_set.insert(e.clone());
        }
    }

    // Groups
    for group in groups {
        user_parents.insert(group.get_uid()?);
        entities_set.insert(group.to_cedar_entity()?);
    }

    // Roles
    for role in roles {
        user_parents.insert(role.get_uid()?);
        entities_set.insert(role.to_cedar_entity()?);
    }

    // Build the User entity
    let user_uid = EntityUid::from_type_name_and_id(
        EntityTypeName::from_str(ENTITY_TYPE_USER)?,
        EntityId::from_str(&user_uuid)?,
    );
    let mut user_attrs = HashMap::new();
    user_attrs.insert(ENTITY_ATTR_NAME.to_string(), RestrictedExpression::new_string(user_info.username));

    entities_set.insert(Entity::new(user_uid, user_attrs, user_parents)?);

    let entities = wrap_entities(entities_set, Some(schema))?;
    cache.cache_entities(cache_key.as_str(), entities.clone()).await?;
    Ok(entities)
}

// --- Role Entities ---
pub async fn get_role_entities(
    db: &DatabaseConnection,
    cache: &CacheService,
    role_uuids: &[String],
    schema: &Schema,
) -> Result<Entities, AppError> {

    if role_uuids.is_empty() {
        return Ok(Entities::empty());
    }

    let mut sorted_uuids = role_uuids.to_vec();
    sorted_uuids.sort();
    let cache_key = format!("{}:{}", ROLE_ENTITIES_CACHE_PREFIX, sorted_uuids.join(","));

    if let Some(entities) = cache.get_entities(&cache_key).await? {
        return Ok(entities);
    }

    let roles = roles::Entity::find()
        .filter(roles::Column::RoleUuid.is_in(role_uuids.to_vec()))
        .all(db)
        .await?;
    let entities = models_to_entities(roles, Some(schema))?;
    cache.cache_entities(cache_key.as_str(), entities.clone()).await?;
    Ok(entities)
}

// --- Dept Entities ---
pub async fn get_dept_entities(
    db: &DatabaseConnection,
    cache: &CacheService,
    dept_uuid: &str,
    schema: &Schema,
) -> Result<Entities, AppError> {

    let cache_key = format!("{}:{}", DEPT_ENTITIES_CACHE_PREFIX, dept_uuid);

    if let Some(entities) = cache.get_entities(&cache_key).await? {
        return Ok(entities);
    }

    let dept = departments::Entity::find()
        .filter(departments::Column::DeptUuid.eq(dept_uuid))
        .one(db)
        .await?
        .ok_or(not_found!("Dept not found"))?;
    let entities = models_to_entities(vec![dept], Some(schema))?;

    cache.cache_entities(cache_key.as_str(), entities.clone()).await?;
    Ok(entities)
}

// --- Find child departments ---
pub async fn find_descendants_entities(
    db: &DatabaseConnection,
    dept_id: i32,
) -> Result<Entities, AppError> {
    let all_depts = departments::Entity::find()
        .filter(departments::Column::IsDeleted.eq(false))
        .all(db)
        .await?;

    let depts_by_parent: HashMap<i32, Vec<&departments::Model>> = all_depts.iter().fold(HashMap::new(), |mut acc, d| {
        acc.entry(d.parent_id).or_default().push(d); acc
    });
    let depts_by_id: HashMap<i32, &departments::Model> = all_depts.iter().map(|d| (d.dept_id, d)).collect();

    let mut entities = HashSet::new();
    let mut queue = VecDeque::from([dept_id]);
    let mut visited = HashSet::new();

    while let Some(current_id) = queue.pop_front() {
        if !visited.insert(current_id) { continue; }
        if let Some(model) = depts_by_id.get(&current_id) {
            entities.insert(model.to_cedar_entity()?);
            if let Some(children) = depts_by_parent.get(&current_id) {
                for child in children { queue.push_back(child.dept_id); }
            }
        }
    }
    wrap_entities(entities, None)
}

// --- Policy Entities ---
pub async fn get_policy_entities(
    db: &DatabaseConnection,
    cache: &CacheService,
    policy_uuid: &str,
    schema: &Schema,
) -> Result<Entities, AppError> {

    let cache_key = format!("{}:{}", POLICY_ENTITIES_CACHE_PREFIX, policy_uuid);
    if let Some(entities) = cache.get_entities(&cache_key).await? {
        return Ok(entities);
    }


    let policy = cedar_policy_set::Entity::find()
        .filter(cedar_policy_set::Column::PolicyUuid.eq(policy_uuid))
        .one(db)
        .await?
        .ok_or(not_found!("Policy not found"))?;
    let entities = models_to_entities(vec![policy], Some(schema))?;
    cache.cache_entities(cache_key.as_str(), entities.clone()).await?;
    Ok(entities)
}

pub async fn get_group_entities(
    db: &DatabaseConnection,
    cache: &CacheService,
    group_uuids: &[String],
    schema: &Schema,
) -> Result<Entities, AppError> {
    if group_uuids.is_empty() {
        return Ok(Entities::empty());
    }

    let mut sorted_uuids = group_uuids.to_vec();
    sorted_uuids.sort();
    let cache_key = format!("{}:{}", GROUP_ENTITIES_CACHE_PREFIX, sorted_uuids.join(","));

    if let Some(entities) = cache.get_entities(&cache_key).await? {
        return Ok(entities);
    }

    let groups = user_groups::Entity::find()
        .filter(user_groups::Column::UserGroupUuid.is_in(sorted_uuids.clone()))
        .all(db)
        .await?;

    let entities = models_to_entities(groups.clone(), Some(schema))?;


    cache.cache_entities(cache_key.as_str(), entities.clone()).await?;
    Ok(entities)
}

// Helpers
pub async fn get_role_models_by_user_uuid(db: &DatabaseConnection, user_uuid: String) -> Result<Vec<roles::Model>, AppError> {
    let user_id_subquery = users::Entity::find()
        .select_only()
        .column(users::Column::UserId)
        .filter(users::Column::UserUuid.eq(user_uuid))
        .into_query();

    // --- Subquery 1: roles assigned to the user directly ---
    let direct_role_ids_query = user_roles::Entity::find()
        .select_only() // select only one column
        .column(user_roles::Column::RoleId) // we only need role_id
        .filter(user_roles::Column::UserId.in_subquery(user_id_subquery.clone()));

    // --- Subquery 2: roles inherited through groups ---
    // First, all group_ids the user belongs to
    let group_ids_query = user_group_members::Entity::find()
        .select_only()
        .column(user_group_members::Column::GroupId)
        .filter(user_group_members::Column::UserId.in_subquery(user_id_subquery));

    // Then the role_ids linked to those groups
    let group_role_ids_query = group_roles::Entity::find()
        .select_only()
        .column(group_roles::Column::RoleId)
        .filter(group_roles::Column::GroupId.in_subquery(group_ids_query.into_query()));

    // --- Main query: all matching roles ---
    // Condition::any() (OR) merges the two subqueries
    let all_roles = roles::Entity::find()
        .distinct()
        .filter(
            Condition::any()
                // role_id is among the directly assigned roles
                .add(roles::Column::RoleId.in_subquery(direct_role_ids_query.into_query()))
                // role_id is among the roles inherited via groups
                .add(roles::Column::RoleId.in_subquery(group_role_ids_query.into_query())),
        )
        .all(db)
        .await?;
    Ok(all_roles)
}