use std::error;

use tokio::sync::mpsc::error::SendError;
use vino_packet::PacketWrapper;

use crate::error::Error;

#[must_use]

/// The error type that components can return on failures.
#[derive(Debug)]
pub struct NativeComponentError {
  msg: String,
}

impl error::Error for NativeComponentError {}

impl std::fmt::Display for NativeComponentError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.msg)
  }
}

impl NativeComponentError {
  /// Constructor for [NativeComponentError].
  pub fn new<T: AsRef<str>>(msg: T) -> Self {
    Self {
      msg: msg.as_ref().to_owned(),
    }
  }

  /// Utility function to create a [Box<NativeComponentError>].
  pub fn boxed<T: AsRef<str>>(msg: T) -> Box<Self> {
    Box::new(Self {
      msg: msg.as_ref().to_owned(),
    })
  }
}

impl From<&'static str> for NativeComponentError {
  fn from(e: &'static str) -> Self {
    NativeComponentError::new(e)
  }
}

impl From<String> for NativeComponentError {
  fn from(e: String) -> Self {
    NativeComponentError::new(e)
  }
}

impl From<SendError<PacketWrapper>> for Error {
  fn from(e: SendError<PacketWrapper>) -> Self {
    Self::ChannelError(e.to_string())
  }
}

impl From<vino_packet::error::Error> for Error {
  fn from(e: vino_packet::error::Error) -> Self {
    Self::Conversion(e.to_string())
  }
}

impl From<Error> for NativeComponentError {
  fn from(e: Error) -> Self {
    Self::new(e.to_string())
  }
}
