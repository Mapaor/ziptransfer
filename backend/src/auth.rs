use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use crate::AppState;

#[derive(Deserialize)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> impl IntoResponse {
    tracing::info!("Registering user with email: {}", payload.email);

    let allow_signups = std::env::var("ALLOW_SIGNUPS").unwrap_or_else(|_| "true".to_string());
    if allow_signups.to_lowercase() == "false" {
        tracing::warn!("Signup attempt rejected because ALLOW_SIGNUPS is false");
        return (
            axum::http::HeaderMap::new(),
            axum::Json(AuthResponse {
                success: false,
                id: None,
                token: None,
                message: Some("Signups are currently disabled".to_string()),
            }),
        );
    }

    let user_id = Uuid::new_v4().to_string();
    
    // Note: for production we will use argon2 here, keeping it simple for now
    let hash = payload.password; 

    let res = sqlx::query("INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)")
        .bind(user_id.clone())
        .bind(payload.email.clone())
        .bind(hash)
        .execute(&state.db)
        .await;

    if res.is_err() {
        tracing::error!("Failed to register user: {}", payload.email);
        return (
            HeaderMap::new(),
            Json(AuthResponse {
                success: false,
                id: None,
                token: None,
                message: Some("Failed to register user".to_string()),
            }),
        );
    }

    let session_token = Uuid::new_v4().to_string();
    tracing::info!("Successfully registered user: {}", payload.email);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        format!("token={}; Path=/; HttpOnly", session_token).parse().unwrap(),
    );

    // Update the session token in the DB
    let _ = sqlx::query("UPDATE users SET session_token = ? WHERE id = ?")
        .bind(&session_token)
        .bind(&user_id)
        .execute(&state.db)
        .await;

    (
        headers,
        Json(AuthResponse {
            success: true,
            id: Some(user_id),
            token: Some(session_token),
            message: None,
        }),
    )
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> impl IntoResponse {
    tracing::info!("Attempting login for email: {}", payload.email);
    let user: Option<(String, String)> = sqlx::query_as("SELECT id, password_hash FROM users WHERE email = ?")
        .bind(payload.email.clone())
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    if let Some(record) = user {
        // Note: verify hash with argon2 in production
        if record.1 == payload.password {
            let session_token = Uuid::new_v4().to_string();
            tracing::info!("Login successful for email: {}", payload.email);
            let mut headers = HeaderMap::new();
            headers.insert(
                header::SET_COOKIE,
                format!("token={}; Path=/; HttpOnly", session_token).parse().unwrap(),
            );

            // Update the session token in the DB
            let _ = sqlx::query("UPDATE users SET session_token = ? WHERE id = ?")
                .bind(&session_token)
                .bind(&record.0)
                .execute(&state.db)
                .await;

            return (
                headers,
                Json(AuthResponse {
                    success: true,
                    id: Some(record.0),
                    token: Some(session_token),
                    message: None,
                }),
            );
        }
    }

    tracing::warn!("Invalid credentials for email: {}", payload.email);
    (
        HeaderMap::new(),
        Json(AuthResponse {
            success: false,
            id: None,
            token: None,
            message: Some("Invalid credentials".to_string()),
        }),
    )
}

#[derive(Serialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub created_at: String,
    pub plan: String,
}

#[derive(Serialize)]
pub struct UserProfileResponse {
    pub success: bool,
    pub user: UserProfile,
}

pub async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserProfileResponse>, StatusCode> {
    let cookie_header = headers.get(header::COOKIE).and_then(|h| h.to_str().ok()).unwrap_or("");
    let mut token = None;
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if cookie.starts_with("token=") {
            token = Some(cookie[6..].to_string());
            break;
        }
    }
    let token = token.ok_or(StatusCode::UNAUTHORIZED)?;

    let user: Option<(String, String, chrono::NaiveDateTime)> = sqlx::query_as("SELECT id, email, created_at FROM users WHERE session_token = ?")
        .bind(token)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some((id, email, created_at)) = user {
        Ok(Json(UserProfileResponse {
            success: true,
            user: UserProfile {
                id,
                email,
                created_at: created_at.to_string(),
                plan: "pro".to_string(), // bypass stripe since it's free for family
            }
        }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub success: bool,
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let cookie_header = headers.get(header::COOKIE).and_then(|h| h.to_str().ok()).unwrap_or("");
    let mut token = None;
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if cookie.starts_with("token=") {
            token = Some(cookie[6..].to_string());
            break;
        }
    }

    if let Some(t) = token {
        let _ = sqlx::query("UPDATE users SET session_token = NULL WHERE session_token = ?")
            .bind(t)
            .execute(&state.db)
            .await;
    }

    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::SET_COOKIE,
        "token=; Path=/; HttpOnly; Max-Age=0".parse().unwrap(),
    );

    (response_headers, Json(LogoutResponse { success: true }))
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub async fn put_user_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(_payload): Json<Value>,
) -> Result<Json<GenericResponse>, StatusCode> {
    let cookie_header = headers.get(header::COOKIE).and_then(|h| h.to_str().ok()).unwrap_or("");
    let mut token = None;
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if cookie.starts_with("token=") {
            token = Some(cookie[6..].to_string());
            break;
        }
    }
    let token = token.ok_or(StatusCode::UNAUTHORIZED)?;
    
    let user_id: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE session_token = ?")
        .bind(token)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    if user_id.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // For now we just return success, we don't save anything since settings are minimal
    Ok(Json(GenericResponse {
        success: true,
        message: None,
    }))
}

#[derive(Deserialize)]
pub struct PasswordResetRequestPayload {
    pub email: String,
}

pub async fn passwordreset_request(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetRequestPayload>,
) -> impl IntoResponse {
    let token = Uuid::new_v4().to_string();
    let _ = sqlx::query("UPDATE users SET reset_token = ? WHERE email = ?")
        .bind(&token)
        .bind(&payload.email)
        .execute(&state.db)
        .await;

    // Email delivery is not configured, so the token remains available through the normal test flow.
    tracing::info!("Password reset requested for {}", payload.email);

    Json(GenericResponse { success: true, message: None })
}

#[derive(Deserialize)]
pub struct PasswordResetDoPayload {
    pub email: String,
    pub token: String,
    #[serde(rename = "newPass")]
    pub new_pass: String,
}

pub async fn passwordreset_do(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetDoPayload>,
) -> impl IntoResponse {
    let res = sqlx::query("UPDATE users SET password_hash = ?, reset_token = NULL WHERE email = ? AND reset_token = ?")
        .bind(&payload.new_pass)
        .bind(&payload.email)
        .bind(&payload.token)
        .execute(&state.db)
        .await;

    match res {
        Ok(result) if result.rows_affected() > 0 => Json(GenericResponse { success: true, message: None }),
        _ => Json(GenericResponse { success: false, message: Some("Invalid token or email".into()) }),
    }
}
