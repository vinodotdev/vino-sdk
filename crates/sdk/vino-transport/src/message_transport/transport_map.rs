use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Display;

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

use super::transport_json::TransportJson;
use crate::error::TransportError;
use crate::{
  Error,
  Failure,
  MessageTransport,
  Success,
};
pub(crate) type Result<T> = std::result::Result<T, TransportError>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[must_use]
/// A wrapper for a map of [String]s to [MessageTransport]
pub struct TransportMap(HashMap<String, MessageTransport>);

impl TransportMap {
  /// Constructor for [TransportMap] with initial map
  pub fn with_map(map: HashMap<String, MessageTransport>) -> Self {
    Self(map)
  }

  /// Constructor for an empty [TransportMap]
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  /// Deserialize a JSON Object into a [TransportMap]
  pub fn from_json_str(json: &str) -> Result<Self> {
    if json.trim() == "" {
      Ok(TransportMap::new())
    } else {
      let json: HashMap<String, TransportJson> = json::deserialize(json).map_err(de_err)?;
      Ok(TransportMap::with_map(
        json
          .into_iter()
          .map(|(name, val)| (name, val.into()))
          .collect(),
      ))
    }
  }

  /// Turn a list of "field=value" strings into a [TransportMap] of [MessageTransport::Json] items.
  pub fn from_kv_json(values: &[String]) -> Result<Self> {
    let mut payload = TransportMap::new();
    for input in values {
      match input.split_once("=") {
        Some((name, value)) => {
          debug!("PORT:'{}', VALUE:'{}'", name, value);
          payload.insert(
            name,
            MessageTransport::Success(Success::Json(value.to_owned())),
          );
        }
        None => {
          return Err(Error::DeserializationError(format!(
            "Invalid port=value pair: '{}'",
            input
          )))
        }
      }
    }
    Ok(payload)
  }

  /// Insert a [MessageTransport] by port name
  pub fn insert<T: AsRef<str>>(
    &mut self,
    port: T,
    msg: MessageTransport,
  ) -> Option<MessageTransport> {
    self.0.insert(port.as_ref().to_owned(), msg)
  }

  /// Get a reference to the [MessageTransport] behind the passed port
  #[must_use]
  pub fn get(&self, port: &str) -> Option<&MessageTransport> {
    self.0.get(port)
  }

  /// Remove a key from the held map and attempt to deserialize it into the destination type
  pub fn consume<T: DeserializeOwned>(&mut self, key: &str) -> Result<T> {
    let v = self.0.remove(key).ok_or_else(|| {
      Error::DeserializationError(format!("TransportMap does not have field '{}'", key))
    })?;
    let e = Err(Error::DeserializationError(format!(
      "Payload could not be converted to destination type. Payload was: {:?}",
      v
    )));
    match v {
      MessageTransport::Success(success) => match success {
        Success::MessagePack(bytes) => messagepack::deserialize(&bytes).map_err(de_err),
        Success::Serialized(v) => raw::deserialize(v).map_err(de_err),
        Success::Json(v) => json::deserialize(&v).map_err(de_err),
      },
      MessageTransport::Failure(_) => e,
      MessageTransport::Signal(_) => e,
    }
  }

  /// Remove a key from the held map and return the raw [MessageTransport].
  pub fn consume_raw(&mut self, key: &str) -> Result<MessageTransport> {
    self.0.remove(key).ok_or_else(|| {
      Error::DeserializationError(format!("TransportMap does not have field '{}'", key))
    })
  }

  /// Transpose any ports named "output" to "input". This is for a better user experience when
  /// trying to pipe components together without a full runtime. This should never be done
  /// without also providing a way to turn it off.
  #[doc(hidden)]
  pub fn transpose_output_name(&mut self) {
    let output = self.0.remove("output");
    if let Some(msg) = output {
      debug!("Transposing [output] to [input]");
      self.0.insert("input".to_owned(), msg);
    }
  }

  /// Returns true if any of the held messages is an error or exception type.
  #[must_use]
  pub fn has_error(&self) -> bool {
    for msg in self.0.values() {
      if msg.is_err() {
        return true;
      }
    }
    false
  }

