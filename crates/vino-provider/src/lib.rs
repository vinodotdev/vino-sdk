//! The Vino provider crate contains the necessary pieces for Native
//! or WebAssembly providers written in Rust.
//!
//! This library is not meant to be integrated manually. Vino uses
//! code generators to automate most integration and — while backwards compatibility
//! is a top priority — the generated code is considered the primary consumer. If you
//! end up using this library to fit other use cases, please open an issue to let us know
//! so we can track that usage.
//!
//! To use this library or learn more about code generation, check out the docs at
//! [docs.vino.dev](https://docs.vino.dev/docs/concepts/codegen/).

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
  deprecated,
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
// !!END_LINTS
// Add exceptions here
#![allow()]

#[cfg(feature = "wasm")]
/// Traits and functions for wasm providers.
pub mod wasm;

#[cfg(feature = "native")]
/// Traits and functions for native providers.
pub mod native;

#[cfg(feature = "native")]
/// Raw value type.
pub mod raw;

/// Module for the root [ProviderLink] struct.
mod provider_link;
pub use provider_link::ProviderLink;

pub use vino_codec as codec;

/// Feature-dependent prelude that imports items depending on whether the 'wasm' or 'native' features are enabled.
pub mod prelude {
  #[cfg(feature = "native")]
  pub use crate::native::prelude::*;
  #[cfg(feature = "wasm")]
  pub use crate::wasm::prelude::*;
}
