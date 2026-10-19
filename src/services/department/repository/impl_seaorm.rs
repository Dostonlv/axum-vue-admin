use crate::common::cedar_utils::{AuthAction, ResourceType};
use crate::common::entities::{get_dept_entities};
use crate::config::state::AppState;
use crate::entity::{departments, user_group_members, user_groups, users};
use crate::schemas::auth::{Claims};
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::department::{CreateDepartmentDto, DepartmentResponse, DeptTreeNode};
use crate::schemas::user::{DeptResponse, GroupResponse, UserResponse};
use crate::{bad_request, conflict, errors::app_error::AppError, not_found};
use async_recursion::async_recursion;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, JoinType, QueryFilter,
    QueryOrder, QuerySelect, TransactionTrait, entity::prelude::*,
};
use std::collections::{HashMap, VecDeque};
use tracing::warn;

const MAX_DEPT_DEPTH: usize = 100;
const ROOT_DEPARTMENT_ID: i32 = 0;
const ROOT_DEPARTMENT_UUID: &str = "0";

#[derive(Clone)]
pub struct DepartmentService {
    app_state: AppState,
}

impl DepartmentService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_dept_id_from_uuid(
        &self,
        db: &impl ConnectionTrait,
        dept_uuid: &str,
    ) -> Result<i32, AppError> {
        departments::Entity::find()
            .select_only()
            .column(departments::Column::DeptId)
            .filter(departments::Column::DeptUuid.eq(dept_uuid))
            .into_tuple::<i32>()
            .one(db)
            .await?
            .ok_or(not_found!(format!(
                "Department with UUID '{}' not found",
                dept_uuid
            )))
    }

    pub async fn list_departments(
        &self,
        current_user: Claims,
        context: CedarContext,
    ) -> Result<Vec<DeptTreeNode>, AppError> {
        self.app_state
            .auth_service
            .check_permission(
                &current_user.sub,
                context,
                AuthAction::ListDepartment,
                ResourceType::Department(None),
            )
            .await?;

        let all_departments = departments::Entity::find()
            .filter(departments::Column::IsDeleted.eq(false))
            .all(self.app_state.db.as_ref())
            .await?;

        let root_parent_id = if current_user.is_super_admin {
            ROOT_DEPARTMENT_ID
        } else {
            // let user_dept_id = users::Entity::find()
            //     .select_only()
            //     .column(users::Column::DeptId)
            //     .filter(users::Column::UserUuid.eq(&current_user.sub))
            //     .into_tuple::<i32>()
            //     .one(self.app_state.db.as_ref())
            //     .await?
            //     .ok_or_else(|| not_found!("User's current department not found"))?;

            // Find the department's id in the full list
            departments::Entity::find()
                .select_only()
                .column(departments::Column::DeptId)
                .filter(departments::Column::DeptUuid.eq(&current_user.dept_uuid))
                .into_tuple::<i32>()
                .one(self.app_state.db.as_ref())
                .await?
                .ok_or(bad_request!("User's parent department not found"))?
        };

        let tree_node =
            build_dept_tree_optimized_with_uuid(&all_departments, root_parent_id).await?;
        Ok(tree_node)
    }

    pub async fn create_department(
        &self,
        current_user: Claims,
        context: CedarContext,
        dto: CreateDepartmentDto,
    ) -> Result<DepartmentResponse, AppError> {
        self.app_state
            .auth_service
            .check_permission(
                &current_user.sub,
                context,
                AuthAction::CreateDepartment,
                ResourceType::Department(None),
            )
            .await?;

        let parent_id = if dto.parent_uuid == ROOT_DEPARTMENT_UUID {
            ROOT_DEPARTMENT_ID
        } else {
            self.get_dept_id_from_uuid(self.app_state.db.as_ref(), &dto.parent_uuid)
                .await?
        };

        if departments::Entity::find()
            .filter(
                Condition::all()
                    .add(departments::Column::Name.eq(&dto.name))
                    .add(departments::Column::ParentId.eq(parent_id))
                    .add(departments::Column::IsDeleted.eq(false)),
            )
            .one(self.app_state.db.as_ref())
            .await?
            .is_some()
        {
            return Err(conflict!(
                "A department with the same name already exists under this parent."
            ));
        };

        let new_department = departments::ActiveModel {
            dept_uuid: Set(Uuid::new_v4().to_string()),
            name: Set(dto.name),
            desc: Set(Some(dto.desc)),
            order: Set(dto.order),
            parent_id: Set(parent_id),
            ..Default::default()
        };

        let saved_department = new_department.insert(self.app_state.db.as_ref()).await?;

        Ok(DepartmentResponse {
            uuid: saved_department.dept_uuid,
            name: saved_department.name,
            desc: saved_department.desc,
            order: saved_department.order,
            parent_uuid: dto.parent_uuid,
        })
    }

    pub async fn update_department(
        &self,
        current_user: Claims,
        context: CedarContext,
        dept_uuid: String,
        dto: CreateDepartmentDto,
    ) -> Result<DepartmentResponse, AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_dept_entities(
            self.app_state.db.as_ref(),
            self.app_state.cache_service.as_ref(),
            &dept_uuid,
            &schema
        ).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.sub,
                context.clone(),
                AuthAction::UpdateDepartment,
                ResourceType::Department(Some(dept_uuid.clone())),
                es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;

        let new_parent_id = if dto.parent_uuid == ROOT_DEPARTMENT_UUID {
            ROOT_DEPARTMENT_ID
        } else {
            self.get_dept_id_from_uuid(&txn, &dto.parent_uuid).await?
        };

        let mut department: departments::ActiveModel = departments::Entity::find()
            .filter(departments::Column::DeptUuid.eq(&dept_uuid))
            .one(&txn)
            .await?
            .ok_or(not_found!("department not found".to_string()))?
            .into();

        let original_parent_id = *department.parent_id.as_ref();
        if original_parent_id != new_parent_id {
            self.app_state
                .auth_service
                .check_permission(
                    &current_user.sub,
                    context,
                    AuthAction::MoveDepartment,
                    ResourceType::Department(Some(dept_uuid)),
                )
                .await?;
        }

        department.name = Set(dto.name);
        department.desc = Set(Some(dto.desc));
        department.order = Set(dto.order);
        department.parent_id = Set(new_parent_id);

        let updated_department = department.update(&txn).await?;
        txn.commit().await?;
        Ok(DepartmentResponse {
            uuid: updated_department.dept_uuid,
            name: updated_department.name,
            desc: updated_department.desc,
            order: updated_department.order,
            parent_uuid: dto.parent_uuid,
        })
    }

    pub async fn delete_department(
        &self,
        current_user: Claims,
        context: CedarContext,
        dept_uuid: String,
    ) -> Result<(), AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_dept_entities(
            self.app_state.db.as_ref(),
            self.app_state.cache_service.as_ref(),
            &dept_uuid,
            &schema
        ).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.sub,
                context,
                AuthAction::DeleteDepartment,
                ResourceType::Department(Some(dept_uuid.clone())),
                es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;

        let dept_model = departments::Entity::find()
            .filter(departments::Column::DeptUuid.eq(&dept_uuid))
            .one(&txn)
            .await?
            .ok_or_else(|| not_found!("Department to delete not found"))?;

        if dept_model.parent_id == ROOT_DEPARTMENT_ID {
            return Err(bad_request!("Deletion of root department is not allowed."));
        };

        // Check whether the department has children
        if self
            .has_children_optimized(&txn, dept_model.dept_id)
            .await?
        {
            return Err(bad_request!(
                "Department has children, deletion not allowed."
            ));
        }

        // Check whether the department still has users
        if self.has_user_optimized(&txn, dept_model.dept_id).await? {
            return Err(bad_request!("Department has users, deletion not allowed."));
        }
        // If other resources get bound to departments, the check needs extending

        // Delete the department
        let mut active_model: departments::ActiveModel = dept_model.into();
        active_model.is_deleted = Set(true);
        active_model.update(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    async fn has_children_optimized(
        &self,
        txn: &DatabaseTransaction,
        department_id: i32,
    ) -> Result<bool, AppError> {
        let child_count = departments::Entity::find()
            .filter(
                departments::Column::ParentId
                    .eq(department_id)
                    .and(departments::Column::IsDeleted.eq(false)),
            )
            .count(txn)
            .await?;
        Ok(child_count > 0)
    }

    async fn has_user_optimized(
        &self,
        txn: &DatabaseTransaction,
        department_id: i32,
    ) -> Result<bool, AppError> {
        let user_count = users::Entity::find()
            .filter(users::Column::DeptId.eq(department_id))
            .count(txn)
            .await?;
        Ok(user_count > 0)
    }

    pub async fn department_users(
        &self,
        current_user: Claims,
        context: CedarContext,
        dept_uuid: String,
    ) -> Result<Vec<UserResponse>, AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_dept_entities(
            self.app_state.db.as_ref(),
            self.app_state.cache_service.as_ref(),
            &dept_uuid,
            &schema
        ).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.sub,
                context,
                AuthAction::ViewDepartmentUsers,
                ResourceType::Department(Some(dept_uuid.clone())),
                es,
            )
            .await?;

        let users_with_dept = users::Entity::find()
            .find_also_related(departments::Entity)
            .join(JoinType::InnerJoin, users::Relation::Departments.def())
            .filter(departments::Column::DeptUuid.eq(dept_uuid))
            .all(self.app_state.db.as_ref())
            .await?;
        let users = assemble_user_info(self.app_state.db.as_ref(), users_with_dept).await?;
        Ok(users)
    }
}

