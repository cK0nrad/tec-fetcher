use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};

use crate::store::Store;

//axum erro
pub async fn avg_speed(State(app): State<Arc<Store>>) -> impl IntoResponse {
    let avg_speed = app.get_speeds();
    let val = avg_speed
        .iter()
        .map(|x| {
            let key = x.key();
            let value = x.value().read().unwrap();

            json!({
                "bus": key,
                "speed": value.speed_average,
                "expire": value.expire,
                "count": value.speeds.len()
            })
        })
        .collect::<Vec<Value>>();
    (StatusCode::OK, Json(val))
}
