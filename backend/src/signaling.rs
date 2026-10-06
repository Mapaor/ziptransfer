use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use crate::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[derive(Deserialize, Debug)]
struct ClientPacket {
    #[serde(rename = "type")]
    packet_type: i32,
    id: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    #[serde(rename = "callerId")]
    caller_id: Option<String>,
    #[serde(rename = "recipientId")]
    recipient_id: Option<String>,
    offer: Option<serde_json::Value>,
    answer: Option<serde_json::Value>,
    candidate: Option<serde_json::Value>,
    #[serde(rename = "currentUserCanFallback")]
    current_user_can_fallback: Option<bool>,
    success: Option<bool>,
}

#[derive(Serialize)]
struct ServerPacket {
    #[serde(rename = "type")]
    packet_type: i32,
    #[serde(rename = "targetId", skip_serializing_if = "Option::is_none")]
    target_id: Option<String>,
    #[serde(rename = "callerId", skip_serializing_if = "Option::is_none")]
    caller_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offer: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    answer: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate: Option<serde_json::Value>,
    #[serde(rename = "currentUserCanFallback", skip_serializing_if = "Option::is_none")]
    current_user_can_fallback: Option<bool>,
    #[serde(rename = "newRtcSessionId", skip_serializing_if = "Option::is_none")]
    new_rtc_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success: Option<bool>,
}

const CPKT_LOGOUT: i32 = -1;
const CPKT_LOGIN: i32 = 0;
const CPKT_OFFER: i32 = 1;
const CPKT_ANSWER: i32 = 2;
const CPKT_CANDIDATE: i32 = 3;
const CPKT_SWITCH_TO_FALLBACK: i32 = 5;
const CPKT_SWITCH_TO_FALLBACK_ACK: i32 = 6;
const CPKT_P2P_FAILED: i32 = 7;

