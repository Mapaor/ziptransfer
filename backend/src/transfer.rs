use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, head, patch, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use uuid::Uuid;
use crate::AppState;
use futures::StreamExt;
use tokio_util::io::ReaderStream;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sqlx::Row;

const UPLOADS_DIR: &str = "./uploads";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/transfer/list", get(list_transfers))
        .route("/api/transfer/new", post(new_transfer))
        .route("/api/transfer/:id", get(get_transfer).delete(delete_transfer))
        .route("/api/transfer/:id/delete", post(delete_transfer))
        .route("/api/sign", post(sign_token))
        .route("/api/upload/:id/complete", post(mark_complete))
        .route("/api/upload/:id", get(get_public_upload).head(get_upload_info).patch(append_upload))
        .route("/api/upload", post(create_upload))
        .route("/api/download/:id", get(download_file))
        .route("/api/download/:id/downloaded", post(register_downloaded))
        .route("/api/download", post(download_transfer_zip))
        .route("/api/transferrequest/list", get(list_transfer_requests))
        .route("/api/transferrequest/new", post(new_transfer_request))
        .route("/api/transferrequest/:id/activate", post(activate_transfer_request))
        .route("/api/transferrequest/:id/deactivate", post(deactivate_transfer_request))
}

async fn get_user_id(headers: &HeaderMap, state: &AppState) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    let mut token = None;
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if cookie.starts_with("token=") {
            token = Some(cookie[6..].to_string());
            break;
        }
    }
    let token = token?;
    
    let user: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE session_token = ?")
        .bind(token)
        .fetch_optional(&state.db)
        .await
        .ok().flatten();
        
    user.map(|u| u.0)
}

#[derive(Serialize)]
pub struct TransferListResponse {
    pub success: bool,
    pub transfers: Vec<TransferObj>,
}

#[derive(Serialize)]
pub struct StatisticsObj {
    pub downloads: Vec<String>,
    pub views: Vec<String>,
}

