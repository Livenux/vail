use axum::{
    extract::State,
    routing::{get, post, put, delete},
    Router,
};

use crate::{api::AppState, error::AppResult, model::*};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/hosts", get(list_hosts).post(create_host))
        .route("/hosts/:id", get(get_host).put(update_host).delete(delete_host))
        .route("/host-groups", get(list_host_groups).post(create_host_group))
        .route("/host-groups/:id", get(get_host_group).put(update_host_group).delete(delete_host_group))
}

async fn list_hosts(
    State(state): State<AppState>,
) -> AppResult<impl axum::response::IntoResponse> {
    let hosts = sqlx::query_as::<_, (i64, String, String, i32, Option<String>, Option<String>, Option<String>, Option<String>, i16, String)>(
        "SELECT id, name, hostname, port, username, credential_type, description, tags::text, status, create_time::text FROM host WHERE deleted = 0 ORDER BY id DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let list: Vec<HostResponse> = hosts.into_iter().map(|h| HostResponse {
        id: h.0,
        name: h.1,
        hostname: h.2,
        port: h.3,
        username: h.4,
        credential_type: h.5,
        description: h.6,
        tags: h.7.and_then(|t| serde_json::from_str(&t).ok()),
        status: h.8,
        create_time: h.9,
    }).collect();

    Ok(axum::json::Json(ApiResponse::success(list)))
}

