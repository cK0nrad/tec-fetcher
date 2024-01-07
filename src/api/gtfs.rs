use std::{net::SocketAddr, sync::Arc};

use crate::{logger, store::Store};
use axum::{
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct Key {
    pub key: Option<String>,
}

pub async fn refresh(
    State(app): State<Arc<Store>>,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    query: Query<Key>,
) -> impl IntoResponse {
    if connect_info.ip().to_string() != "127.0.0.1" {
        logger::critical(
            "REFRESH GTFS",
            &format!("Forbidden access from {}", connect_info.ip().to_string()),
        );
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Forbidden"}))));
    }

    let key = match &query.key {
        Some(key) => key,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing key"})),
            ))
        }
    };

    match app.refresh_gtfs(key).await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({"ok": "refreshed"})))),
        Err(e) => {
            logger::critical(
                "REFRESH GTFS",
                &format!("Forbidden access from {}", connect_info.ip().to_string()),
            );
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))
        }
    }
}
