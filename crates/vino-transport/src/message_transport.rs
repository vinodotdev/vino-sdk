/// Module for the [crate::MessageTransport], [crate::TransportWrapper], and the JSON
/// representations of each.
#[cfg(feature = "async")]
pub(super) mod stream;

/// JSON-related module.
#[cfg(feature = "json")]
pub(super) mod transport_json;

/// The module for the TransportMap, a Port->[MessageTransport] map that serves as input to a component invocation.
pub(super) mod transport_map;

/// The module for TransportWrapper, a struct that includes the port a [MessageTransport] originated from.
pub(super) mod transport_wrapper;

use std::fmt::Display;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use vino_codec::error::CodecError;
#[cfg(feature = "json")]
use vino_codec::json;
use vino_codec::messagepack;
#[cfg(feature = "raw")]
use vino_codec::raw;
use vino_packet::{v0, v1, Packet};

use crate::{Error, Result};

/// The [MessageTransport] is the primary way messages are sent around Vino Networks and Schematics. It is the internal representation for normalized output [Packet]'s.
#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageTransport {
  /// A successful message.
  #[serde(rename = "0")]
  Success(Serialized),

  /// A message stemming from an error somewhere.
  #[serde(rename = "1")]
  Failure(Failure),

  #[serde(rename = "3")]
  /// An internal signal.
  Signal(MessageSignal),
}

/// A success message.
#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Serialized {
  #[serde(rename = "0")]
  /// A message carrying a payload encoded with MessagePack.
  MessagePack(Vec<u8>),

  #[serde(rename = "1")]
  #[cfg(feature = "raw")]
  /// A successful payload in a generic intermediary format.
  Struct(serde_value::Value),

  #[serde(rename = "2")]
  #[cfg(feature = "json")]
  /// A JSON String.
  Json(String),
}

impl Serialized {
  /// Deserialize a [Serialized] payload into the destination type.
  pub fn deserialize<T: DeserializeOwned>(self) -> std::result::Result<T, CodecError> {
    match self {
      Serialized::MessagePack(v) => vino_codec::messagepack::deserialize(&v),
      Serialized::Struct(v) => vino_codec::raw::deserialize(v),
      Serialized::Json(v) => vino_codec::json::deserialize(&v),
    }
  }

  /// Convert a [Serialized] payload into messagepack bytes.
  pub fn into_messagepack(self) -> Vec<u8> {
    // These unwraps *should* be OK. The internal data should be pre-validated
    // so changing between them is infallible.
    match self {
      Serialized::MessagePack(v) => v,
      Serialized::Struct(v) => vino_codec::messagepack::serialize(&v).unwrap(),
      Serialized::Json(v) => {
        vino_codec::messagepack::serialize(&vino_codec::json::deserialize::<serde_value::Value>(&v).unwrap()).unwrap()
      }
    }
  }
}

/// A Failure message.
#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Failure {
  #[serde(rename = "0")]
  /// Invalid payload. Used when a default message is unavoidable.
  Invalid,

  #[serde(rename = "1")]
  /// A message carrying an exception (an error that short-circuited a port's downstream).
  Exception(String),

  #[serde(rename = "2")]
  /// A message carrying an error (an error that short circuited all downstreams from a component).
  Error(String),
}

impl Failure {
  /// Return the inner message of a [Failure] payload.
  #[must_use]
  pub fn message(&self) -> &str {
    match self {
      Failure::Invalid => "Invalid payload",
      Failure::Exception(e) => e.as_str(),
      Failure::Error(e) => e.as_str(),
    }
  }
}

/// Internal signals that need to be handled before propagating to a downstream consumer.
#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialEq)]
pub enum MessageSignal {
  /// Indicates the job that opened this port is finished with it.
  Done,

  /// Indicates that a message is coming down in chunks and this is the start.
  OpenBracket,

  /// Indicates a chunked message has been completed.
  CloseBracket,

  /// The end state of a component run.
  Status(serde_value::Value),
}

impl MessageTransport {
  /// Returns `true` if the Message contains success data destined for a downstream
  /// consumer, `false` for Errors, Exceptions, and otherwise.
  #[must_use]
  pub fn is_ok(&self) -> bool {
    matches!(self, MessageTransport::Success(_))
  }

  #[must_use]
  /// Returns true if the [MessageTransport] is holding an Error or Exception variant.
  pub fn is_err(&self) -> bool {
    matches!(self, MessageTransport::Failure(_))
  }

  #[must_use]
  /// Returns true if the [MessageTransport] is a [MessageTransport::Signal] variant.
  pub fn is_signal(&self) -> bool {
    matches!(self, Self::Signal(_))
  }

  /// Converts the [MessageTransport] into a messagepack-compatible transport.
  pub fn to_messagepack(&mut self) {
    match &self {
      Self::Success(Serialized::MessagePack(_)) => {}
      #[cfg(feature = "raw")]
      Self::Success(Serialized::Struct(v)) => *self = Self::messagepack(&v),
      #[cfg(feature = "json")]
      Self::Success(Serialized::Json(json)) => {
        *self = match json::deserialize::<serde_value::Value>(json) {
          Ok(val) => Self::messagepack(&val),
          Err(e) => Self::error(format!("Could not convert JSON payload to MessagePack: {}", e)),
        }
      }
      _ => {}
    };
  }

