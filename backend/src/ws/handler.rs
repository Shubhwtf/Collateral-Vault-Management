use axum::{
    extract::{ws, State, WebSocketUpgrade},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    Subscribe { vault_address: String },
    Unsubscribe { vault_address: String },
    BalanceUpdate { vault_address: String, balance: i64 },
    TransactionNotification { vault_address: String, tx_type: String, amount: i64 },
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: ws::WebSocket, _state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    let welcome = serde_json::to_string(&serde_json::json!({
        "type": "connected",
        "message": "Connected to Collateral Vault WebSocket"
    }))
    .unwrap();
    
    if sender.send(ws::Message::Text(welcome)).await.is_err() {
        return;
    }
    
    // websocket support is stubbed out for now - just logs subscriptions
    // would need a pubsub system (Redis) to actually broadcast updates across multiple backend instances
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                ws::Message::Text(text) => {
                    tracing::debug!("Received WebSocket message: {}", text);
                    
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg {
                            WsMessage::Subscribe { vault_address } => {
                                tracing::info!("Subscribed to vault: {}", vault_address);
                            }
                            WsMessage::Unsubscribe { vault_address } => {
                                tracing::info!("Unsubscribed from vault: {}", vault_address);
                            }
                            _ => {}
                        }
                    }
                }
                ws::Message::Close(_) => {
                    tracing::info!("WebSocket connection closed");
                    break;
                }
                _ => {}
            }
        } else {
            break;
        }
    }
}