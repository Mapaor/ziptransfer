use axum::{Json, Router, routing::get};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

#[derive(Serialize)]
pub struct IceServersResponse {
    pub ice_servers: Vec<IceServer>,
}

pub fn router() -> Router<crate::AppState> {
    Router::new().route("/api/turn-credentials", get(get_turn_credentials))
}

async fn get_turn_credentials() -> Json<IceServersResponse> {
    // 1. Always include the free Google STUN server as a fallback
    let mut ice_servers = vec![IceServer {
        urls: vec!["stun:stun.l.google.com:19302".to_string()],
        username: None,
        credential: None,
    }];

    // 2. If the user has configured their own TURN server on their VPS, add it to the list
    let shared_secret = std::env::var("TURN_SHARED_SECRET").unwrap_or_default();
    let turn_domain = std::env::var("TURN_DOMAIN").unwrap_or_default();

    if !shared_secret.is_empty() && !turn_domain.is_empty() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 86400; // 24-hour expiry

        let username = format!("{}", timestamp);

        type HmacSha1 = Hmac<Sha1>;
        if let Ok(mut mac) = HmacSha1::new_from_slice(shared_secret.as_bytes()) {
            mac.update(username.as_bytes());
            let result = mac.finalize().into_bytes();
            let credential = BASE64.encode(result);

            ice_servers.push(IceServer {
                urls: vec![
                    format!("turn:{}:3478?transport=udp", turn_domain),
                    format!("turn:{}:3478?transport=tcp", turn_domain),
                ],
                username: Some(username),
                credential: Some(credential),
            });
        }
    }

    Json(IceServersResponse { ice_servers })
}