  /// Creates a [MessageTransport] by serializing a passed object with messagepack
  pub fn messagepack<T: ?Sized + Serialize>(item: &T) -> Self {
    match messagepack::serialize(item) {
      Ok(bytes) => Self::Success(Serialized::MessagePack(bytes)),
      Err(e) => Self::Failure(Failure::Error(format!("Error serializing into messagepack: {}", e))),
    }
  }

  /// Creates a [MessageTransport] by serializing a passed object into a raw intermediary format
  pub fn success<T: Serialize>(item: &T) -> Self {
    #[cfg(feature = "raw")]
    match raw::serialize(item) {
      Ok(v) => Self::Success(Serialized::Struct(v)),
      Err(e) => Self::Failure(Failure::Error(format!(
        "Error serializing into raw intermediary format: {}",
        e
      ))),
    }
    #[cfg(not(feature = "raw"))]
    match messagepack::serialize(item) {
      Ok(v) => Self::Success(Serialized::MessagePack(v)),
      Err(e) => Self::Failure(Failure::Error(format!(
        "Error serializing into messagepack format: {}",
        e
      ))),
    }
  }

  #[cfg(feature = "json")]
  /// Creates a [MessageTransport] by serializing a passed object into JSON
  pub fn json<T: Serialize>(item: &T) -> Self {
    match json::serialize(item) {
      Ok(v) => Self::Success(Serialized::Json(v)),
      Err(e) => Self::Failure(Failure::Error(format!("Error serializing into json: {}", e))),
    }
  }

  /// Creates a [MessageTransport::Failure(Failure::Error)] with the passed message.
  pub fn error<T: AsRef<str>>(msg: T) -> Self {
    Self::Failure(Failure::Error(msg.as_ref().to_owned()))
  }

  /// Creates a [MessageTransport::Failure(Failure::Exception)] with the passed message.
  pub fn exception<T: AsRef<str>>(msg: T) -> Self {
    Self::Failure(Failure::Exception(msg.as_ref().to_owned()))
  }

  /// A utility function for [MessageTransport::Signal(MessageSignal::Done)]
  pub fn done() -> Self {
    MessageTransport::Signal(MessageSignal::Done)
  }

  /// Try to deserialize a [MessageTransport] into the target type
  pub fn deserialize<T: DeserializeOwned>(self) -> Result<T> {
    try_from(self)
  }
}

fn try_from<T: DeserializeOwned>(value: MessageTransport) -> Result<T> {
  match value {
    MessageTransport::Success(success) => match success {
      Serialized::MessagePack(v) => {
        messagepack::rmp_deserialize(&v).map_err(|e| Error::DeserializationError(e.to_string()))
      }
      #[cfg(feature = "raw")]
      Serialized::Struct(v) => raw::raw_deserialize(v).map_err(|e| Error::DeserializationError(e.to_string())),
      #[cfg(feature = "json")]
      Serialized::Json(v) => json::json_deserialize(&v).map_err(|e| Error::DeserializationError(e.to_string())),
    },
    MessageTransport::Failure(failure) => match failure {
      Failure::Invalid => Err(Error::Invalid),
      Failure::Exception(v) => Err(Error::Exception(v)),
      Failure::Error(v) => Err(Error::Error(v)),
    },
    MessageTransport::Signal(_) => Err(Error::Signal),
  }
}

