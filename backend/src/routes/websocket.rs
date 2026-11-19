use axum::{
    extract::{ws::{Message as WsMessage, WebSocket, WebSocketUpgrade}, State},
    response::Response,
};
use futures_util::{stream::StreamExt, SinkExt};

pub async fn handler(
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(Ok(msg)) = receiver.next().await {
        if let WsMessage::Text(text) = msg {
            let response = serde_json::json!({
                "type": "message",
                "content": text,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            if sender.send(WsMessage::Text(response.to_string())).await.is_err() {
                break;
            }
        }
    }
}