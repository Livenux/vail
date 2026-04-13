use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Duration, Utc};

use crate::{api::AppState, error::AppResult, model::*};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/refresh", post(refresh))
        .route("/auth/me", get(me))
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: i64,
    exp: i64,
    iat: i64,
}

pub fn create_token(user_id: i64, username: &str, secret: &str, expiration: u64) -> String {
    let now = Utc::now();
    let exp = now + Duration::seconds(expiration as i64);

    let claims = Claims {
        sub: username.to_string(),
        user_id,
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

pub fn verify_token(token: &str, secret: &str) -> AppResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| crate::error::AppError::Auth(e.to_string()))?;

    Ok(token_data.claims)
}

async fn login(
    State(state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    let user = sqlx::query_as::<_, (i64, String, String, Option<String>)>(
        "SELECT id, username, password, nickname FROM sys_user WHERE username = $1 AND deleted = 0"
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::Auth("User not found".to_string()))?;

    if !bcrypt::verify(&payload.password, &user.2).unwrap_or(false) {
        return Err(crate::error::AppError::Auth("Invalid password".to_string()));
    }

    let access_token = create_token(user.0, &user.1, &state.config.jwt.secret, state.config.jwt.expiration);
    let refresh_token = create_token(user.0, &user.1, &state.config.jwt.secret, state.config.jwt.refresh_expiration);

    sqlx::query(
        "UPDATE sys_user SET last_login_time = NOW(), last_login_ip = $1 WHERE id = $2"
    )
    .bind("0.0.0.0")
    .bind(user.0)
    .execute(&state.db)
    .await?;

    sqlx::query(
        "INSERT INTO login_log (user_id, username, ip, result, create_time) VALUES ($1, $2, $3, 1, NOW())"
    )
    .bind(user.0)
    .bind(&user.1)
    .bind("0.0.0.0")
    .execute(&state.db)
    .await?;

    Ok(axum::json::Json(ApiResponse::success(LoginResponse {
        access_token,
        refresh_token,
        expires_in: state.config.jwt.expiration,
        user: UserInfo {
            id: user.0,
            username: user.1,
            nickname: user.3,
            avatar: None,
            email: None,
        },
    })))
}

async fn logout(
    State(state): State<AppState>,
    _req: axum::extract::Json<()>,
) -> AppResult<impl IntoResponse> {
    Ok(axum::json::Json(ApiResponse::success("Logged out")))
}

async fn refresh(
    State(state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<RefreshRequest>,
) -> AppResult<impl IntoResponse> {
    let claims = verify_token(&payload.refresh_token, &state.config.jwt.secret)?;

    let access_token = create_token(
        claims.user_id,
        &claims.sub,
        &state.config.jwt.secret,
        state.config.jwt.expiration,
    );

    Ok(axum::json::Json(ApiResponse::success(serde_json::json!({
        "access_token": access_token,
        "expires_in": state.config.jwt.expiration
    }))))
}

async fn me(
    State(state): State<AppState>,
    axum::extract::HeaderMap(headers): axum::extract::HeaderMap,
) -> AppResult<impl IntoResponse> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| crate::error::AppError::Auth("Missing token".to_string()))?;

    let claims = verify_token(token, &state.config.jwt.secret)?;

    let user = sqlx::query_as::<_, (i64, String, Option<String>, Option<String>, Option<String>)>(
        "SELECT id, username, nickname, avatar, email FROM sys_user WHERE id = $1 AND deleted = 0"
    )
    .bind(claims.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::Auth("User not found".to_string()))?;

    Ok(axum::json::Json(ApiResponse::success(UserInfo {
        id: user.0,
        username: user.1,
        nickname: user.2,
        avatar: user.3,
        email: user.4,
    })))
}
