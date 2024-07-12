use std::{collections::HashMap, sync::Arc};

use bluetooth::start_bluetooth_task;
use bluez_async::DeviceId;
use drone::Drone;

mod bluetooth;
mod drone;
mod odid;
mod web;
mod wifi;

use tokio::sync::Mutex;
use web::{init_router, start_webserver};
use wifi::{start_wifi_task, WifiInterfaceBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bluetooth_device_name = "hci0";
    let wifi_device_name = "wlx08beac26e3e8";

    let drones: Arc<Mutex<HashMap<String, Drone>>> = Arc::new(Mutex::new(HashMap::new()));

    env_logger::builder().filter_level(log::LevelFilter::Trace).init();

    let conn_url =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");

    let sqlx_connection = sqlx::PgPool::connect(&conn_url).await.unwrap();

    // run the migrations
    sqlx::migrate!()
        .run(&sqlx_connection)
        .await
        .expect("Failed to run migrations");

    let (router, tx) = init_router(sqlx_connection.clone());

    let update_tx = Arc::new(Mutex::new(tx));
    let db_pool = Arc::new(Mutex::new(sqlx_connection));

    let wifi_interface = WifiInterfaceBuilder::default()
        .name(String::from(wifi_device_name))
        .build()
        .unwrap();

    let wifi_interface = Arc::new(Mutex::new(wifi_interface));
    let wifi_interface_clone = Arc::clone(&wifi_interface);

    let _ = tokio::spawn(async move {
        let wifi_interface = wifi_interface_clone;
        loop {
            let mut wifi_interface = wifi_interface.lock().await;
            if wifi_interface.should_change_channel() {
                wifi_interface.adjust_channel();
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });

    let wifi_task = start_wifi_task(
        String::from(wifi_device_name),
        Arc::clone(&db_pool),
        Arc::clone(&drones),
        Arc::clone(&update_tx),
        Arc::clone(&wifi_interface),
    )
    .await;

    // Spawn a task to handle bluetooth events
    // let bt_task = start_bluetooth_task(
    //     String::from(bluetooth_device_name),
    //     Arc::clone(&drones),
    //     Arc::clone(&db_pool),
    //     Arc::clone(&update_tx),
    // )
    // .await;

    // Run both tasks concurrently
    let (_, _) = tokio::join!(wifi_task, start_webserver(router));

    Ok(())
}
