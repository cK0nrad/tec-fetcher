use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::Method,
    response::Response,
    routing::get,
    Router,
};
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

use crate::{logger, store::Store};

pub async fn init(ip: String, port: String, store: Arc<Store>) {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PUT])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
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

async fn websocket(ws: WebSocketUpgrade, State(app): State<Arc<Store>>) -> Response {
    ws.on_upgrade(move |socket| async {
        handle_socket(socket, app).await;
    })
}

async fn handle_socket(mut socket: WebSocket, store: Arc<Store>) {
    println!("New connection");

    loop {
        let datas = store.retrieve_json().await;
        let _ = socket.send(Message::Text(datas)).await;
        sleep(std::time::Duration::from_secs(5)).await
    }
}
