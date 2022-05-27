pub mod packet {
  // pub use vino_packet::*;
  pub use vino_packet::{PacketMap, PacketWrapper};
  pub mod v1 {
    pub use vino_packet::v1::{Packet, PacketMap};
  }
}

pub mod sdk {
  #[cfg(target_arch = "wasm32")]
  pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;
  #[cfg(not(target_arch = "wasm32"))]
  pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;
  pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

  pub use futures::{Stream, StreamExt};
  pub use vino_provider::{PortOutput, ProviderLink, ProviderOutput};
  pub use vino_transport::{Invocation, MessageTransport, TransportWrapper};
  pub use wasmflow_boundary::IncomingPayload;
  pub use wasmflow_traits::{Component, IntoInputs, PortChannel, Writable};

  pub mod payload {
    #[cfg(not(target_arch = "wasm32"))]
    pub use wasmflow_boundary::native::v1::from_invocation;
    #[cfg(target_arch = "wasm32")]
    pub use wasmflow_boundary::wasm::from_buffer;
  }

  pub mod stateful {
    #[cfg(not(target_arch = "wasm32"))]
    pub use vino_wapc::guest::stateful::native::Dispatcher as NativeDispatcher;
    #[cfg(target_arch = "wasm32")]
    pub use vino_wapc::guest::stateful::wasm::Dispatcher as WasmDispatcher;
    pub use vino_wapc::guest::stateful::BatchedJobExecutor;
    pub use wasmflow_traits::stateful::BatchedComponent;
  }

  pub mod ephemeral {
    #[cfg(not(target_arch = "wasm32"))]
    pub use vino_wapc::guest::ephemeral::native::Dispatcher as NativeDispatcher;
    #[cfg(target_arch = "wasm32")]
    pub use vino_wapc::guest::ephemeral::wasm::Dispatcher as WasmDispatcher;
    pub use vino_wapc::guest::ephemeral::BatchedJobExecutor;
    pub use wasmflow_traits::ephemeral::BatchedComponent;
  }

  #[cfg(target_arch = "wasm32")]
  pub mod wasm {
    pub use wasmflow_boundary::wasm::EncodedMap;
    pub mod runtime {
      pub use vino_wapc::guest::wasm::runtime::register_dispatcher;
    }
    pub use vino_provider::wasm::{port_send, port_send_close, PortOutput, ProviderOutput};
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub mod native {
    // pub use vino_provider::native::port_sender::{PortChannel, PortSender};
    pub use vino_provider::native::provider_output::{PortOutput, ProviderOutput};
  }
}

pub mod error {
  pub use crate::sdk::BoxedError;

  #[derive(Debug)]
  pub enum Error {
    /// An input the component expects was not found.
    MissingInput(String),

    /// An error from an upstream module.
    Upstream(Box<dyn std::error::Error + Send + Sync>),

    /// Error sending packet to output port.
    SendError(String),

    /// The requested component was not found in this module.
    ComponentNotFound(String, String),

    /// An error resulting from deserializing or serializing a payload.
    CodecError(String),
    /// culling line
    /// An error returned from the WaPC host, the system running the WebAssembly module.
    HostError(String),

    /// Async runtime failure.
    Async,

    /// Dispatcher not set before guest call
    DispatcherNotSet,
  }

  #[derive(Debug)]
  /// Error originating from a component task.
  pub struct ComponentError(String);

  impl ComponentError {
    /// Constructor for a [ComponentError].
    pub fn new<T: std::fmt::Display>(message: T) -> Self {
      Self(message.to_string())
    }
  }

  impl std::error::Error for ComponentError {}

  impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{}", self.0)
    }
  }

  impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        Error::ComponentNotFound(v, valid) => write!(f, "Component '{}' not found. Valid components are: {}", v, valid),
        Error::Upstream(v) => write!(f, "{}", v),
        Error::MissingInput(v) => write!(f, "Missing input for port '{}'", v),
        Error::SendError(port) => write!(f, "Error sending packet to output port '{}'", port),
        Error::CodecError(v) => write!(f, "{}", v),
        Error::HostError(v) => write!(f, "Error executing host call: {}", v),
        Error::Async => write!(f, "Async runtime error"),
        Error::DispatcherNotSet => write!(f, "Dispatcher not set before host call"),
      }
    }
  }

  impl std::error::Error for Error {}

  impl From<vino_packet::error::Error> for Error {
    fn from(e: vino_packet::error::Error) -> Self {
      Error::Upstream(Box::new(e))
    }
  }

  impl From<vino_codec::Error> for Error {
    fn from(e: vino_codec::Error) -> Self {
      Error::CodecError(e.to_string())
    }
  }

  impl From<BoxedError> for Error {
    fn from(e: BoxedError) -> Self {
      Error::Upstream(e)
    }
  }
}

pub mod codec {
  #[cfg(not(target_arch = "wasm32"))]
  pub use vino_codec::json;
  pub use vino_codec::messagepack;
}

pub mod provider {
  pub mod error {}
  // pub use vino_provider::*;
}

pub mod types {
  pub use vino_transport::{BoxedTransportStream, TransportStream};
  pub use vino_types::*;
  pub use wasmflow_streams::PacketStream;
}

#[macro_use]
#[allow(unreachable_pub)]
pub use vino_wapc::console_log;
