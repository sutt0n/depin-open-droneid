mod config;
pub mod error;

pub use config::*;
pub use error::*;

use rumqttc::{AsyncClient, MqttOptions, TlsConfiguration, Transport};
use std::time::Duration;

use crate::app::TrebuchetApp;

use self::error::MqttClientError;

#[derive(Clone)]
pub struct MqttClient {
    _config: MqttClientConfig,
    options: MqttOptions,
}

impl MqttClient {
    pub async fn init(config: MqttClientConfig) -> anyhow::Result<Self, MqttClientError> {
        println!("Initializing MQTT client with config: {:?}", config);
        // TODO: create unique client id; this would be the machine id
        let mut mqtt_options = MqttOptions::new("Test", config.uri.clone(), config.port);
        mqtt_options.set_keep_alive(Duration::from_secs(config.keep_alive));

        if !config.ca_cert.is_empty() {
            let ca = std::fs::read(config.ca_cert.clone()).unwrap();
            let client_key = std::fs::read(config.client_key.clone()).unwrap();
            let client_cert = std::fs::read(config.client_cert.clone()).unwrap();

            let transport = Transport::Tls(TlsConfiguration::Simple {
                ca,
                alpn: None,
                client_auth: Some((client_cert, client_key)),
            });

            mqtt_options.set_transport(transport);
        }

        Ok(Self {
            _config: config,
            options: mqtt_options.clone(),
        })
    }

    pub async fn publish(&self, payload: Vec<u8>) -> anyhow::Result<(), MqttClientError> {
        let (client, mut eventloop) = AsyncClient::new(self.options.clone(), 10);

        println!("Publishing message to topic: {}", self._config.topic);

        println!("Payload: {:?}", payload);

        client
            .publish(
                self._config.topic.clone(),
                rumqttc::QoS::AtLeastOnce,
                true,
                payload,
            )
            .await?;

        // wait until we get our payload as an outgoing memssage
        while let Ok(notification) = eventloop.poll().await {
            if let rumqttc::Event::Outgoing(rumqttc::Outgoing::Publish(_publish)) = notification {
                break;
            }
        }

        Ok(())
    }

    pub async fn run_eventloop(&self) -> anyhow::Result<(), MqttClientError> {
        let (_, mut eventloop) = AsyncClient::new(self.options.clone(), 10);

        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(notification) => println!("Received: {:?}", notification),
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}
