use thiserror::Error;

#[derive(Error, Debug)]
pub enum MqttClientError {
    #[error("StdError: {0}")]
    StdError(#[from] std::io::Error),
    #[error("RumQTTError - ClientError: {0}")]
    RumQTTClientError(#[from] rumqttc::ClientError),
    #[error("RumQTTError - ConnectionError: {0}")]
    RumQTTConnectionError(#[from] rumqttc::ConnectionError),
}
