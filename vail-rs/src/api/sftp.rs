use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use std::path::PathBuf;

use crate::{api::AppState, error::AppResult, model::*};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sftp/task", post(create_upload_task))
        .route("/sftp/upload", post(upload_chunk))
        .route("/sftp/complete", post(complete_upload))
        .route("/sftp/tasks", get(list_upload_tasks))
        .route("/sftp/tasks/:id", get(get_upload_task))
}

async fn create_upload_task(
    State(state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<CreateUploadTaskRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    let task_no = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO upload_task (task_no, user_id, host_id, remote_path, file_name, file_size, file_md5, chunk_size, uploaded_size, status, create_time, update_time) VALUES ($1, 1, $2, $3, $4, $5, $6, $7, 0, 0, NOW(), NOW()) RETURNING id"
    )
    .bind(&task_no)
    .bind(payload.host_id)
    .bind(&payload.remote_path)
    .bind(&payload.file_name)
    .bind(payload.file_size)
    .bind(&payload.file_md5)
    .bind(payload.chunk_size.unwrap_or(1048576))
    .fetch_one(&state.db)
    .await?;

    let task_dir = PathBuf::from(&state.config.storage.temp_dir).join(&task_no);
    std::fs::create_dir_all(&task_dir).ok();

    Ok(axum::json::Json(ApiResponse::success(UploadTaskResponse {
        id: result.0::<i64>(),
        task_no,
        status: 0,
        uploaded_size: 0,
        file_size: payload.file_size,
    })))
}

async fn upload_chunk(
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> AppResult<impl axum::response::IntoResponse> {
    let mut task_id: Option<i64> = None;
    let mut chunk_index: Option<i32> = None;
    let mut offset: Option<i64> = None;
    let mut content: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| crate::error::AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "task_id" => {
                if let Ok(text) = field.text().await {
                    task_id = text.parse().ok();
                }
            }
            "chunk_index" => {
                if let Ok(text) = field.text().await {
                    chunk_index = text.parse().ok();
                }
            }
            "offset" => {
                if let Ok(text) = field.text().await {
                    offset = text.parse().ok();
                }
            }
            "content" => {
                if let Ok(data) = field.bytes().await {
                    content = Some(data.to_vec());
                }
            }
            _ => {}
        }
    }

    let task_id = task_id.ok_or_else(|| crate::error::AppError::BadRequest("Missing task_id".to_string()))?;
    let chunk_index = chunk_index.ok_or_else(|| crate::error::AppError::BadRequest("Missing chunk_index".to_string()))?;
    let offset = offset.unwrap_or(0);
    let content = content.ok_or_else(|| crate::error::AppError::BadRequest("Missing content".to_string()))?;

    let task = sqlx::query_as::<_, (String, i64)>(
        "SELECT task_no, uploaded_size FROM upload_task WHERE id = $1"
    )
    .bind(task_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound("Task not found".to_string()))?;

    let task_dir = PathBuf::from(&state.config.storage.temp_dir).join(&task.0);
    let chunk_file = task_dir.join(format!("chunk_{}", chunk_index));

    std::fs::write(&chunk_file, &content).map_err(|e| crate::error::AppError::Internal(e.to_string()))?;

    let new_size = task.1 + content.len() as i64;
    sqlx::query("UPDATE upload_task SET uploaded_size = $1, update_time = NOW() WHERE id = $2")
        .bind(new_size)
        .bind(task_id)
        .execute(&state.db)
        .await?;

    Ok(axum::json::Json(ApiResponse::success(serde_json::json!({
        "task_id": task_id,
        "chunk_index": chunk_index,
        "offset": new_size,
        "uploaded_size": new_size
    }))))
}

async fn complete_upload(
    State(state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> AppResult<impl axum::response::IntoResponse> {
    let task_id = payload.get("task_id").and_then(|v| v.as_i64())
        .ok_or_else(|| crate::error::AppError::BadRequest("Missing task_id".to_string()))?;

    let task = sqlx::query_as::<_, (String, i64, String, i64)>(
        "SELECT task_no, file_size, remote_path, uploaded_size FROM upload_task WHERE id = $1"
    )
    .bind(task_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound("Task not found".to_string()))?;

    if task.1 != task.3 {
        return Err(crate::error::AppError::BadRequest("File not fully uploaded".to_string()));
    }

    let task_dir = PathBuf::from(&state.config.storage.temp_dir).join(&task.0);
    let target_path = PathBuf::from(&task.2);

    if !target_path.parent().map(|p| std::fs::create_dir_all(p).is_ok()).unwrap_or(false) {
        return Err(crate::error::AppError::BadRequest("Invalid remote path".to_string()));
    }

    let mut final_file = std::fs::File::create(&target_path).map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    
    let chunks: Vec<_> = std::fs::read_dir(&task_dir)
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("chunk_"))
        .collect();

    for chunk in chunks {
        let data = std::fs::read(chunk.path()).map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
        std::io::Write::write_all(&mut final_file, &data).map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    }

    std::fs::remove_dir_all(&task_dir).ok();

    sqlx::query("UPDATE upload_task SET status = 2, update_time = NOW() WHERE id = $1")
        .bind(task_id)
        .execute(&state.db)
        .await?;

    Ok(axum::json::Json(ApiResponse::success(serde_json::json!({
        "task_id": task_id,
        "status": "completed"
    }))))
}

async fn list_upload_tasks(
    State state: State<AppState>,
) -> AppResult<impl axum::response::IntoResponse> {
    let tasks = sqlx::query_as::<_, (i64, String, i64, i64, i64, i16, String)>(
        "SELECT id, task_no, host_id, file_size, uploaded_size, status, create_time::text FROM upload_task ORDER BY id DESC LIMIT 50"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(axum::json::Json(ApiResponse::success(tasks)))
}

async fn get_upload_task(
    State state: State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> AppResult<impl axum::response::IntoResponse> {
    let task = sqlx::query_as::<_, (i64, String, i64, i64, i64, i16, String)>(
        "SELECT id, task_no, host_id, file_size, uploaded_size, status, create_time::text FROM upload_task WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound("Task not found".to_string()))?;

    Ok(axum::json::Json(ApiResponse::success(task)))
}
