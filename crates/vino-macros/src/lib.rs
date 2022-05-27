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
#![allow(unused_attributes)]
// !!END_LINTS
// Add exceptions here
#![allow()]

#[macro_export]
/// Test a condition and if it is false, return the supplied error.
/// It's like an assert! that doesn't panic.
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($msg.into());
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return Err($err);
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err(format!($fmt, $($arg)*));
        }
    };
}

#[macro_export]
/// Test a condition and if it is false, return the supplied error.
/// It's like an assert! that doesn't panic.
macro_rules! map_wrapper {
  ($cond:ident, $t:ty, $($arg:tt)*) => {
    impl $cond {

      #[must_use]
      /// Get the value for the requested field.
      fn get<K: AsRef<str>>(&self, field: K) -> Option<&$t> {
        self.0.get(field.as_ref())
      }

      #[must_use]
      /// Get the value for the requested field.
      fn contains_key<K: AsRef<str>>(&self, field: K) -> bool {
        self.0.contains_key(field.as_ref())
      }

      /// Insert a $t into the inner map.
      fn insert<K: AsRef<str>>(&mut self, field: K, value: $t) {
        self.0.insert(field.as_ref().to_owned(), value);
      }

      $($arg)*
    }
  };
}

#[macro_export]
/// Test a condition and if it is false, return the supplied error.
/// It's like an assert! that doesn't panic.
macro_rules! kv_impl {
  ($t:ty) => {
    kv_impl!{$t, pub(self)}
  };
  ($t:ty, $v:vis) => {
    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Get the value for the requested field.
    $v fn get<K: AsRef<str>>(&self, field: K) -> Option<&$t> {
      self.0.get(field.as_ref())
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Get the value for the requested field.
    $v fn get_mut<K: AsRef<str>>(&mut self, field: K) -> Option<&mut $t> {
      self.0.get_mut(field.as_ref())
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Get the value for the requested field.
    $v fn contains_key<K: AsRef<str>>(&self, field: K) -> bool {
      self.0.contains_key(field.as_ref())
    }

    /// Insert a $t into the inner map.
    #[allow(unused, unreachable_pub)]
    $v fn insert<K: AsRef<str>>(&mut self, field: K, value: $t) {
      self.0.insert(field.as_ref().to_owned(), value);
    }


    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Return a list of names in the inner HashMap.
    $v fn names(&self) -> Vec<String> {
      self.0.keys().cloned().collect()
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Return true if the inner HashMap is empty.
    $v fn is_empty(&self) -> bool {
      self.0.is_empty()
    }

    /// Return the inner HashMap.
    #[must_use]
    #[allow(unused, unreachable_pub)]
    $v fn into_inner(self) -> std::collections::HashMap<String, $t> {
      self.0
    }

    /// Return a reference to the inner HashMap.
    #[must_use]
    #[allow(unused, unreachable_pub)]
    $v fn inner(&self) -> &std::collections::HashMap<String, $t> {
      &self.0
    }

    #[must_use]
    #[allow(unused, unreachable_pub)]
    /// Returns the number of fields in the map.
    $v fn len(&self) -> usize {
      self.0.len()
    }
  };
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  #[test]
  fn map_wrapper() -> Result<()> {
    #[derive(Default)]
    struct MyMap(std::collections::HashMap<String, u32>);
    impl MyMap {
      fn custom_len(&self) -> usize {
        self.0.len() * 10
      }
      kv_impl! {u32}
    }

    let mut map = MyMap::default();
    map.insert("hey", 0);

    assert_eq!(map.custom_len(), 10);
    Ok(())
  }
}