// IDs of all child departments
pub async fn get_all_child_dept_ids(
    db: &DatabaseConnection,
    parent_dept_uuid: &str,
) -> Result<Vec<i32>, AppError> {
    let parent_dept_id = departments::Entity::find()
        .select_only()
        .column(departments::Column::DeptId)
        .filter(departments::Column::DeptUuid.eq(parent_dept_uuid))
        .into_tuple::<i32>()
        .one(db)
        .await?
        .ok_or(not_found!("department not found".to_string()))?;

    let all_depts = departments::Entity::find()
        .filter(departments::Column::IsDeleted.eq(false))
        .all(db)
        .await?;

    let depts_by_parent: HashMap<i32, Vec<departments::Model>> =
        all_depts.into_iter().fold(HashMap::new(), |mut acc, dept| {
            acc.entry(dept.parent_id).or_default().push(dept);
            acc
        });

    let mut all_child_ids = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(parent_dept_id);

    // include the starting parent ID
    all_child_ids.push(parent_dept_id);

    while let Some(current_id) = queue.pop_front() {
        if let Some(children) = depts_by_parent.get(&current_id) {
            for child in children {
                if !all_child_ids.contains(&child.dept_id) {
                    all_child_ids.push(child.dept_id);
                    queue.push_back(child.dept_id);
                }
            }
        }
    }

    Ok(all_child_ids)
}


