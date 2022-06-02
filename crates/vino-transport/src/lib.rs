#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/71604398?s=200&v=4")]
#![doc = include_str!("../README.md")]
// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::if_then_some_else_none,
  bad_style,
  clashing_extern_declarations,
  const_err,
  dead_code,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
#![allow(unused_attributes)]
// !!END_LINTS
// Add exceptions here
#![allow()]

/// The crate's error module;.
pub mod error;

/// The core module that contains the [MessageTransport] and [TransportWrapper]
mod message_transport;

pub(crate) type Result<T> = std::result::Result<T, error::TransportError>;

/// The crate's Error type.
pub type Error = error::TransportError;

/// The module containing [Invocation] related logic.
#[cfg(feature = "invocation")]
pub mod invocation;

#[cfg(feature = "invocation")]
pub use invocation::{InherentData, Invocation};
#[cfg(feature = "async")]
pub use message_transport::stream::{BoxedTransportStream, TransportStream};
#[cfg(feature = "json")]
pub use message_transport::transport_json::{JsonError, TransportJson};
pub use message_transport::transport_map::TransportMap;
pub use message_transport::transport_wrapper::TransportWrapper;
pub use message_transport::{Failure, MessageSignal, MessageTransport, Success};

/// The name of system-originating messages on a port, schematic, or origin.
pub const SYSTEM_ID: &str = "<system>";

/// The reserved port name to use when a component returns an error before it has a chance to send it to an output port.
pub const COMPONENT_ERROR: &str = "<error>";

#[macro_use]
extern crate tracing;
