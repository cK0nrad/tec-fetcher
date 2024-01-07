use std::sync::Arc;

use crate::store::Store;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use base64::{engine::general_purpose, Engine as _};
use flate2::{write::GzEncoder, Compression};
use std::io::Write;
use tokio::time::sleep;

pub async fn websocket(ws: WebSocketUpgrade, State(app): State<Arc<Store>>) -> Response {
    ws.on_upgrade(move |socket| async {
        handle_socket(socket, app).await;
    })
}

async fn handle_socket(mut socket: WebSocket, store: Arc<Store>) {
    println!("New connection");

    loop {
        let datas = store.retrieve_json().await;
        let message = general_purpose::STANDARD.encode(&datas);

        let message = match compress_string(&message) {
            Ok(message) => message,
            Err(e) => {
                println!("Error compressing message: {}", e);
                return;
            }
        };

        let message = Message::Binary(message);

        if socket.send(message).await.is_err() {
            println!("Error sending message");
            return;
        }

        sleep(std::time::Duration::from_secs(5)).await
    }
}

fn compress_string(input: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(input.as_bytes())?;
    encoder.finish()
}
