pub mod error;
pub mod exports;
pub mod imports;
pub mod runtime;

use error::Error;
use wasm_rs_async_executor::single_threaded as executor;

/// Utility type for a Pin<Box<Future<T>>>
pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;
/// Utility type for a Box<dyn std::error::Error + Send + Sync>
pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

pub trait Dispatcher {
  /// Dispatch method
  fn dispatch(&self, id: i32, op: &'static str, payload: &'static [u8]) -> BoxedFuture<Result<Vec<u8>, BoxedError>>;
}
