use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    //#[error("{0}")]
    //BluetoothError(#[from] crate::broker::BrokerError),
    //#[error("{0}")]
    //WifiError(#[from] crate::broker::BrokerError),
    //#[error("{0}")]
    //DroneError(#[from] crate::broker::BrokerError),
    #[error("{0}")]
    MqttClientError(#[from] crate::mqtt_client::error::MqttClientError),
}
