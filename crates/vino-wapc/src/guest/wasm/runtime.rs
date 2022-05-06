use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};

use tokio::sync::oneshot;

use super::error::Error;
use super::imports::*;
use super::{BoxedError, BoxedFuture, Dispatcher};
thread_local! {
  pub(super) static ASYNC_HOST_CALLS: UnsafeCell<HashMap<i32,oneshot::Sender<i32>>>  = UnsafeCell::new(HashMap::new());
}

thread_local! {
  pub(super) static DISPATCHER: UnsafeCell<Option<Box<dyn Dispatcher + Sync + Send>>>  = UnsafeCell::new(None);
}
type CallResult = Result<Vec<u8>, BoxedError>;

static CALL_NUM: AtomicI32 = AtomicI32::new(0);

pub fn exhaust_tasks() {
  wasm_rs_async_executor::single_threaded::run_while(Some(Box::new(move || {
    let num_in_flight = ASYNC_HOST_CALLS.with(|cell| {
      #[allow(unsafe_code)]
      unsafe {
        let map: &HashMap<i32, oneshot::Sender<i32>> = &*cell.get();
        map.len()
      }
    });
    num_in_flight == 0
  })));
}

pub fn register_dispatcher(dispatcher: Box<dyn Dispatcher + Send + Sync>) {
  #[allow(unsafe_code)]
  DISPATCHER.with(|cell| {
    let option: &mut Option<Box<dyn Dispatcher + Sync + Send>> = unsafe { &mut *cell.get() };
    option.replace(dispatcher);
  });
}

pub fn get_dispatcher() -> Result<&'static (dyn Dispatcher + Sync + Send), Error> {
  #[allow(unsafe_code)]
  DISPATCHER.with(|cell| {
    let option: &mut Option<Box<dyn Dispatcher + Sync + Send>> = unsafe { &mut *cell.get() };
    option.as_deref().ok_or_else(|| Error::Async)
  })
}

/// The function through which all host calls take place.
pub fn host_call(binding: &str, ns: &str, op: &str, msg: &[u8]) -> CallResult {
  let id = CALL_NUM.fetch_add(1, Ordering::SeqCst);

  #[allow(unsafe_code)]
  let callresult = unsafe {
    __host_call(
      id,
      binding.as_ptr(),
      binding.len(),
      ns.as_ptr(),
      ns.len(),
      op.as_ptr(),
      op.len(),
      msg.as_ptr(),
      msg.len(),
    )
  };

  if callresult != 1 {
    // call was not successful
    #[allow(unsafe_code)]
    let len = unsafe { __host_error_len(id) };
    let buf = Vec::with_capacity(len);
    let retptr = buf.as_ptr();
    #[allow(unsafe_code)]
    let _slice = unsafe {
      __host_error(id, retptr);
      std::slice::from_raw_parts(retptr, len)
    };
    Err(Box::new(super::Error::HostError(
      String::from_utf8_lossy(_slice).to_string(),
    )))
  } else {
    // call succeeded
    #[allow(unsafe_code)]
    let len = unsafe { __host_response_len(id) };
    let buf = Vec::with_capacity(len);
    let retptr = buf.as_ptr();
    #[allow(unsafe_code)]
    let slice = unsafe {
      __host_response(id, retptr);
      std::slice::from_raw_parts(retptr, len)
    };
    Ok(slice.to_vec())
  }
}

#[cold]
#[inline(never)]
/// Request a line to be printed on the native host.
pub fn console_log(s: &str) {
  #[allow(unsafe_code)]
  unsafe {
    __console_log(s.as_ptr(), s.len());
  }
}

#[allow(clippy::future_not_send)]
/// The function through which all host calls take place.
pub fn async_host_call<'a>(binding: &'a str, ns: &'a str, op: &'a str, msg: &'a [u8]) -> BoxedFuture<CallResult> {
  let id = CALL_NUM.fetch_add(1, Ordering::SeqCst);
  let (send, recv) = tokio::sync::oneshot::channel();
  ASYNC_HOST_CALLS.with(|cell| {
    #[allow(unsafe_code)]
    let map = unsafe { (&*cell).get().as_mut().unwrap() };
    map.insert(id, send);
  });

  #[allow(unsafe_code)]
  let callresult = unsafe {
    __async_host_call(
      id,
      binding.as_ptr(),
      binding.len(),
      ns.as_ptr(),
      ns.len(),
      op.as_ptr(),
      op.len(),
      msg.as_ptr(),
      msg.len(),
    )
  };
  println!(">> guest: wasm: async host call result: {}", callresult);

  Box::pin(async move {
    println!(">> guest: inner wasm task awaiting channel recv");
    if callresult != 0 {
      println!(">> guest: call failed");
      // call was not successful
      #[allow(unsafe_code)]
      let errlen = unsafe { __host_error_len(id) };

      let mut buf = Vec::with_capacity(errlen);
      let retptr = buf.as_mut_ptr();

      #[allow(unsafe_code)]
      unsafe {
        __host_error(id, retptr);
        buf.set_len(errlen);
      }
      Ok(buf)
    } else {
      // call succeeded
      match recv.await {
        Ok(code) => {
          println!(">> guest: call succeeded with code: {}", code);
          #[allow(unsafe_code)]
          let len = unsafe { __host_response_len(id) };

          let mut buf = Vec::with_capacity(len);
          let retptr = buf.as_mut_ptr();

          #[allow(unsafe_code)]
          unsafe {
            __host_response(id, retptr);
            buf.set_len(len);
          }
          Ok(buf)
        }
        Err(e) => {
          println!(">> guest: call failed : {}", e);
          Err(Error::Async.into())
        }
      }
    }
  })
}
