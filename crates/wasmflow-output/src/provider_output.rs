use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use tokio_stream::StreamExt;
use wasmflow_packet::{Packet, PacketWrapper};
use vino_transport::TransportStream;
use wasmflow_streams::PacketStream;

/// A wrapper object for the packets returned from the provider call.
#[allow(missing_debug_implementations)]
pub struct ProviderOutput {
  packets: PacketStream,
}

impl ProviderOutput {
  /// Initialize a [ProviderOutput] with a [Vec<TransportWrapper>]
  #[must_use]
  pub fn new<S>(packets: S) -> Self
  where
    S: tokio_stream::Stream<Item = PacketWrapper> + Unpin + Send + Sync + 'static,
    S: Sized,
  {
    Self {
      packets: PacketStream::new(Box::new(packets)),
    }
  }

  /// Initialize a [ProviderOutput] with a [TransportStream]
  #[must_use]
  pub fn new_from_ts(packets: TransportStream) -> Self {
    Self {
      packets: PacketStream::new(Box::new(
        packets.map(|a| PacketWrapper::new_raw(a.port, a.payload.into())),
      )),
    }
  }
  /// Get a list of [MessageTransport] from the specified port.
  pub async fn drain_port(&mut self, port: &str) -> Vec<Packet> {
    self.packets.take_port(port).await.unwrap()
  }
}

/// Iterator wrapper for a list of [MessageTransport]s
#[must_use]
pub struct PortOutput<T: DeserializeOwned> {
  name: String,
  iter: Box<dyn Iterator<Item = Packet>>,
  _data: PhantomData<T>,
}

impl<T: DeserializeOwned> std::fmt::Debug for PortOutput<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PortOutput").field("iter", &self.name).finish()
  }
}

impl<T: DeserializeOwned> PortOutput<T> {
  /// Constructor for [PortOutput] that takes a list of [MessageTransport]
  pub fn new(name: String, packets: Vec<Packet>) -> Self {
    Self {
      name,
      iter: Box::new(packets.into_iter()),
      _data: PhantomData,
    }
  }

  /// Grab the next value and deserialize it in one method.
  pub fn deserialize_next(&mut self) -> Result<T, crate::error::Error> {
    match self.iter.next() {
      Some(val) => Ok(
        val
          .deserialize()
          .map_err(|e| crate::error::Error::Codec(e.to_string()))?,
      ),
      None => Err(crate::error::Error::EndOfOutput(self.name.clone())),
    }
  }
}

impl<T: DeserializeOwned> Iterator for PortOutput<T> {
  type Item = Packet;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}
