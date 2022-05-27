//! Vino's WebAssembly provider implementations. This module
//!  is only active with the feature `wasm` enabled.
//!
#![allow(unsafe_code, missing_debug_implementations, missing_copy_implementations)]

use vino_wapc::guest::wasm::runtime::*;
use vino_wapc::OutputSignal;

/// Module that encapsulates the wasm version of a provider output stream.
pub(crate) mod provider_output;

/// Module for log-like functionality in WASM.
pub mod log;

/// The WebAssembly implementation of a Port Sender.
// mod port_sender;
// pub use port_sender::PortSender;
pub use provider_output::{PortOutput, ProviderOutput};
use vino_codec::messagepack::serialize;
use vino_packet::Packet;

use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

#[doc(hidden)]
// This is meant for code generation consumers.
pub type CallResult = Result<Vec<u8>>;

fn serialize_payload(id: u32, packet: Option<Packet>) -> Result<Vec<u8>> {
  let bytes = match packet {
    Some(packet) => {
      let bytes = serialize(&packet)?;
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
pub fn port_send(port_name: &str, id: u32, packet: Packet) -> Result<()> {
  let bytes = serialize_payload(id, Some(packet))?;
  host_call("0", port_name, OutputSignal::Output.as_str(), &bytes).map_err(Error::Protocol)?;
  Ok(())
}

/// Send a [Packet] out the named port and immediately close it.
pub fn port_send_close(port_name: &str, id: u32, packet: Packet) -> Result<()> {
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
