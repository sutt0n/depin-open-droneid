use std::net::SocketAddr;

use axum::Router;

use super::WebConfig;

pub async fn start_webserver(config: WebConfig, router: Router) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let _ = axum::serve(listener, router).await.unwrap();

    Ok(())
}
