use crate::{app::TrebuchetApp, web::DroneDto};
use rand::Rng;

pub async fn start_miner_task(app: TrebuchetApp) -> anyhow::Result<()> {
    tokio::spawn(async move {
        loop {
            if !should_send_payload() {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }

            let drone = DroneDto::dummy();
            let bytes = bincode::serialize(&drone).unwrap();
            let _ = app.send_payload(bytes).await;
        }
    })
    .await?;

    Ok(())
}

fn should_send_payload() -> bool {
    let probability: f64 = 0.05; // 5% chance to send
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() < probability
}