// UUIDs of all ancestors of a department
pub async fn find_parents_dept_id(
    db: &DatabaseConnection,
    current_dept_uuid: &str,
) -> Result<Vec<String>, AppError> {
    let all_depts = departments::Entity::find()
        .select_only()
        .columns([
            departments::Column::DeptId,
            departments::Column::ParentId,
            departments::Column::DeptUuid,
        ])
        .filter(departments::Column::IsDeleted.eq(false))
        .into_tuple::<(i32, i32, String)>()
        .all(db)
        .await?;

    let start_id = all_depts
        .iter()
        .find(|(_, _, uuid)| uuid == current_dept_uuid)
        .map(|(id, _, _)| *id) // extract the id
        .ok_or_else(|| {
            not_found!(format!(
                "Starting department with UUID '{}' not found.",
                current_dept_uuid
            ))
        })?;

    // `id -> parent_id` is used for the walk.
    let parent_map: HashMap<i32, i32> = all_depts
        .iter()
        .map(|(id, parent_id, _)| (*id, *parent_id))
        .collect();

    // `id -> uuid` is used to build the response at the end.
    let id_to_uuid_map: HashMap<i32, &String> =
        all_depts.iter().map(|(id, _, uuid)| (*id, uuid)).collect();

    let mut parent_uuids = Vec::new();
    let mut current_id = Some(start_id);

    while let Some(id) = current_id {
        if id == ROOT_DEPARTMENT_ID {
            break;
        }

        if let Some(&parent_id) = parent_map.get(&id) {
            if parent_id != ROOT_DEPARTMENT_ID {
                // Look up the parent's UUID
                if let Some(parent_uuid) = id_to_uuid_map.get(&parent_id) {
                    parent_uuids.push(parent_uuid.to_string());
                } else {
                    // Data integrity issue: parent_id found, but id_to_uuid_map has no entry for it.
                    // The parent may have been soft-deleted, or the row is orphaned.
                    warn!(
                        "Data integrity issue: parent department with ID {} (parent_id: {}) not found in the active list.",
                        id, parent_id
                    );
                    break;
                }
            }
            current_id = Some(parent_id);
        } else {
            // The chain is broken, the current ID is not in the map.
            break;
        }
    }

    Ok(parent_uuids)
}

#[async_recursion]
pub async fn build_dept_tree(
    depts_by_parent: &HashMap<i32, Vec<&departments::Model>>,
    id_to_uuid_map: &HashMap<i32, &String>,
    parent_id: i32,
) -> Result<Vec<DeptTreeNode>, AppError> {
    let mut tree_nodes = Vec::new();

    if let Some(children_models) = depts_by_parent.get(&parent_id) {
        for dept in children_models {
            // Build the subtree recursively
            let children = build_dept_tree(depts_by_parent, id_to_uuid_map, dept.dept_id).await?;

            let parent_uuid = id_to_uuid_map
                .get(&dept.parent_id)
                .map(|s| s.to_string())
                // If parent_id is 0 (root), use the virtual root UUID
                .unwrap_or_else(|| ROOT_DEPARTMENT_UUID.to_string());

            tree_nodes.push(DeptTreeNode {
                uuid: dept.dept_uuid.clone(),
                name: dept.name.clone(),
                desc: dept.desc.clone(),
                order: dept.order,
                parent_uuid,
                children,
            });
        }
    }

    tree_nodes.sort_by_key(|n| n.order);
    Ok(tree_nodes)
}