impl From<Packet> for MessageTransport {
  fn from(output: Packet) -> MessageTransport {
    match output {
      Packet::V0(v) => match v {
        v0::Payload::Exception(v) => MessageTransport::Failure(Failure::Exception(v)),
        v0::Payload::Error(v) => MessageTransport::Failure(Failure::Error(v)),
        v0::Payload::Invalid => MessageTransport::Failure(Failure::Invalid),
        v0::Payload::MessagePack(bytes) => MessageTransport::Success(Serialized::MessagePack(bytes)),
        #[cfg(feature = "json")]
        v0::Payload::Json(v) => MessageTransport::Success(Serialized::Json(v)),
        #[cfg(not(feature = "json"))]
        v0::Payload::Json(v) => MessageTransport::success(&v),
        #[cfg(feature = "raw")]
        v0::Payload::Success(v) => MessageTransport::Success(Serialized::Struct(v)),
        #[cfg(not(feature = "raw"))]
        v0::Payload::Success(v) => MessageTransport::success(&v),
        v0::Payload::Done => MessageTransport::Signal(MessageSignal::Done),
        v0::Payload::OpenBracket => MessageTransport::Signal(MessageSignal::OpenBracket),
        v0::Payload::CloseBracket => MessageTransport::Signal(MessageSignal::CloseBracket),
      },
      Packet::V1(v) => match v {
        vino_packet::v1::Packet::Success(success) => match success {
          vino_packet::v1::Serialized::MessagePack(bytes) => MessageTransport::Success(Serialized::MessagePack(bytes)),
          #[cfg(feature = "raw")]
          vino_packet::v1::Serialized::Struct(v) => MessageTransport::Success(Serialized::Struct(v)),
          #[cfg(not(feature = "raw"))]
          vino_packet::v1::Serialized::Struct(v) => MessageTransport::success(&v),
          #[cfg(feature = "json")]
          vino_packet::v1::Serialized::Json(v) => MessageTransport::Success(Serialized::Json(v)),
          #[cfg(not(feature = "json"))]
          vino_packet::v1::Serialized::Json(v) => MessageTransport::success(&v),
        },
        vino_packet::v1::Packet::Failure(failure) => match failure {
          vino_packet::v1::Failure::Invalid => MessageTransport::Failure(Failure::Invalid),
          vino_packet::v1::Failure::Exception(v) => MessageTransport::Failure(Failure::Exception(v)),
          vino_packet::v1::Failure::Error(v) => MessageTransport::Failure(Failure::Error(v)),
        },
        vino_packet::v1::Packet::Signal(signal) => match signal {
          vino_packet::v1::Signal::Done => MessageTransport::Signal(MessageSignal::Done),
          vino_packet::v1::Signal::OpenBracket => todo!(),
          vino_packet::v1::Signal::CloseBracket => todo!(),
          vino_packet::v1::Signal::Status(v) => MessageTransport::Signal(MessageSignal::Status(v)),
        },
      },
    }
  }
}

impl From<MessageTransport> for v1::Packet {
  fn from(output: MessageTransport) -> v1::Packet {
    match output {
      MessageTransport::Success(success) => match success {
        Serialized::MessagePack(v) => v1::Packet::Success(v1::Serialized::MessagePack(v)),
        #[cfg(feature = "raw")]
        Serialized::Struct(v) => v1::Packet::Success(v1::Serialized::Struct(v)),
        #[cfg(feature = "json")]
        Serialized::Json(v) => v1::Packet::Success(v1::Serialized::Json(v)),
      },
      MessageTransport::Failure(failure) => match failure {
        Failure::Invalid => v1::Packet::Failure(v1::Failure::Invalid),
        Failure::Exception(m) => v1::Packet::Failure(v1::Failure::Exception(m)),
        Failure::Error(m) => v1::Packet::Failure(v1::Failure::Error(m)),
      },
      MessageTransport::Signal(signal) => match signal {
        MessageSignal::Done => v1::Packet::Signal(v1::Signal::Done),
        MessageSignal::OpenBracket => v1::Packet::Signal(v1::Signal::OpenBracket),
        MessageSignal::CloseBracket => v1::Packet::Signal(v1::Signal::CloseBracket),
        MessageSignal::Status(v) => v1::Packet::Signal(v1::Signal::Status(v)),
      },
    }
  }
}

impl From<MessageTransport> for Packet {
  fn from(output: MessageTransport) -> Packet {
    Packet::V1(output.into())
  }
}
impl Display for MessageTransport {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let MessageTransport::Signal(signal) = self {
      return write!(f, "Signal({})", signal);
    }
    write!(
      f,
      "{}",
      match self {
        MessageTransport::Failure(v) => v.to_string(),
        MessageTransport::Signal(v) => v.to_string(),
        MessageTransport::Success(v) => v.to_string(),
      }
    )
  }
}

impl Display for MessageSignal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "{}",
      match self {
        MessageSignal::Done => "Done",
        MessageSignal::OpenBracket => "OpenBracket",
        MessageSignal::CloseBracket => "CloseBracket",
        MessageSignal::Status(_) => "Status",
      }
    ))
  }
}
impl Display for Failure {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Failure::Invalid => f.write_str("Invalid"),
      Failure::Exception(v) => f.write_fmt(format_args!("Exception: {}", v)),
      Failure::Error(v) => f.write_fmt(format_args!("Exception: {}", v)),
    }
  }
}
impl Display for Serialized {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "{}",
      match self {
        Serialized::MessagePack(_) => "MessagePack",
        #[cfg(feature = "raw")]
        Serialized::Struct(_) => "Success",
        #[cfg(feature = "json")]
        Serialized::Json(_) => "JSON",
      }
    ))
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  #[test_log::test]
  #[cfg(feature = "json")]
  fn serializes_done() -> Result<()> {
    let close = MessageTransport::done();
    let value = close.as_json();
    println!("Value: {}", value);
    assert_eq!(value.to_string(), r#"{"signal":"Done","value":null}"#);
    Ok(())
  }

  #[test_log::test]
  fn messagepack_rt() -> Result<()> {
    // let mut original = TransportMap::new();
    let mut payload = MessageTransport::success(&false);
    println!("payload: {:?}", payload);
    payload.to_messagepack();
    let result: bool = payload.deserialize()?;
    assert!(!result);
    Ok(())
  }
}
