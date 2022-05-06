#[doc(hidden)]
// This is meant for code generation consumers.
pub type CallResult = Result<TransportStream, Box<NativeComponentError>>;

#[async_trait]
pub trait Dispatcher {
  type Context: Send + Sync;
  async fn dispatch(
    op: &str,
    context: Self::Context,
    data: TransportMap,
  ) -> Result<TransportStream, Box<NativeComponentError>>;
}
