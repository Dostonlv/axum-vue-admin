use crate::config::openapi::DEPARTMENT_TAG;
use crate::schemas::auth::Claims;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::department::{CreateDepartmentDto, DepartmentResponse, DeptTreeNode};
use crate::schemas::{response::ApiResponse, user::UserResponse};
use crate::{errors::app_error::AppError, services::department::service::DepartmentService};
use axum::response::IntoResponse;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    Extension,
};

#[utoipa::path(get, path = "",
    responses((status = 200, body = Vec<DeptTreeNode>),),
    tag = DEPARTMENT_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn list_departments(
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    State(service): State<DepartmentService>,
) -> Result<ApiResponse<Vec<DeptTreeNode>>, AppError> {
    let departments = service.list_departments(current_user, context).await?;
    Ok(ApiResponse::success(departments, StatusCode::OK))
}

#[utoipa::path(
    post,
    path = "",
    request_body=CreateDepartmentDto,
    responses(( status=201, body=DepartmentResponse, description = "Department created"),
                (status=409, description = "Department already exists"),),
    tag = DEPARTMENT_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn create_department(
    State(service): State<DepartmentService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(params): Json<CreateDepartmentDto>,
) -> Result<ApiResponse<DepartmentResponse>, AppError> {
    let department = service
        .create_department(current_user, context, params)
        .await?;
    Ok(ApiResponse::success(department, StatusCode::CREATED))
}

#[utoipa::path(
    put,
    path = "/{dept_uuid}",
    request_body=CreateDepartmentDto,
    params(
        ("dept_uuid" = String, Path, description = "Department UUID")
    ),
    responses(  (status=200, body=DepartmentResponse, description="Updated"),
                (status=404, description="Department not found"),),
    tag = DEPARTMENT_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_department(
    Path(dept_uuid): Path<String>,
    State(service): State<DepartmentService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<CreateDepartmentDto>,
) -> Result<ApiResponse<DepartmentResponse>, AppError> {
    let department = service
        .update_department(current_user, context, dept_uuid, dto)
        .await?;
    Ok(ApiResponse::success(department, StatusCode::OK))
}

#[utoipa::path(
    delete,
    path = "/{dept_uuid}",
    params(
        ("dept_uuid" = String, Path, description = "Department UUID")
    ),
    responses(
    ( status=204, description="Deleted"),
    ( status=404, description="Department not found"),
    ( status=403, description="Cannot delete"),
    ),
    tag = DEPARTMENT_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn delete_department(
    Path(dept_uuid): Path<String>,
    State(service): State<DepartmentService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service
        .delete_department(current_user, context, dept_uuid)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// GET	/api/departments/{id}/users	list all users of a department	dept:read
#[utoipa::path(
    get,
    path = "/{dept_uuid}/users",
    params(
        ("dept_uuid" = String, Path, description = "Department UUID")
    ),
    responses(
    ( status=200, description="OK")
    ),
    tag = DEPARTMENT_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn departments_users(
    Path(dept_uuid): Path<String>,
    State(service): State<DepartmentService>,
    Extension(current_user): Extension<Claims>,
    Extension(context): Extension<CedarContext>,
) -> Result<ApiResponse<Vec<UserResponse>>, AppError> {
    let users = service
        .department_users(current_user, context, dept_uuid)
        .await?;
    Ok(ApiResponse::success(users, StatusCode::OK))
}
