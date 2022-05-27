/// Errors originating from WASM components.
#[derive(Debug)]
pub enum Error {
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
      Error::HostError(v) => write!(f, "Error executing host call: {}", v),
      Error::Async => write!(f, "Async runtime error"),
      Error::DispatcherNotSet => write!(f, "Dispatcher not set before host call"),
    }
  }
}

impl std::error::Error for Error {}
