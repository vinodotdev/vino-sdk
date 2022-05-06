//! Vino's WebAssembly provider implementations. This module
//!  is only active with the feature `wasm` enabled.
//!
#![allow(unsafe_code, missing_debug_implementations, missing_copy_implementations)]

use std::collections::HashMap;

use vino_wapc::guest::wasm::runtime::*;
use vino_wapc::OutputSignal;

/// Module that encapsulates the wasm version of a provider output stream.
mod provider_output;

/// Module for log-like functionality in WASM.
pub mod log;

use vino_codec::messagepack::{deserialize, serialize};

/// Errors for WebAssembly providers.
pub mod error;
/// The WebAssembly implementation of a Port Sender.
mod port_sender;
pub use error::Error;
pub use port_sender::PortSender;
use vino_packet::{v0, Packet};

type Result<T> = std::result::Result<T, Error>;

#[doc(hidden)]
// This is meant for code generation consumers.
pub type CallResult = Result<Vec<u8>>;

/// Common imports for WebAssembly providers and components.
pub mod prelude {
  pub use vino_entity::Entity;
  pub use vino_transport::error::TransportError;
  pub use vino_transport::{MessageTransport, TransportMap, TransportWrapper};
  pub use vino_types::*;
  pub use vino_wapc::guest::wasm::exports as wapc;
  pub use vino_wapc::*;

  pub use super::error::ComponentError;
  pub use super::provider_output::{PortOutput, ProviderOutput};
  pub use super::{Error as WasmError, IncomingPayload, PortSender, WapcComponent};
  pub use crate::codec::messagepack::{deserialize, serialize};
  pub use crate::provider_link::ProviderLink;
  pub use crate::wasm::log;
}

fn serialize_payload(id: u32, packet: Option<v0::Payload>) -> Result<Vec<u8>> {
  let bytes = match packet {
    Some(packet) => {
      let bytes = serialize(&Packet::V0(packet))?;
      let mut payload = Vec::with_capacity(bytes.len() + 4);
      payload.extend_from_slice(&id.to_be_bytes());
      payload.extend(bytes.into_iter());
      payload
    }
    None => {
      let mut payload = Vec::with_capacity(4);
      payload.extend_from_slice(&id.to_be_bytes());
      payload
    }
  };
  Ok(bytes)
}

/// Send a [Packet] out the named port.
pub fn port_send(port_name: &str, id: u32, packet: v0::Payload) -> Result<()> {
  let bytes = serialize_payload(id, Some(packet))?;
  host_call("0", port_name, OutputSignal::Output.as_str(), &bytes).map_err(Error::Protocol)?;
  Ok(())
}

/// Send a [Packet] out the named port and immediately close it.
pub fn port_send_close(port_name: &str, id: u32, packet: v0::Payload) -> Result<()> {
  let bytes = serialize_payload(id, Some(packet))?;
  host_call("0", port_name, OutputSignal::OutputDone.as_str(), &bytes).map_err(Error::Protocol)?;
  Ok(())
}

/// Close the referenced port.
pub fn port_close(port_name: &str, id: u32) -> Result<()> {
  let bytes = serialize_payload(id, None)?;
  host_call("0", port_name, OutputSignal::Done.as_str(), &bytes).map_err(Error::Protocol)?;
  Ok(())
}

/// A map of port name to payload message.
pub struct IncomingPayload {
  id: u32,
  encoded: HashMap<String, Vec<u8>>,
}

impl IncomingPayload {
  /// Decode MessagePack bytes into an [IncomingPayload].
  pub fn from_buffer(buffer: &[u8]) -> Result<Self> {
    let (id, input_encoded): (u32, HashMap<String, Vec<u8>>) = deserialize(buffer)?;

    Ok(Self {
      id,
      encoded: input_encoded,
    })
  }

  /// Get the transaction ID associated with this [IncomingPayload].
  #[must_use]
  pub fn id(&self) -> u32 {
    self.id
  }

  /// Get the contained bytes for the specified port.
  pub fn get(&self, port: &str) -> Result<&Vec<u8>> {
    self
      .encoded
      .get(port)
      .ok_or_else(|| Error::MissingInput(port.to_owned()))
  }
}

/// The trait for WaPC-based WebAssembly components.
pub trait WapcComponent {
  /// This method takes an incoming payload and is expected to return nothing.
  /// Vino expects execution output to be sent over the WaPC protocol via host calls.
  fn execute(
    &self,
    payload: IncomingPayload,
  ) -> vino_wapc::guest::wasm::BoxedFuture<std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>>;
}
