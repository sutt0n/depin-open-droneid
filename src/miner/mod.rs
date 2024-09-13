use crate::{app::TrebuchetApp, web::DroneDto};
use rand::Rng;
use sha2::{Digest, Sha256};

pub async fn start_miner_task(app: TrebuchetApp) -> anyhow::Result<()> {
    tokio::spawn(async move {
        loop {
            if !should_send_payload() {
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
    // use sha256 and "mine" for a hash that starts with 0000
    let mut hasher = Sha256::new();
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    hasher.update(random_bytes);
    let result = hasher.finalize();
    let result = result.as_slice();

    // contains 00 at the start
    result[0] == 0 && result[1] == 0 && result[2] == 0
}
