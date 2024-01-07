use std::sync::Arc;

use crate::store::Store;
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn serve(State(app): State<Arc<Store>>) -> impl IntoResponse {
    let datas = app.raw_data().await;
    match Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(datas))
    {
        Ok(response) => Ok(response),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Error building response")),
    }
}
