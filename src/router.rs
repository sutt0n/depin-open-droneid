use axum::{
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;

// use crate::routes;

use tokio::sync::broadcast::{channel, Sender};

use crate::{models::DroneUpdate, routes};

pub type DronesStream = Sender<DroneUpdate>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn init_router(db: PgPool) -> (Router, DronesStream) {
    let (tx, _rx) = channel::<DroneUpdate>(10);
    let state = AppState { db };

    (
        Router::new()
            .route("/api/drones/active", post(routes::get_active_drones))
            .route("/api/drones/all", get(routes::get_all_drones))
            .route("/api/stream", get(routes::handle_stream))
            .with_state(state)
            .layer(Extension(tx.clone())),
        tx,
    )
}
