use axum::extract::Query;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use validator::Validate;

use crate::schemas::auth::{Claims};
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::{
    paginated::PaginatedApiResponse,
    response::ApiResponse,
    role::{CreateRoleDto, QueryParams, RoleResponse, UpdateRoleDto},
};
use crate::{config::openapi::ROLE_TAG, 
            errors::app_error::AppError, 
            services::role::service::RoleService};

#[utoipa::path(get, path = "",
    params(QueryParams),
    responses((status = 200, body = Vec<RoleResponse>),),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn list_roles(
    State(service): State<RoleService>,
    Query(params): Query<QueryParams>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    params.validate()?;
    let (roles, total) = service
        .list_roles(current_user, context, params.clone())
        .await?;
    Ok(PaginatedApiResponse::success(
        roles,
        total,
        params.page,
        params.page_size,
        StatusCode::OK,
    ))
}

#[utoipa::path(
    get,
    path = "/{role_uuid}",  // {id} in the path is treated as a parameter
    params(
        ("role_id" = String, Path, description = "Role UUID")
    ),
    responses(
        (status = 200, description = "Role details", body = RoleResponse),
        (status = 404, description = "Role not found"),
    ),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn get_role(
    Path(role_uuid): Path<String>,
    State(service): State<RoleService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let role = service.get_role(current_user, context, role_uuid).await?;
    Ok(ApiResponse::success(role, StatusCode::OK))
}

#[utoipa::path(
    post,
    path = "",
    request_body=CreateRoleDto,
    responses(( status=201, body=RoleResponse, description = "Role created"),
                (status=409, description = "Role already exists"),),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn create_role(
    State(service): State<RoleService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<CreateRoleDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let role = service.create_role(current_user, context, dto).await?;
    Ok(ApiResponse::success(role, StatusCode::CREATED))
}

#[utoipa::path(
    put,
    path = "/{role_uuid}",
    request_body=UpdateRoleDto,
    responses(  (status=200, body=RoleResponse, description="Updated"),
                (status=404, description="Role not found"),),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_role(
    Path(role_uuid): Path<String>,
    State(service): State<RoleService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<UpdateRoleDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let role = service
        .update_role(current_user, context, role_uuid, dto)
        .await?;
    Ok(ApiResponse::success(role, StatusCode::OK))
}

#[utoipa::path(
    delete,
    path = "/{role_uuid}",
    params(
        ("role_uuid" = String, Path, description = "Role UUID")
    ),
    responses(( status=204, description="Deleted"),),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn delete_role(
    Path(role_uuid): Path<String>,
    State(service): State<RoleService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service
        .delete_role(current_user, context, role_uuid)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
