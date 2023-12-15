use std::sync::Arc;

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose, Engine as _};
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};
use flate2::{write::GzEncoder, Compression};
use std::io::Write;
use crate::{logger, store::Store};

pub async fn init(ip: String, port: String, store: Arc<Store>) {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PUT])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/raw", get(serve_raw))
        .route("/ws", get(websocket))
        .layer(cors)
        .with_state(store);

    let listener = match tokio::net::TcpListener::bind(format!("{}:{}", ip, port)).await {
        Ok(listener) => listener,
        Err(e) => {
            logger::critical(
                "WEBSERVER",
                format!("Error binding to {}:{}", ip, port).as_str(),
            );
            panic!("{}", e);
        }
    };
    logger::fine("WEBSERVER", format!("{}:{}", ip, port).as_str());

    axum::serve(listener, app).await.unwrap();
}

async fn serve_raw(State(app): State<Arc<Store>>) -> impl IntoResponse {
    let datas = app.raw_data().await;
    match Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(datas))
    {
        Ok(response) => Ok(response),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Error building response")),
    }
}

async fn websocket(ws: WebSocketUpgrade, State(app): State<Arc<Store>>) -> Response {
    ws.on_upgrade(move |socket| async {
        handle_socket(socket, app).await;
    })
}

fn compress_string(input: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(input.as_bytes())?;
    encoder.finish()
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