pub async fn build_dept_tree_optimized_with_uuid(
    all_departments: &[departments::Model],
    target_id: i32,
) -> Result<Vec<DeptTreeNode>, AppError> {
    if all_departments.is_empty() {
        return Ok(vec![]);
    }

    let depts_by_parent: HashMap<i32, Vec<&departments::Model>> =
        all_departments
            .iter()
            .fold(HashMap::new(), |mut acc, dept| {
                acc.entry(dept.parent_id).or_default().push(dept);
                acc
            });

    let id_to_uuid_map: HashMap<i32, &String> = all_departments
        .iter()
        .map(|dept| (dept.dept_id, &dept.dept_uuid))
        .collect();

    return if target_id == 0 {
        build_dept_tree(&depts_by_parent, &id_to_uuid_map, 0).await
    } else {
        let target_dept_opt = all_departments.iter().find(|d| d.dept_id == target_id);

        if let Some(dept) = target_dept_opt {
            // Build this department's children recursively
            let children = build_dept_tree(&depts_by_parent, &id_to_uuid_map, dept.dept_id).await?;

            // Get this department's parent_uuid
            let parent_uuid = id_to_uuid_map
                .get(&dept.parent_id)
                .map(|s| s.to_string())
                .unwrap_or_else(|| ROOT_DEPARTMENT_UUID.to_string());

            // Build the single node
            let node = DeptTreeNode {
                uuid: dept.dept_uuid.clone(),
                name: dept.name.clone(),
                desc: dept.desc.clone(),
                order: dept.order,
                parent_uuid,
                children,
            };

            // Return a Vec with just this root
            Ok(vec![node])
        } else {
            // Unknown ID: return an empty list
            Ok(vec![])
        }
    };
}

// IDs of all child departments
pub async fn children_dept(db: &DatabaseConnection, parent_id: i32) -> Result<Vec<i32>, DbErr> {
    // Load all departments ordered by `order`
    let all_depts = departments::Entity::find()
        .columns([departments::Column::DeptId, departments::Column::ParentId])
        .order_by_asc(departments::Column::Order)
        .all(db)
        .await?;

    // Result set (includes the department itself)
    let mut dept_ids = vec![parent_id];
    // VecDeque as the queue (better suited to FIFO than Vec)
    let mut to_visit = VecDeque::from([parent_id]);

    while let Some(current_id) = to_visit.pop_front() {
        // Find the direct children
        let children: Vec<i32> = all_depts
            .iter()
            .filter(|dept| dept.parent_id == current_id)
            .map(|dept| dept.dept_id)
            .collect();

        // Add to the result set and the queue
        dept_ids.extend(&children);
        to_visit.extend(children);
    }

    Ok(dept_ids)
}

// Assemble user info (user, roles, department)
pub async fn assemble_user_info(
    db: &DatabaseConnection,
    users_with_dept: Vec<(users::Model, Option<departments::Model>)>,
) -> Result<Vec<UserResponse>, AppError> {
    // 2. Collect all user IDs
    let user_ids: Vec<i32> = users_with_dept
        .iter()
        .map(|(user, _)| user.user_id)
        .collect();

    // 3. Query the groups of all users in one go
    let user_group_relations = user_group_members::Entity::find()
        .find_also_related(user_groups::Entity)
        .filter(user_group_members::Column::UserId.is_in(user_ids))
        .all(db)
        .await?;

    // 4. Group them by user ID
    let mut user_groups_map: HashMap<i32, Vec<GroupResponse>> = HashMap::new();
    for (relation, group_opt) in user_group_relations {
        if let Some(group) = group_opt {
            user_groups_map
                .entry(relation.user_id)
                .or_insert_with(Vec::new)
                .push(GroupResponse {
                    user_group_uuid: group.user_group_uuid,
                    name: group.name,
                });
        }
    }

    // 5. Build the final result
    let result = users_with_dept
        .into_iter()
        .map(|(user, dept)| UserResponse {
            uuid: user.user_uuid,
            username: user.username.clone(),
            alias: user.alias.clone(),
            email: user.email.clone(),
            phone: user.phone.clone(),
            is_active: user.is_active,
            dept: dept.map(|d| DeptResponse {
                uuid: d.dept_uuid,
                name: d.name,
            }),
            groups: user_groups_map
                .get(&user.user_id)
                .cloned()
                .unwrap_or_default(),
            avatar: user.avatar.clone(),
            last_login: user.last_login,
        })
        .collect();
    Ok(result)
}
