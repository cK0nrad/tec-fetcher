use std::sync::Arc;
use tokio::time::{interval, Duration};

use crate::store::Store;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use tokio::time::sleep;

pub async fn websocket(ws: WebSocketUpgrade, State(app): State<Arc<Store>>) -> Response {
    ws.on_upgrade(move |socket| async {
        handle_socket(socket, app).await;
    })
}

async fn handle_socket(mut socket: WebSocket, store: Arc<Store>) {
    let mut interval = interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        let datas = store.retrieve_json().await;
        let message = Message::Binary(datas);

        if socket.send(message).await.is_err() {
            return;
        }
    }
}