#[derive(Serialize)]
pub struct EmailEntry {
    pub email: String,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct TransferObj {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub expiresAt: String,
    pub createdAt: String,
    pub secretCode: String,
    pub status: String,
    pub files: Vec<FileObj>,
    pub statistics: StatisticsObj,
    #[serde(rename = "emailsSharedWith")]
    pub emails_shared_with: Vec<EmailEntry>,
    #[serde(rename = "finishedUploading")]
    pub finished_uploading: bool,
    #[serde(rename = "hasTransferRequest")]
    pub has_transfer_request: bool,
}

#[derive(Serialize)]
pub struct FileObj {
    pub id: String,
    pub filename: String,
    pub size: i64,
}

async fn list_transfers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TransferListResponse>, StatusCode> {
    let user_id = get_user_id(&headers, &state).await.ok_or(StatusCode::UNAUTHORIZED)?;

    let records = sqlx::query(
        "SELECT id, name, description, expires_at, created_at FROM transfers WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut transfers = Vec::new();
    for row in records {
        let transfer_id: String = row.get("id");
        let name: Option<String> = row.get("name");
        let name = name.unwrap_or_else(|| "Untitled Transfer".to_string());
        let description: Option<String> = row.get("description");
        
        let expires_at: chrono::NaiveDateTime = row.get("expires_at");
        let created_at: chrono::NaiveDateTime = row.get("created_at");

        let files_records = sqlx::query("SELECT id, filename, size FROM files WHERE transfer_id = ?")
            .bind(&transfer_id)
            .fetch_all(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
        let mut files = Vec::new();
        let mut finished_uploading = true;
        for file_row in files_records {
            let file_id: String = file_row.get("id");
            let expected_size: i64 = file_row.get("size");
            
            let disk_size = tokio::fs::metadata(format!("{}/{}", UPLOADS_DIR, file_id))
                .await
                .map(|m| m.len() as i64)
                .unwrap_or(0);
                
            if disk_size < expected_size {
                finished_uploading = false;
            }

            files.push(FileObj {
                id: file_id,
                filename: file_row.get("filename"),
                size: expected_size,
            });
        }

        transfers.push(TransferObj {
            id: transfer_id.clone(),
            name,
            description,
            expiresAt: expires_at.to_string(),
            createdAt: created_at.to_string(),
            secretCode: transfer_id, 
            status: if finished_uploading { "completed".to_string() } else { "uploading".to_string() },
            files,
            statistics: StatisticsObj {
                downloads: vec![],
                views: vec![],
            },
            emails_shared_with: vec![],
            finished_uploading,
            has_transfer_request: false,
        });
    }

    Ok(Json(TransferListResponse {
        success: true,
        transfers,
    }))
}

#[derive(Serialize)]
pub struct SingleTransferResponse {
    pub success: bool,
    pub transfer: TransferObj,
}

// GET /api/transfer/:id
async fn get_transfer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<SingleTransferResponse>, StatusCode> {
    let user_id = get_user_id(&headers, &state).await.ok_or(StatusCode::UNAUTHORIZED)?;

    let row = sqlx::query(
        "SELECT id, name, description, expires_at, created_at FROM transfers WHERE id = ? AND user_id = ?"
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(row) = row {
        let transfer_id: String = row.get("id");
        let name: Option<String> = row.get("name");
        let name = name.unwrap_or_else(|| "Untitled Transfer".to_string());
        let description: Option<String> = row.get("description");
        let expires_at: chrono::NaiveDateTime = row.get("expires_at");
        let created_at: chrono::NaiveDateTime = row.get("created_at");

        let files_records = sqlx::query("SELECT id, filename, size FROM files WHERE transfer_id = ?")
            .bind(&transfer_id)
            .fetch_all(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
        let mut files = Vec::new();
        let mut finished_uploading = true;
        for file_row in files_records {
            let file_id: String = file_row.get("id");
            let expected_size: i64 = file_row.get("size");
            let disk_size = tokio::fs::metadata(format!("{}/{}", UPLOADS_DIR, file_id))
                .await
                .map(|m| m.len() as i64)
                .unwrap_or(0);
            if disk_size < expected_size {
                finished_uploading = false;
            }
            files.push(FileObj {
                id: file_id,
                filename: file_row.get("filename"),
                size: expected_size,
            });
        }

        let transfer = TransferObj {
            id: transfer_id.clone(),
            name,
            description,
            expiresAt: expires_at.to_string(),
            createdAt: created_at.to_string(),
            secretCode: transfer_id,
            status: if finished_uploading { "completed".to_string() } else { "uploading".to_string() },
            files,
            statistics: StatisticsObj {
                downloads: vec![],
                views: vec![],
            },
            emails_shared_with: vec![],
            finished_uploading,
            has_transfer_request: false,
        };

        Ok(Json(SingleTransferResponse {
            success: true,
            transfer,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}


#[derive(Deserialize)]
pub struct NewTransferFile {
    #[serde(rename = "tmpId")]
    pub tmp_id: String,
    pub name: String,
    pub size: u64,
}

#[derive(Deserialize)]
pub struct NewTransferPayload {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "expiresInDays")]
    pub expires_in_days: Option<String>,
    pub files: Vec<NewTransferFile>,
}

#[derive(Serialize)]
pub struct TransferResponse {
    pub id: String,
    #[serde(rename = "secretCode")]
    pub secret_code: String,
    #[serde(rename = "nodeUrl")]
    pub node_url: String,
}

#[derive(Serialize)]
pub struct IdMapEntry {
    #[serde(rename = "tmpId")]
    pub tmp_id: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct NewTransferResult {
    pub success: bool,
    pub transfer: TransferResponse,
    #[serde(rename = "idMap")]
    pub id_map: Vec<IdMapEntry>,
}

async fn new_transfer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<NewTransferPayload>,
) -> Result<Json<NewTransferResult>, StatusCode> {
    let user_id = get_user_id(&headers, &state).await.ok_or(StatusCode::UNAUTHORIZED)?;
    
    let transfer_id = Uuid::new_v4().to_string();
    let secret_code = Uuid::new_v4().to_string();
    let expires_in = payload.expires_in_days.unwrap_or_else(|| "1".to_string()).parse::<i64>().unwrap_or(1);
    let expires_at = chrono::Utc::now() + chrono::Duration::days(expires_in);

    sqlx::query("INSERT INTO transfers (id, user_id, type, expires_at, name, description) VALUES (?, ?, 'persistent', ?, ?, ?)")
        .bind(&secret_code) 
        .bind(&user_id)
        .bind(expires_at.naive_utc())
        .bind(&payload.name)
        .bind(&payload.description)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut id_map = Vec::new();
    
    for file in payload.files {
        let file_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO files (id, transfer_id, filename, size, disk_path) VALUES (?, ?, ?, ?, ?)")
            .bind(&file_id)
            .bind(&secret_code)
            .bind(&file.name)
            .bind(file.size as i64)
            .bind(&file_id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
        id_map.push(IdMapEntry {
            tmp_id: file.tmp_id,
            id: file_id,
        });
    }

    Ok(Json(NewTransferResult {
        success: true,
        transfer: TransferResponse {
            id: transfer_id,
            secret_code: secret_code,
            node_url: "/api".to_string(), 
        },
        id_map,
    }))
}

#[derive(Deserialize)]
pub struct SignPayload {
    #[serde(rename = "secretCode")]
    pub secret_code: String,
}

#[derive(Serialize)]
pub struct SignResult {
    pub success: bool,
    pub token: String,
    #[serde(rename = "nodeUrl")]
    pub node_url: String,
}

async fn sign_token(Json(payload): Json<SignPayload>) -> Json<SignResult> {
    Json(SignResult {
        success: true,
        token: payload.secret_code,
        node_url: "/api".to_string(),
    })
}

#[derive(Serialize)]
pub struct GenericResult {
    pub success: bool,
}

async fn mark_complete(Path(_id): Path<String>) -> Json<GenericResult> {
    Json(GenericResult { success: true })
}

async fn ensure_uploads_dir() {
    let _ = tokio::fs::create_dir_all(UPLOADS_DIR).await;
}

// POST /api/upload
async fn create_upload(
    State(_state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    ensure_uploads_dir().await;
    
    let metadata_header = headers.get("Upload-Metadata").and_then(|h| h.to_str().ok()).unwrap_or("");
    tracing::info!("create_upload called with metadata: {}", metadata_header);
    
    let mut file_id = Uuid::new_v4().to_string(); 
    for pair in metadata_header.split(',') {
        let parts: Vec<&str> = pair.trim().split(' ').collect();
        if parts.len() == 2 && parts[0] == "id" {
            if let Ok(decoded) = BASE64.decode(parts[1]) {
                if let Ok(id_str) = String::from_utf8(decoded) {
                    file_id = id_str;
                }
            }
        }
    }

    let file_path = format!("{}/{}", UPLOADS_DIR, file_id);
    tokio::fs::File::create(&file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut response_headers = HeaderMap::new();
    response_headers.insert("Location", format!("/api/upload/{}", file_id).parse().unwrap());
    response_headers.insert("Tus-Resumable", "1.0.0".parse().unwrap());

    Ok((StatusCode::CREATED, response_headers))
}

// HEAD /api/upload/:id
async fn get_upload_info(Path(id): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("get_upload_info called for id: {}", id);
    let file_path = format!("{}/{}", UPLOADS_DIR, id);
    let metadata = tokio::fs::metadata(&file_path).await.map_err(|_| {
        tracing::error!("get_upload_info file not found: {}", file_path);
        StatusCode::NOT_FOUND
    })?;

    let mut headers = HeaderMap::new();
    headers.insert("Upload-Offset", metadata.len().to_string().parse().unwrap());
    headers.insert("Tus-Resumable", "1.0.0".parse().unwrap());
    headers.insert("Cache-Control", "no-store".parse().unwrap());

    Ok((StatusCode::OK, headers))
}

// PATCH /api/upload/:id
async fn append_upload(
    Path(id): Path<String>,
    headers: HeaderMap,
    request: Request<Body>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("append_upload called for id: {}", id);
    let upload_offset = headers
        .get("Upload-Offset")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| {
            tracing::error!("append_upload missing Upload-Offset header");
            StatusCode::BAD_REQUEST
        })?;

    let file_path = format!("{}/{}", UPLOADS_DIR, id);
    let mut file = OpenOptions::new()
        .write(true)
        .open(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let current_size = file.metadata().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.len();
    
    if current_size != upload_offset {
        return Err(StatusCode::CONFLICT);
    }

    file.seek(SeekFrom::Start(upload_offset)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut stream = request.into_body().into_data_stream();
    let mut written = 0;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write_all(&chunk).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        written += chunk.len() as u64;
    }

    let mut response_headers = HeaderMap::new();
    response_headers.insert("Upload-Offset", (current_size + written).to_string().parse().unwrap());
    response_headers.insert("Tus-Resumable", "1.0.0".parse().unwrap());

    Ok((StatusCode::NO_CONTENT, response_headers))
}

// GET /api/download/:id
async fn download_file(Path(id): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let file_path = format!("{}/{}", UPLOADS_DIR, id);
    let file = File::open(&file_path).await.map_err(|_| StatusCode::NOT_FOUND)?;
    
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
    headers.insert(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", id).parse().unwrap());

    Ok((headers, body))
}

#[derive(Deserialize)]
pub struct DownloadForm {
    token: String,
}

// POST /api/download
async fn download_transfer_zip(
    State(state): State<AppState>,
    axum::extract::Form(form): axum::extract::Form<DownloadForm>,
) -> Result<impl IntoResponse, StatusCode> {
    let secret_code = form.token; 
    let file = sqlx::query("SELECT id, filename FROM files WHERE transfer_id = ? LIMIT 1")
        .bind(&secret_code)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(row) = file {
        let file_id: String = row.get("id");
        let filename: String = row.get("filename");
        
        let file_path = format!("{}/{}", UPLOADS_DIR, file_id);
        let file = tokio::fs::File::open(&file_path).await.map_err(|_| StatusCode::NOT_FOUND)?;
        
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
        headers.insert(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename).parse().unwrap());

        Ok((headers, body))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// POST /api/download/:id/downloaded
async fn register_downloaded(Path(_id): Path<String>) -> Json<GenericResult> {
    Json(GenericResult { success: true })
}

#[derive(Serialize)]
pub struct PublicUploadResponse {
    pub success: bool,
    pub upload: TransferObj,
}

// GET /api/upload/:id
async fn get_public_upload(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PublicUploadResponse>, StatusCode> {
    let row = sqlx::query(
        "SELECT id, name, description, expires_at, created_at FROM transfers WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(row) = row {
        let transfer_id: String = row.get("id");
        let name: Option<String> = row.get("name");
        let name = name.unwrap_or_else(|| "Untitled Transfer".to_string());
        let description: Option<String> = row.get("description");
        let expires_at: chrono::NaiveDateTime = row.get("expires_at");
        let created_at: chrono::NaiveDateTime = row.get("created_at");

        let files_records = sqlx::query("SELECT id, filename, size FROM files WHERE transfer_id = ?")
            .bind(&transfer_id)
            .fetch_all(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
        let mut files = Vec::new();
        let mut finished_uploading = true;
        for file_row in files_records {
            let file_id: String = file_row.get("id");
            let expected_size: i64 = file_row.get("size");
            let disk_size = tokio::fs::metadata(format!("{}/{}", UPLOADS_DIR, file_id))
                .await
                .map(|m| m.len() as i64)
                .unwrap_or(0);
            if disk_size < expected_size {
                finished_uploading = false;
            }
            files.push(FileObj {
                id: file_id,
                filename: file_row.get("filename"),
                size: expected_size,
            });
        }

        let transfer = TransferObj {
            id: transfer_id.clone(),
            name,
            description,
            expiresAt: expires_at.to_string(),
            createdAt: created_at.to_string(),
            secretCode: transfer_id,
            status: if finished_uploading { "completed".to_string() } else { "uploading".to_string() },
            files,
            statistics: StatisticsObj {
                downloads: vec![],
                views: vec![],
            },
            emails_shared_with: vec![],
            finished_uploading,
            has_transfer_request: false,
        };

        Ok(Json(PublicUploadResponse {
            success: true,
            upload: transfer,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn delete_transfer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<GenericResult>, StatusCode> {
    let user_id = get_user_id(&headers, &state).await.ok_or(StatusCode::UNAUTHORIZED)?;

    let record = sqlx::query("SELECT id FROM transfers WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if record.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let files = sqlx::query("SELECT id FROM files WHERE transfer_id = ?")
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for row in files {
        let file_id: String = row.get("id");
        let file_path = format!("{}/{}", UPLOADS_DIR, file_id);
        let _ = tokio::fs::remove_file(file_path).await;
    }

    sqlx::query("DELETE FROM files WHERE transfer_id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("DELETE FROM transfers WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(GenericResult { success: true }))
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct TransferRequestDto {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub secretCode: String,
    pub active: bool,
    pub createdAt: String,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct TransferRequestListResponse {
    pub success: bool,
    pub transferRequests: Vec<TransferRequestDto>,
}

async fn list_transfer_requests(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TransferRequestListResponse>, StatusCode> {
    let user_id = get_user_id(&headers, &state).await.ok_or(StatusCode::UNAUTHORIZED)?;

    let records = sqlx::query(
        "SELECT id, secret_code, name, description, active, created_at FROM transfer_requests WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut transfer_requests = Vec::new();
    for row in records {
        let id: String = row.get("id");
        let secret_code: String = row.get("secret_code");
        let name: Option<String> = row.get("name");
        let description: Option<String> = row.get("description");
        let active: bool = row.get("active");
        let created_at: chrono::NaiveDateTime = row.get("created_at");

        transfer_requests.push(TransferRequestDto {
            id,
            name,
            description,
            secretCode: secret_code,
            active,
            createdAt: created_at.to_string(),
        });
    }

    Ok(Json(TransferRequestListResponse {
        success: true,
        transferRequests: transfer_requests,
    }))
}

#[derive(Deserialize)]
pub struct NewTransferRequestPayload {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct NewTransferRequestResponse {
    pub success: bool,
    pub transferRequest: TransferRequestDto,
}

async fn new_transfer_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<NewTransferRequestPayload>,
) -> Result<Json<NewTransferRequestResponse>, StatusCode> {
    let user_id = get_user_id(&headers, &state).await.ok_or(StatusCode::UNAUTHORIZED)?;
    
    let id = Uuid::new_v4().to_string();
    let secret_code = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO transfer_requests (id, user_id, secret_code, name, description, active) VALUES (?, ?, ?, ?, ?, 1)")
        .bind(&id)
        .bind(&user_id)
        .bind(&secret_code)
        .bind(&payload.name)
        .bind(&payload.description)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let transfer_request = TransferRequestDto {
        id,
        name: payload.name,
        description: payload.description,
        secretCode: secret_code,
        active: true,
        createdAt: chrono::Utc::now().naive_utc().to_string(),
    };

    Ok(Json(NewTransferRequestResponse {
        success: true,
        transferRequest: transfer_request,
    }))
}

async fn activate_transfer_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<GenericResult>, StatusCode> {
    let user_id = get_user_id(&headers, &state).await.ok_or(StatusCode::UNAUTHORIZED)?;

    sqlx::query("UPDATE transfer_requests SET active = 1 WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(GenericResult { success: true }))
}

async fn deactivate_transfer_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<GenericResult>, StatusCode> {
    let user_id = get_user_id(&headers, &state).await.ok_or(StatusCode::UNAUTHORIZED)?;

    sqlx::query("UPDATE transfer_requests SET active = 0 WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(GenericResult { success: true }))
}

