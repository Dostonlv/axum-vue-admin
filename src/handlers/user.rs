use axum::extract::Query;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use validator::Validate;

use crate::schemas::auth::Claims;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::paginated::PaginatedApiResponse;
use crate::schemas::response::ApiResponse;
use crate::schemas::user::{
    AssignRoleDto, CreateUserDto, QueryParams, UpdateUserDto, UserResponse, UserRoleResponse,
};
use crate::{config::openapi::USER_TAG, errors::app_error::AppError, services::user::service::UserService};

#[utoipa::path(get, path = "",
    params(QueryParams),
    responses((status = 200, body = Vec<UserResponse>),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn list_users(
    State(service): State<UserService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Query(params): Query<QueryParams>,
) -> Result<impl IntoResponse, AppError> {
    params.validate()?;
    let (users, total) = service
        .list_users(current_user, context, params.clone())
        .await?;
    Ok(PaginatedApiResponse::success(
        users,
        total,
        params.page,
        params.page_size,
        StatusCode::OK,
    ))
}

#[utoipa::path(
    get,
    path = "/{user_uuid}",  // {id} in the path is treated as a parameter
    params(
        ("user_uuid" = String, Path, description = "User UUID")
    ),
    responses(
        (status = 200, description = "User details", body = UserResponse),
        (status = 404, description = "User not found"),
    ),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn get_user(
    Path(user_uuid): Path<String>,
    State(service): State<UserService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let user = service.get_user(current_user, context, user_uuid).await?;
    Ok(ApiResponse::success(user, StatusCode::OK))
}

#[utoipa::path(
    post,
    path = "",
    request_body=CreateUserDto,
    responses(( status=201, body=UserResponse, description = "User created"),
                (status=409, description = "Username or email already exists"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn create_user(
    State(service): State<UserService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<CreateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let user = service.create_user(current_user, context, dto).await?;
    Ok(ApiResponse::success(user, StatusCode::CREATED))
}

#[utoipa::path(
    put,
    path = "/{user_uuid}",
    request_body=UpdateUserDto,
    params(
        ("user_uuid" = String, Path, description = "User UUID")
    ),
    responses(  (status=200, body=UserResponse, description="Updated"),
                (status=404, description="User not found"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_user(
    Path(user_uuid): Path<String>,
    State(service): State<UserService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let user = service
        .update_user(current_user, context, user_uuid, dto)
        .await?;
    Ok(ApiResponse::success(user, StatusCode::OK))
}

#[utoipa::path(
    delete,
    path = "/{user_uuid}",
    params(
        ("user_uuid" = String, Path, description = "User UUID")
    ),
    responses(( status=200, description="Deleted"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn delete_user(
    Path(user_uuid): Path<String>,
    State(service): State<UserService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service
        .delete_user(current_user, context, user_uuid)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/{user_uuid}/roles",
    params(
        ("user_uuid" = String, Path, description = "User UUID")
    ),
    responses(( status=200, description="User roles", body=Vec<UserRoleResponse>),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn user_roles(
    Path(user_uuid): Path<String>,
    State(service): State<UserService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let roles = service.user_roles(current_user, context, user_uuid).await?;
    Ok(ApiResponse::success(roles, StatusCode::OK))
}

#[utoipa::path(
    post,
    path = "/{user_uuid}/roles",
    request_body=AssignRoleDto,
    params(
        ("user_uuid" = String, Path, description = "User UUID")
    ),
    responses(( status=201, description="Role assigned"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn assign_roles(
    Path(user_uuid): Path<String>,
    State(service): State<UserService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<AssignRoleDto>, // the Json extractor must come last, otherwise it fails to compile
) -> Result<impl IntoResponse, AppError> {
    service
        .assign_roles(current_user, context, user_uuid, dto)
        .await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/{user_uuid}/roles/{role_uuid}",
    params(
        ("user_uuid" = String, Path, description = "User UUID"),
        ("role_uuid" = String, Path, description = "Role UUID")
    ),
    responses(( status=204, description="Role removed from user"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn revoke_roles(
    Path((user_uuid, role_uuid)): Path<(String, String)>,
    State(service): State<UserService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service
        .revoke_roles(current_user, context, user_uuid, role_uuid)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
