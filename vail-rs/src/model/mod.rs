use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateHostRequest {
    pub name: String,
    pub hostname: String,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub credential_type: Option<String>,
    pub credential_data: Option<String>,
    pub description: Option<String>,
    pub tags: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateHostRequest {
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub credential_type: Option<String>,
    pub credential_data: Option<String>,
    pub description: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub status: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HostResponse {
    pub id: i64,
    pub name: String,
    pub hostname: String,
    pub port: i32,
    pub username: Option<String>,
    pub credential_type: Option<String>,
    pub description: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub status: i16,
    pub create_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUploadTaskRequest {
    pub host_id: i64,
    pub remote_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_md5: Option<String>,
    pub chunk_size: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadTaskResponse {
    pub id: i64,
    pub task_no: String,
    pub status: i16,
    pub uploaded_size: i64,
    pub file_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadChunkRequest {
    pub task_id: i64,
    pub chunk_index: i32,
    pub offset: i64,
    pub content: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: u16, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }
}
