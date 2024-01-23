use std::{net::SocketAddr, sync::Arc};

use crate::{logger, store::Store};
use axum::{http::Method, routing::get, Router};
use tower_http::cors::{Any, CorsLayer};

mod gtfs;
mod rt;
mod static_serve;
mod ws;

pub async fn init(ip: String, port: String, store: Arc<Store>) {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PUT])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/raw", get(static_serve::serve))
        .route("/ws", get(ws::websocket))
        .route("/refresh_gtfs", get(gtfs::refresh))
        .route("/avg_speed", get(rt::avg_speed))
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

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