async fn create_host(
    State(state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<CreateHostRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    let result = sqlx::query(
        "INSERT INTO host (name, hostname, port, username, credential_type, credential_data, description, tags, status, create_time, update_time) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1, NOW(), NOW()) RETURNING id"
    )
    .bind(&payload.name)
    .bind(&payload.hostname)
    .bind(payload.port.unwrap_or(22))
    .bind(&payload.username)
    .bind(&payload.credential_type)
    .bind(&payload.credential_data)
    .bind(&payload.description)
    .bind(payload.tags.map(|t| t.to_string()))
    .fetch_one(&state.db)
    .await?;

    Ok(axum::json::Json(ApiResponse::success(serde_json::json!({
        "id": result.0::<i64>()
    }))))
}

async fn get_host(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> AppResult<impl axum::response::IntoResponse> {
    let host = sqlx::query_as::<_, (i64, String, String, i32, Option<String>, Option<String>, Option<String>, Option<String>, i16, String)>(
        "SELECT id, name, hostname, port, username, credential_type, description, tags::text, status, create_time::text FROM host WHERE id = $1 AND deleted = 0"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound("Host not found".to_string()))?;

    Ok(axum::json::Json(ApiResponse::success(HostResponse {
        id: host.0,
        name: host.1,
        hostname: host.2,
        port: host.3,
        username: host.4,
        credential_type: host.5,
        description: host.6,
        tags: host.7.and_then(|t| serde_json::from_str(&t).ok()),
        status: host.8,
        create_time: host.9,
    })))
}

async fn update_host(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    axum::extract::Json(payload): axum::extract::Json<UpdateHostRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    let mut updates = vec![];
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres>> + Send + Sync> = vec![];
    
    if let Some(name) = payload.name {
        updates.push("name = $1");
        params.push(Box::new(name));
    }
    if let Some(hostname) = payload.hostname {
        updates.push(format!("hostname = ${}", params.len() + 1));
        params.push(Box::new(hostname));
    }
    if let Some(port) = payload.port {
        updates.push(format!("port = ${}", params.len() + 1));
        params.push(Box::new(port));
    }
    if let Some(username) = payload.username {
        updates.push(format!("username = ${}", params.len() + 1));
        params.push(Box::new(username));
    }
    if let Some(credential_type) = payload.credential_type {
        updates.push(format!("credential_type = ${}", params.len() + 1));
        params.push(Box::new(credential_type));
    }
    if let Some(credential_data) = payload.credential_data {
        updates.push(format!("credential_data = ${}", params.len() + 1));
        params.push(Box::new(credential_data));
    }
    if let Some(description) = payload.description {
        updates.push(format!("description = ${}", params.len() + 1));
        params.push(Box::new(description));
    }
    if let Some(tags) = payload.tags {
        updates.push(format!("tags = ${}", params.len() + 1));
        params.push(Box::new(tags.to_string()));
    }
    if let Some(status) = payload.status {
        updates.push(format!("status = ${}", params.len() + 1));
        params.push(Box::new(status));
    }

    if updates.is_empty() {
        return Err(crate::error::AppError::BadRequest("No fields to update".to_string()));
    }

    updates.push("update_time = NOW()".to_string());
    
    let query = format!(
        "UPDATE host SET {} WHERE id = ${} AND deleted = 0",
        updates.join(", "),
        params.len() + 1
    );
    
    let mut query_builder = sqlx::query(&query);
    for p in params {
        query_builder = match p.downcast::<String>() {
            Ok(v) => query_builder.bind(v),
            Err(_) => continue,
        };
    }
    query_builder = query_builder.bind(id);
    query_builder.execute(&state.db).await?;

    Ok(axum::json::Json(ApiResponse::success("Updated")))
}

async fn delete_host(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> AppResult<impl axum::response::IntoResponse> {
    sqlx::query("UPDATE host SET deleted = 1, update_time = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(axum::json::Json(ApiResponse::success("Deleted")))
}

async fn list_host_groups(
    State(state): State<AppState>,
) -> AppResult<impl axum::response::IntoResponse> {
    let groups = sqlx::query_as::<_, (i64, String, Option<i64>, Option<String>, Option<i32>, String)>(
        "SELECT id, name, parent_id, description, sort, create_time::text FROM host_group WHERE deleted = 0 ORDER BY sort, id"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(axum::json::Json(ApiResponse::success(groups)))
}

async fn create_host_group(
    State(state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> AppResult<impl axum::response::IntoResponse> {
    let result = sqlx::query(
        "INSERT INTO host_group (name, parent_id, description, sort, create_time) VALUES ($1, $2, $3, $4, NOW()) RETURNING id"
    )
    .bind(payload.get("name").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(payload.get("parent_id").and_then(|v| v.as_i64()))
    .bind(payload.get("description").and_then(|v| v.as_str()))
    .bind(payload.get("sort").and_then(|v| v.as_i64()).unwrap_or(0))
    .fetch_one(&state.db)
    .await?;

    Ok(axum::json::Json(ApiResponse::success(serde_json::json!({
        "id": result.0::<i64>()
    }))))
}

async fn get_host_group(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> AppResult<impl axum::response::IntoResponse> {
    let group = sqlx::query_as::<_, (i64, String, Option<i64>, Option<String>, Option<i32>, String)>(
        "SELECT id, name, parent_id, description, sort, create_time::text FROM host_group WHERE id = $1 AND deleted = 0"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound("Host group not found".to_string()))?;

    Ok(axum::json::Json(ApiResponse::success(group)))
}

async fn update_host_group(
    State state: State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> AppResult<impl axum::response::IntoResponse> {
    sqlx::query("UPDATE host_group SET name = COALESCE($1, name), parent_id = COALESCE($2, parent_id), description = COALESCE($3, description), sort = COALESCE($4, sort) WHERE id = $5")
        .bind(payload.get("name").and_then(|v| v.as_str()))
        .bind(payload.get("parent_id").and_then(|v| v.as_i64()))
        .bind(payload.get("description").and_then(|v| v.as_str()))
        .bind(payload.get("sort").and_then(|v| v.as_i64()))
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(axum::json::Json(ApiResponse::success("Updated")))
}

async fn delete_host_group(
    State state: State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> AppResult<impl axum::response::IntoResponse> {
    sqlx::query("UPDATE host_group SET deleted = 1 WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(axum::json::Json(ApiResponse::success("Deleted")))
}
