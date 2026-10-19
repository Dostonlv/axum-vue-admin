use crate::config::openapi::GROUP_TAG;
use crate::schemas::auth::{Claims};
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::{
    groups::{
        AssignRolesDto, AssignUsersDto, CreateGroupDto, GroupResponse, GroupRoleResponse,
        QueryParams,
    },
    paginated::PaginatedApiResponse,
    response::ApiResponse,
};
use crate::{errors::app_error::AppError, services::groups::GroupService};
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use validator::Validate;

#[utoipa::path(
    get,
    path = "",
    params(QueryParams),
    responses((status = 200, body = Vec<GroupResponse>),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn list_groups(
    State(service): State<GroupService>,
    Query(params): Query<QueryParams>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    params.validate()?;
    let (groups, total) = service
        .list_groups(current_user, context, params.clone())
        .await?;
    Ok(PaginatedApiResponse::success(
        groups,
        total,
        params.page,
        params.page_size,
        StatusCode::OK,
    ))
}

#[utoipa::path(
    post,
    path = "",
    request_body=CreateGroupDto,
    responses(( status=200, body=GroupResponse,  description = "Group created"),
                (status=409, description = "Group already exists"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn create_group(
    State(service): State<GroupService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(payload): Json<CreateGroupDto>,
) -> Result<ApiResponse<GroupResponse>, AppError> {
    payload.validate()?;
    let group = service.create_group(current_user, context, payload).await?;
    Ok(ApiResponse::success(group, StatusCode::CREATED))
}

#[utoipa::path(
    get,
    path = "/{group_uuid}",
    params(
        ("group_uuid" = String, Path, description = "Group UUID")
    ),
    responses(( status=200, body=GroupResponse, description = "OK"),
    ( status=404, description = "Not found"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn get_group(
    Path(group_uuid): Path<String>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let group = service.get_group(current_user, context, group_uuid).await?;
    Ok(ApiResponse::success(group, StatusCode::OK))
}

#[utoipa::path(
    put,
    path = "/{group_uuid}",
    request_body=CreateGroupDto,
    params(
        ("group_uuid" = String, Path, description = "Group UUID")
    ),
    responses(( status=200, body=GroupResponse, description = "Updated"),
                (status=404, description="Group not found"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_group(
    Path(group_uuid): Path<String>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<CreateGroupDto>,
) -> Result<ApiResponse<GroupResponse>, AppError> {
    let group = service
        .update_group(current_user, context, group_uuid, dto)
        .await?;
    Ok(ApiResponse::success(group, StatusCode::OK))
}

#[utoipa::path(
    delete,
    path = "/{group_uuid}",
    params(
        ("group_uuid" = String, Path, description = "Group UUID")
    ),
    responses(
    ( status=200, description="Deleted"),
    ( status=404, description="Group not found"),
    ( status=403, description="Cannot delete"),
    ),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn delete_group(
    Path(group_uuid): Path<String>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service
        .delete_group(current_user, context, group_uuid)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/{group_uuid}/users",
    request_body=AssignUsersDto,
    params(
        ("group_uuid" = String, Path, description = "Group UUID")
    ),
    responses(( status=201, description="User added to group"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn assign_users(
    Path(group_uuid): Path<String>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<AssignUsersDto>,
) -> Result<impl IntoResponse, AppError> {
    service
        .assign_users(current_user, context, group_uuid, dto)
        .await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/{group_uuid}/users/{user_uuid}",
    request_body=AssignUsersDto,
    params(
        ("group_uuid" = String, Path, description = "Group UUID"),
        ("user_uuid" = String, Path, description = "User UUID",)
    ),
    responses(( status=200, description="Deleted"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn revoke_users(
    Path((group_uuid, user_uuid)): Path<(String, String)>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service
        .revoke_user(current_user, context, group_uuid, user_uuid)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/{group_uuid}/roles",
    request_body=AssignRolesDto,
    params(
        ("group_uuid" = String, Path, description = "Group ID")
    ),
    responses(( status=201, description="Role assigned"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn assign_roles(
    Path(group_uuid): Path<String>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<AssignRolesDto>,
) -> Result<impl IntoResponse, AppError> {
    service
        .assign_roles(current_user, context, group_uuid, dto)
        .await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/{group_uuid}/roles/{role_uuid}",
    params(
        ("group_uuid" = String, Path, description = "Group ID",),
        ("role_uuid" = String, Path, description = "Role ID",)
    ),
    responses(( status=204, description="Role removed"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn revoke_roles(
    Path((group_uuid, role_uuid)): Path<(String, String)>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service
        .revoke_roles(current_user, context, group_uuid, role_uuid)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/{group_uuid}/roles",
    params(
        ("group_uuid" = String, Path, description = "Group UUID")
    ),
    responses(( status=200, body=GroupRoleResponse, description = "OK"),
    ( status=404, description = "Not found"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn get_group_roles(
    Path(group_uuid): Path<String>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let group_roles = service
        .get_group_roles(current_user, context, group_uuid)
        .await?;
    Ok(ApiResponse::success(group_roles, StatusCode::OK))
}
