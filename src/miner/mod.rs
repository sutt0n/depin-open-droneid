use crate::{app::TrebuchetApp, web::DroneDto};

pub async fn start_miner_task(app: TrebuchetApp) -> anyhow::Result<()> {
    // we need a loop, but we need to return Ok(()) at the end

    tokio::spawn(async move {
        loop {
            let drone = DroneDto::dummy();
            let bytes = bincode::serialize(&drone).unwrap();
            let _ = app.send_payload(bytes).await;

            println!("Sent payload");

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    })
    .await?;

    Ok(())
}
