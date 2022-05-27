use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::Error;
// use vino_codec::messagepack;
#[cfg(feature = "v0")]
pub use crate::v0;
#[cfg(feature = "v1")]
pub use crate::v1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// The output payload that component's push out of output ports.
pub enum Packet {
  /// Version 0 of the payload format (unstable).
  #[serde(rename = "v0")]
  #[cfg(feature = "v0")]
  V0(v0::Payload),
  /// Version 1 of the payload format (alpha).
  #[serde(rename = "v1")]
  #[cfg(feature = "v1")]
  V1(v1::Packet),
}

impl Packet {
  #[must_use]
  /// Does the [Packet] signify the originating job is completed?.
  #[cfg(any(feature = "v1", feature = "v0"))]
  pub fn is_done(&self) -> bool {
    match self {
      #[cfg(feature = "v0")]
      Packet::V0(v) => matches!(v, v0::Payload::Done | v0::Payload::Error(_)),
      #[cfg(feature = "v1")]
      Packet::V1(v) => matches!(
        v,
        v1::Packet::Signal(v1::Signal::Done) | v1::Packet::Failure(v1::Failure::Error(_))
      ),
    }
  }

  #[must_use]
  /// Does the [Packet] signify the originating job is completed?.
  #[cfg(any(feature = "v1", feature = "v0"))]
  pub fn is_signal(&self) -> bool {
    match self {
      #[cfg(feature = "v0")]
      Packet::V0(v) => matches!(v, v0::Payload::Done),
      #[cfg(feature = "v1")]
      Packet::V1(v) => matches!(v, v1::Packet::Signal(_)),
    }
  }

  /// Convert a messagepack encoded payload into a [Packet]
  pub fn from_messagepack(bytes: &[u8]) -> Self {
    match vino_codec::messagepack::deserialize::<Packet>(bytes) {
      Ok(packet) => packet,
      Err(e) => Packet::V0(v0::Payload::Error(format!("Error deserializing packet: {}", e))),
    }
  }

  /// Converts the [MessageTransport] into a messagepack-compatible transport.
  pub fn to_messagepack(&mut self) {
    match &self {
      Packet::V0(_) => unimplemented!("Converted a V0 packet to messagepack is not implemented via this function."),
      Packet::V1(v) => match v {
        v1::Packet::Success(v) => match v {
          v1::Serialized::MessagePack(_) => { /* nothing */ }
          v1::Serialized::Struct(v) => {
            *self = v1::Packet::Success(v1::Serialized::MessagePack(
              vino_codec::messagepack::serialize(&v).unwrap(),
            ))
            .into()
          }
          v1::Serialized::Json(json) => {
            *self = v1::Packet::Success(v1::Serialized::Json(vino_codec::json::serialize(&json).unwrap())).into()
          }
        },
        _ => { /* nothing */ }
      },
    };
  }

  /// Try to deserialize a [Packet] into the target type
  pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, Error> {
    try_from(self)
  }
}

fn try_from<T: DeserializeOwned>(value: Packet) -> Result<T, Error> {
  match value {
    Packet::V0(p) => p.deserialize(),
    Packet::V1(p) => p.deserialize(),
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A [PacketWrapper] is a wrapper around a [Packet] with the port name embedded.
pub struct PacketWrapper {
  /// The port name.
  pub port: String,
  /// The wrapped packet [Packet].
  pub payload: Packet,
}

impl PacketWrapper {
  /// The reserved port name for status messages emitted from a component.
  pub const STATUS: &'static str = "<status>";

  /// Create a new [PacketWrapper] by setting the packet directly.
  pub fn new_raw(port: impl AsRef<str>, packet: Packet) -> Self {
    PacketWrapper {
      port: port.as_ref().to_owned(),
      payload: packet,
    }
  }

  /// Create a special [PacketWrapper] that contains the end state of a component's run.
  pub fn state(packet: Packet) -> Self {
    PacketWrapper {
      port: Self::STATUS.to_owned(),
      payload: packet,
    }
  }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
/// A map of port names to packets.
pub struct PacketMap {
  inner: HashMap<String, Packet>,
}

impl PacketMap {
  /// Constructor for a new [PacketMap]
  #[must_use]
  pub fn new(map: HashMap<String, Packet>) -> Self {
    Self { inner: map }
  }

  /// Remove a [Packet] from a [PacketMap]
  #[must_use]
  pub fn remove(&mut self, port: &str) -> Option<Packet> {
    self.inner.remove(port)
  }

  /// Insert a [Packet] into a [PacketMap]
  pub fn insert<T: AsRef<str>>(&mut self, port: T, value: impl Serialize) {
    self
      .inner
      .insert(port.as_ref().to_owned(), v1::Packet::success(&value).into());
  }
}

impl IntoIterator for PacketMap {
  type Item = (String, Packet);
  type IntoIter = std::collections::hash_map::IntoIter<String, Packet>;

  fn into_iter(self) -> Self::IntoIter {
    self.inner.into_iter()
  }
}

impl<K: AsRef<str>, V: Serialize> From<Vec<(K, V)>> for PacketMap {
  fn from(list: Vec<(K, V)>) -> Self {
    let mut map = PacketMap::default();
    for (k, v) in list {
      map.insert(k, v);
    }
    map
  }
}

impl<V, const N: usize> From<[(&str, V); N]> for PacketMap
where
  V: Serialize + Sized,
{
  fn from(list: [(&str, V); N]) -> Self {
    let mut map = PacketMap::default();
    for (k, v) in list {
      map.insert(k, v);
    }
    map
  }
}
