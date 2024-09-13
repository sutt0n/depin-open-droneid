pub mod config;
mod db;

use anyhow::Context;
use clap::Parser;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    app::TrebuchetApp, bluetooth::start_bluetooth_task, web::init_router, wifi::WifiInterface,
};

use self::config::{Config, EnvOverride};

#[derive(Parser)]
#[clap(long_about = None)]
struct Cli {
    #[clap(short, long, env = "TREBUCHET_CONFIG", value_name = "FILE")]
    config: Option<PathBuf>,
    #[clap(env = "PG_CON")]
    pg_con: String,
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = Config::from_path(cli.config, EnvOverride { db_con: cli.pg_con })?;

    run_cmd(config).await?;

    Ok(())
}

async fn run_cmd(config: Config) -> anyhow::Result<()> {
    //crate::tracing::init_tracer(config.tracing)?;
    //console_subscriber::init();

    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let (send, mut receive) = tokio::sync::mpsc::channel(1);
    let mut handles = vec![];
    let pool = db::init_pool(&config.db).await?;
    let app = TrebuchetApp::init(pool.clone(), config.app.clone()).await?;

    let (router, drone_update_tx) = init_router(pool.clone());

    let ts_app = Arc::new(app.clone());
    let ts_pool = Arc::new(Mutex::new(pool.clone()));
    let ts_drone_update = Arc::new(Mutex::new(drone_update_tx.clone()));

    println!("Starting Bluetooth LTE listener");
    //let bt_send = send.clone();
    //let bt_pool = Arc::clone(&ts_pool);
    //let bt_drone_update = Arc::clone(&ts_drone_update);
    //let bt_drones = Arc::clone(&app.drones);
    //handles.push(tokio::spawn(async move {
    //    let _ = bt_send.try_send(
    //        start_bluetooth_task(
    //            config.app.bluetooth.device_name.clone(),
    //            bt_drones,
    //            bt_pool,
    //            bt_drone_update,
    //        )
    //        .await
    //        .context("bluetooth task error"),
    //    );
    //}));

    println!("Starting WiFi interface modulator");
    //let wifi_interface_send = send.clone();
    //let wifi_interface = WifiInterface::init(config.app.wifi.clone()).await?;
    //let wifi_interface = Arc::new(Mutex::new(wifi_interface.clone()));
    //handles.push(tokio::spawn({
    //    let wifi_interface = Arc::clone(&wifi_interface);
    //    async move {
    //        let _ = wifi_interface_send.try_send(
    //            wifi_interface
    //                .lock()
    //                .await
    //                .run()
    //                .await
    //                .context("wifi interface task error"),
    //        );
    //    }
    //}));

    println!("Starting WiFi listener");
    //let wifi_send = send.clone();
    //let wifi_pool = Arc::clone(&ts_pool);
    //let wifi_drone_update = Arc::clone(&ts_drone_update);
    //let wifi_drones = Arc::clone(&app.drones);
    //let wifi_interface = Arc::clone(&wifi_interface);
    //handles.push(tokio::spawn(async move {
    //    let _ = wifi_send.try_send(
    //        crate::wifi::start_wifi_task(
    //            config.app.wifi.device_name.clone(),
    //            wifi_pool,
    //            wifi_drones,
    //            wifi_drone_update,
    //            wifi_interface,
    //        )
    //        .await
    //        .context("wifi task error"),
    //    );
    //}));

    println!("Starting miner");
    let miner_send = send.clone();
    handles.push(tokio::spawn(async move {
        let _ = miner_send.try_send(
            crate::miner::start_miner_task(app.clone())
                .await
                .context("miner task error"),
        );
    }));

    //println!("Starting MQTT client");
    //let mqtt_send = send.clone();
    //let mqtt_app = Arc::clone(&ts_app);
    //handles.push(tokio::spawn(async move {
    //    let _ = mqtt_send.try_send(
    //        mqtt_app
    //            .run_eventloop()
    //            .await
    //            .context("mqtt client task error"),
    //    );
    //}));

    println!("Starting Web server");
    let web_send = send.clone();
    let web_config = config.app.web;
    handles.push(tokio::spawn(async move {
        let _ = web_send.try_send(
            crate::web::start_webserver(web_config, router)
                .await
                .context("web server error"),
        );
    }));

    let reason = receive.recv().await.expect("Didn't receive msg");
    for handle in handles {
        println!("Cancelling task");
        handle.abort();
    }

    reason
}
