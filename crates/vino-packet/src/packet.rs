use serde::de::DeserializeOwned;
use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::{
  json,
  messagepack,
  raw,
};

use crate::error::DeserializationError;
pub use crate::{
  v0,
  v1,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// The output payload that component's push out of output ports.
pub enum Packet {
  /// Version 0 of the payload format (unstable).
  #[serde(rename = "0")]
  V0(v0::Payload),
  /// Version 1 of the payload format (alpha).
  #[serde(rename = "0")]
  V1(v1::Payload),
}

impl Packet {
  /// Try to deserialize a [Packet] into the target type.
  pub fn try_into<T: DeserializeOwned>(self) -> Result<T, DeserializationError> {
    match self {
      Packet::V0(v) => match v {
        v0::Payload::Invalid => Err(DeserializationError::Invalid),
        v0::Payload::Exception(v) => Err(DeserializationError::Exception(v)),
        v0::Payload::Error(v) => Err(DeserializationError::Error(v)),
        v0::Payload::MessagePack(buf) => {
          messagepack::deserialize::<T>(&buf).map_err(DeserializationError::DeserializationError)
        }
        v0::Payload::Success(v) => {
          raw::deserialize::<T>(v).map_err(DeserializationError::DeserializationError)
        }
        v0::Payload::Json(v) => {
          json::deserialize::<T>(v.as_str()).map_err(DeserializationError::DeserializationError)
        }
        v0::Payload::Done => Err(DeserializationError::InternalError),
        v0::Payload::OpenBracket => Err(DeserializationError::InternalError),
        v0::Payload::CloseBracket => Err(DeserializationError::InternalError),
      },
      Packet::V1(v) => {
        match v {
          v1::Payload::Success(v) => match v {
            v1::Success::MessagePack(buf) => messagepack::deserialize::<T>(&buf)
              .map_err(DeserializationError::DeserializationError),
            v1::Success::Success(val) => {
              raw::deserialize::<T>(val).map_err(DeserializationError::DeserializationError)
            }
            v1::Success::Json(val) => json::deserialize::<T>(val.as_str())
              .map_err(DeserializationError::DeserializationError),
          },
          v1::Payload::Failure(v) => match v {
            v1::Failure::Invalid => Err(DeserializationError::Invalid),
            v1::Failure::Exception(msg) => Err(DeserializationError::Exception(msg)),
            v1::Failure::Error(msg) => Err(DeserializationError::Error(msg)),
          },
          v1::Payload::Signal(_) => Err(DeserializationError::InternalError),
        }
      }
    }
  }

  #[must_use]
  /// Does the [Packet] signify the originating job is completed?.
  pub fn is_done(&self) -> bool {
    match self {
      Packet::V0(v) => matches!(v, v0::Payload::Done | v0::Payload::Error(_)),
      Packet::V1(v) => matches!(
        v,
        v1::Payload::Signal(v1::Signal::Done) | v1::Payload::Failure(v1::Failure::Error(_))
      ),
    }
  }
}

impl From<&Vec<u8>> for Packet {
  fn from(buf: &Vec<u8>) -> Self {
    match messagepack::deserialize::<Packet>(buf) {
      Ok(packet) => packet,
      Err(e) => Packet::V0(v0::Payload::Error(format!(
        "Error deserializing packet: {}",
        e
      ))),
    }
  }
}

impl From<&[u8]> for Packet {
  fn from(buf: &[u8]) -> Self {
    match messagepack::deserialize::<Packet>(buf) {
      Ok(packet) => packet,
      Err(e) => Packet::V0(v0::Payload::Error(format!(
        "Error deserializing packet: {}",
        e
      ))),
    }
  }
}

#[derive(Debug, Clone)]
/// A [PacketWrapper] is a wrapper around a [Packet] with the port name embedded.
pub struct PacketWrapper {
  /// The port name.
  pub port: String,
  /// The wrapped packet [Packet].
  pub payload: Packet,
}