const SPKT_OFFER: i32 = 11;
const SPKT_ANSWER: i32 = 12;
const SPKT_CANDIDATE: i32 = 13;
const SPKT_SWITCH_TO_FALLBACK: i32 = 15;
const SPKT_SWITCH_TO_FALLBACK_ACK: i32 = 16;
const SPKT_P2P_FAILED: i32 = 17;

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let mut current_session: Option<String> = None;
    let tx_clone = tx.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg_enum)) = receiver.next().await {
            match msg_enum {
                Message::Text(text) => {
                    if text == "." {
                        continue;
                    }
                    if let Ok(msg) = serde_json::from_str::<ClientPacket>(&text) {
                        match msg.packet_type {
                    CPKT_LOGOUT => {
                        if let Some(session_id) = msg.session_id {
                            state_clone.signaling.remove(&session_id);
                        }
                    }
                    CPKT_LOGIN => {
                        if let Some(id) = msg.id {
                            state_clone.signaling.insert(id.clone(), tx_clone.clone());
                            current_session = Some(id);
                        }
                    }
                    CPKT_OFFER => {
                        if let (Some(recipient), Some(caller), Some(offer)) = (msg.recipient_id, msg.caller_id, msg.offer) {
                            if let Some(peer_tx) = state_clone.signaling.get(&recipient) {
                                let out = ServerPacket {
                                    packet_type: SPKT_OFFER,
                                    target_id: Some(recipient.clone()),
                                    caller_id: Some(caller),
                                    offer: Some(offer),
                                    answer: None,
                                    candidate: None,
                                    current_user_can_fallback: None,
                                    new_rtc_session_id: None,
                                    success: None,
                                };
                                let _ = peer_tx.send(Message::Text(serde_json::to_string(&out).unwrap()));
                            }
                        }
                    }
                    CPKT_ANSWER => {
                        if let (Some(recipient), Some(session_id), Some(answer)) = (msg.recipient_id, msg.session_id, msg.answer) {
                            if let Some(peer_tx) = state_clone.signaling.get(&recipient) {
                                let out = ServerPacket {
                                    packet_type: SPKT_ANSWER,
                                    target_id: Some(recipient.clone()),
                                    caller_id: Some(session_id),
                                    offer: None,
                                    answer: Some(answer),
                                    candidate: None,
                                    current_user_can_fallback: msg.current_user_can_fallback,
                                    new_rtc_session_id: None,
                                    success: None,
                                };
                                let _ = peer_tx.send(Message::Text(serde_json::to_string(&out).unwrap()));
                            }
                        }
                    }
                    CPKT_CANDIDATE => {
                        if let (Some(recipient), Some(caller), Some(candidate)) = (msg.recipient_id, msg.caller_id, msg.candidate) {
                            if let Some(peer_tx) = state_clone.signaling.get(&recipient) {
                                let out = ServerPacket {
                                    packet_type: SPKT_CANDIDATE,
                                    target_id: Some(recipient.clone()),
                                    caller_id: Some(caller),
                                    offer: None,
                                    answer: None,
                                    candidate: Some(candidate),
                                    current_user_can_fallback: None,
                                    new_rtc_session_id: None,
                                    success: None,
                                };
                                let _ = peer_tx.send(Message::Text(serde_json::to_string(&out).unwrap()));
                            }
                        }
                    }
                    CPKT_SWITCH_TO_FALLBACK => {
                        if let (Some(recipient), Some(caller)) = (msg.recipient_id, msg.caller_id) {
                            if let Some(peer_tx) = state_clone.signaling.get(&recipient) {
                                let out = ServerPacket {
                                    packet_type: SPKT_SWITCH_TO_FALLBACK,
                                    target_id: Some(recipient.clone()),
                                    caller_id: Some(caller),
                                    offer: None,
                                    answer: None,
                                    candidate: None,
                                    current_user_can_fallback: None,
                                    new_rtc_session_id: Some(uuid::Uuid::new_v4().to_string()),
                                    success: None,
                                };
                                let _ = peer_tx.send(Message::Text(serde_json::to_string(&out).unwrap()));
                            }
                        }
                    }
                    CPKT_SWITCH_TO_FALLBACK_ACK => {
                        if let (Some(recipient), Some(caller)) = (msg.recipient_id, msg.caller_id) {
                            if let Some(peer_tx) = state_clone.signaling.get(&recipient) {
                                let out = ServerPacket {
                                    packet_type: SPKT_SWITCH_TO_FALLBACK_ACK,
                                    target_id: Some(recipient.clone()),
                                    caller_id: Some(caller),
                                    offer: None,
                                    answer: None,
                                    candidate: None,
                                    current_user_can_fallback: None,
                                    new_rtc_session_id: None,
                                    success: msg.success,
                                };
                                let _ = peer_tx.send(Message::Text(serde_json::to_string(&out).unwrap()));
                            }
                        }
                    }
                    CPKT_P2P_FAILED => {
                        if let (Some(recipient), Some(caller)) = (msg.recipient_id, msg.caller_id) {
                            if let Some(peer_tx) = state_clone.signaling.get(&recipient) {
                                let out = ServerPacket {
                                    packet_type: SPKT_P2P_FAILED,
                                    target_id: Some(recipient.clone()),
                                    caller_id: Some(caller),
                                    offer: None,
                                    answer: None,
                                    candidate: None,
                                    current_user_can_fallback: None,
                                    new_rtc_session_id: None,
                                    success: None,
                                };
                                let _ = peer_tx.send(Message::Text(serde_json::to_string(&out).unwrap()));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Message::Binary(mut bin) => {
            if bin.len() >= 17 && bin[0] == 4 { // CPKT_RELAY
                let target_id_bytes = &bin[1..9];
                if let Ok(target_id) = String::from_utf8(target_id_bytes.to_vec()) {
                    if let Some(peer_tx) = state_clone.signaling.get(&target_id) {
                        bin[0] = 14; // SPKT_RELAY
                        let _ = peer_tx.send(Message::Binary(bin));
                    }
                }
            }
        }
        _ => {}
    }
}

        if let Some(session_id) = current_session {
            state_clone.signaling.remove(&session_id);
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
