use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::{sse::Event, IntoResponse, Response, Sse},
    Extension,
};
use serde_json::json;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt as _};

use crate::{errors::ApiError, router::AppState, templates};
use crate::{
    models::{DroneDto, DroneUpdate, MutationKind},
    router::DronesStream,
};

pub async fn home() -> impl IntoResponse {
    templates::HomeTemplate
}

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

pub async fn update_drone(drone: DroneDto, db: &sqlx::PgPool, tx: &DronesStream) {
    let drone_copy = drone.clone();
    let _ = sqlx::query(
        "UPDATE drones SET
        serial_number = $1,
        latitude = $2,
        longitude = $3,
        altitude = $4,
        yaw = $5,
        x_speed = $6,
        y_speed = $7,
        pilot_latitude = $8,
        pilot_longitude = $9,
        home_latitude = $10,
        home_longitude = $11
    WHERE id = $12",
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
    .bind(drone.id)
    .execute(db)
    .await
    .unwrap();

    let _ = tx.send(DroneUpdate {
        mutation_kind: MutationKind::Update,
        drone: drone_copy.into(),
        id: drone.id,
    });
}

pub async fn insert_drone(drone: DroneDto, db: &sqlx::PgPool, tx: &DronesStream) -> DroneDto {
    let drone = sqlx::query_as::<_, DroneDto>(
        "INSERT INTO drones (
        serial_number,
        latitude,
        longitude,
        altitude,
        yaw,
        x_speed, y_speed,
        pilot_latitude, pilot_longitude,
        home_latitude, home_longitude
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id, created, serial_number, latitude, longitude, altitude, yaw, x_speed, y_speed, pilot_latitude, pilot_longitude, home_latitude, home_longitude",
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
        drone: drone.clone().into(),
        id: drone.id,
    });

    drone
}

pub async fn handle_stream(
    ws: WebSocketUpgrade,
    Extension(tx): Extension<DronesStream>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = tx.subscribe();

    let stream = BroadcastStream::new(rx);

    Sse::new(
        stream
            .map(|msg| {
                let msg = msg.unwrap();
                let json = format!("{}", json!(msg));
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