  /// Returns an error if the transport is holding one, otherwise returns None.
  #[must_use]
  pub fn take_error(self) -> Option<MessageTransport> {
    for (_, v) in self.0 {
      if v.is_err() {
        return Some(v);
      }
    }
    None
  }

  /// Returns the inner [HashMap]
  #[must_use]
  pub fn into_inner(self) -> HashMap<String, MessageTransport> {
    self.0
  }

  /// Attempts to normalize the [TransportMap] into messagepacked bytes
  /// by serializing success formats or throwing an error.
  pub fn try_into_messagepack_bytes(self) -> Result<HashMap<String, Vec<u8>>> {
    let mut map = HashMap::new();
    for (k, v) in self.0 {
      let bytes = match v {
        MessageTransport::Success(success) => match success {
          Success::MessagePack(bytes) => Ok(bytes),
          Success::Serialized(v) => {
            let bytes = messagepack::serialize(&v).map_err(ser_err)?;
            Ok(bytes)
          }
          Success::Json(v) => {
            let value: serde_value::Value = json::deserialize(&v).map_err(de_err)?;
            let bytes = messagepack::serialize(&value).map_err(ser_err)?;
            Ok(bytes)
          }
        },
        MessageTransport::Failure(failure) => match failure {
          Failure::Invalid => Err(Error::SerializationError(
            "Refusing to serialize an invalid payload".to_owned(),
          )),
          Failure::Exception(e) => Err(Error::SerializationError(format!(
            "Exceptions need to be processed by a runtime, not sent to components. Error was: {}",
            e
          ))),
          Failure::Error(e) => Err(Error::SerializationError(format!(
            "Errors need to be processed by a runtime, not sent to components. Error was: {}",
            e
          ))),
        },
        MessageTransport::Signal(_) => Err(Error::SerializationError(
          "Signal messages need to be processed by a runtime, not sent to components.".to_owned(),
        )),
      }?;
      map.insert(k, bytes);
    }
    Ok(map)
  }

  /// Merge another [TransportMap] into the calling map.
  pub fn merge(&mut self, map: TransportMap) {
    for (k, v) in map.into_inner() {
      self.insert(k, v);
    }
  }
}

impl<K, V> TryFrom<&HashMap<K, V>> for TransportMap
where
  K: AsRef<str> + Send + Sync,
  V: Serialize + Sync,
{
  type Error = TransportError;

  fn try_from(v: &HashMap<K, V>) -> Result<Self> {
    let serialized_data: HashMap<String, MessageTransport> = v
      .iter()
      .map(|(k, v)| {
        Ok((
          k.as_ref().to_owned(),
          MessageTransport::Success(Success::MessagePack(
            messagepack::serialize(&v).map_err(ser_err)?,
          )),
        ))
      })
      .filter_map(Result::ok)
      .collect();

    let payload = TransportMap::with_map(serialized_data);
    Ok(payload)
  }
}

impl<K: AsRef<str>, V: Serialize> From<Vec<(K, V)>> for TransportMap {
  fn from(list: Vec<(K, V)>) -> Self {
    let mut map = TransportMap::new();
    for (k, v) in list {
      map.insert(k.as_ref().to_owned(), MessageTransport::success(&v));
    }
    map
  }
}

impl<K: AsRef<str>, V: Serialize> From<HashMap<K, V>> for TransportMap {
  fn from(hashmap: HashMap<K, V>) -> Self {
    let mut map = TransportMap::new();
    for (k, v) in hashmap {
      map.insert(k.as_ref().to_owned(), MessageTransport::success(&v));
    }
    map
  }
}

fn ser_err<T: Display>(e: T) -> Error {
  Error::SerializationError(e.to_string())
}

fn de_err<T: Display>(e: T) -> Error {
  Error::DeserializationError(e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::MessageTransport;

  #[test_env_log::test]
  fn test_merge() -> Result<()> {
    let mut map1 = TransportMap::new();
    map1.insert("first", MessageTransport::success(&"first-val"));
    let mut map2 = TransportMap::new();
    map2.insert("second", MessageTransport::success(&"second-val"));
    map1.merge(map2);
    let val1: String = map1.consume("first")?;
    assert_eq!(val1, "first-val");
    let val2: String = map1.consume("second")?;
    assert_eq!(val2, "second-val");

    Ok(())
  }
}