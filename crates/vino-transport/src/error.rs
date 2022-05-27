use thiserror::Error;

#[derive(Error, Debug, Clone)]

/// The public Error type for [crate::MessageTransport] values.
/// These errors signify a held error or an error transforming
/// a [crate::MessageTransport] into another type.
pub enum TransportError {
  /// Error to proxy codec errors.
  #[error("Serialization error: {0}")]
  SerializationError(String),

  /// Error to proxy decoding errors.
  #[error("Deserialization error: {0}")]
  DeserializationError(String),

  /// Error deserializing incoming payload.
  #[error("Error deserializing incoming payload: {0}")]
  IncomingPayload(String),

  /// Error used when a payload is invalid or invalidated.
  #[error("Invalid payload")]
  Invalid,

  /// Error used when a payload is invalid or invalidated.
  #[error("Payload was an internal signal and should not be deserialized")]
  Signal,

  /// Error from the actual payload.
  #[error("{0}")]
  Error(String),

  /// Exception from the actual payload.
  #[error("{0}")]
  Exception(String),

  /// Error returned when a stream is collected when already consumed.
  #[error("Stream has already been collected")]
  AlreadyCollected,

  /// General errors.
  #[error("General error : {0}")]
  Other(String),

  /// Error resulting from a [vino_entity::Entity], usually related to parsing an Entity url.
  #[error(transparent)]
  #[cfg(feature = "invocation")]
  Entity(#[from] vino_entity::Error),
}
