use thiserror::Error;

#[derive(Error, Debug, Clone)]
/// The crate's Error type
pub enum TransportError {
  /// Error to proxy rmp_serde encoding errors
  #[error("Failed to serialize payload {0}")]
  SerializationError(String),
  /// Error to proxy rmp_serde decoding errors
  #[error("Failed to deserialize payload {0}")]
  DeserializationError(String),
  /// Error used when converting to a variant fails
  #[error("Payload conversion error")]
  PayloadConversionError(String),
  /// Error used when a payload is invalid or invalidated
  #[error("Invalid payload")]
  Invalid,
  /// Error from the payload
  #[error("{0}")]
  Error(String),
  /// Exception from the payload
  #[error("{0}")]
  Exception(String),

  /// General errors
  #[error("General error : {0}")]
  Other(String),
}
