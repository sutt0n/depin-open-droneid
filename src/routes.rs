use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, IntoResponse, Response, Sse},
    Extension,
};
use serde_json::json;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt as _};

use crate::{errors::ApiError, router::AppState};
use crate::{
    models::{DroneDto, DroneUpdate, MutationKind},
    router::DronesStream,
};

pub async fn get_active_drones(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let drones = sqlx::query_as::<_, DroneDto>(
        "SELECT * FROM drones WHERE created > NOW() - INTERVAL '10 minutes'",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&drones).unwrap())
        .unwrap())
}

pub async fn get_all_drones(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let drones = sqlx::query_as::<_, DroneDto>("SELECT * FROM drones")
        .fetch_all(&state.db)
        .await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&drones).unwrap())
        .unwrap())
}

/*
            serial_number: drone.basic_id.unwrap().uas_id,
            latitude,
            longitude,
            altitude,
            yaw,
            x_speed: speed,
            y_speed,
            pilot_latitude,
            pilot_longitude,
            home_latitude,
            home_longitude,

*/
pub async fn insert_drone(drone: DroneDto, db: &sqlx::PgPool, tx: &DronesStream) {
    let drone = sqlx::query_as::<_, DroneDto>(
        "INSERT INTO drones (
        serial_number,
        latitude,
        longtitude,
        altitude,
        yaw,
        x_speed, y_speed,
        pilot_latitude, pilot_longitude,
        home_latitude, home_longitude
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(drone.serial_number)
    .bind(drone.latitude)
    .bind(drone.longitude)
    .bind(drone.altitude)
    .bind(drone.yaw)
    .bind(drone.x_speed)
    .bind(drone.y_speed)
    .bind(drone.pilot_latitude)
    .bind(drone.pilot_longitude)
    .bind(drone.home_latitude)
    .bind(drone.home_longitude)
    .fetch_one(db)
    .await
    .unwrap();

    let _ = tx.send(DroneUpdate {
        mutation_kind: MutationKind::Create,
        id: drone.id,
    });
}

pub async fn handle_stream(
    Extension(tx): Extension<DronesStream>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = tx.subscribe();

    let stream = BroadcastStream::new(rx);

    Sse::new(
        stream
            .map(|msg| {
                let msg = msg.unwrap();
                let json = format!("<div>{}</div>", json!(msg));
                Event::default().data(json)
            })
            .map(Ok),
    )
    .keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(600))
            .text("keep-alive-text"),
    )
}
