/// Module for native provider errors.
pub mod error;
use async_trait::async_trait;
/// Module for native ports.
// pub mod port_sender;
/// Module for making transport streams simpler to use.
pub mod provider_output;

/// The JobResult for native components.
pub type JobResult = Result<(), NativeComponentError>;

#[async_trait]
/// Trait used by auto-generated provider components. You shouldn't need to implement this if you are using Vino's code generator.
pub trait NativeComponent {
  /// The provider state passed to every component's execution.
  type Context: Send + Sync;
  /// The wrapper method that is called to execute the component's job.
  async fn execute(
    &self,
    context: Self::Context,
    data: TransportMap,
  ) -> Result<TransportStream, Box<NativeComponentError>>;
}

// pub use port_sender::PortSender;
pub use provider_output::{PortOutput, ProviderOutput};
pub use vino_entity as entity;
use vino_transport::{TransportMap, TransportStream};

use self::error::NativeComponentError;
use crate::error::Error;

#[doc(hidden)]
#[async_trait]
pub trait Dispatch {
  type Context: Send + Sync;
  async fn dispatch(
    op: &str,
    context: Self::Context,
    data: TransportMap,
  ) -> Result<TransportStream, Box<NativeComponentError>>;
}
