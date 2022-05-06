use serde::{Deserialize, Serialize};
use vino_entity::Entity;

/// An implementation that encapsulates a provider link that components can use to call out to a Vino network.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct ProviderLink(Entity, Entity);

impl ProviderLink {
  /// Constructor for a [ProviderLink]
  pub fn new(from: Entity, to: Entity) -> Self {
    Self(from, to)
  }

  #[must_use]
  /// Get the URL for the called component
  pub fn get_origin_url(&self) -> String {
    self.0.url()
  }

  /// Make a call to the linked provider.
  #[cfg(all(not(feature = "native"), feature = "wasm"))]
  pub fn call(
    &self,
    component: &str,
    payload: impl Into<vino_transport::TransportMap>,
  ) -> vino_wapc::guest::wasm::BoxedFuture<Result<crate::wasm::prelude::ProviderOutput, crate::wasm::Error>> {
    let payload: vino_transport::TransportMap = payload.into();
    let origin = self.get_origin_url();
    let target = Entity::component(self.1.namespace(), component).url();
    Box::pin(async move {
      let result = vino_wapc::guest::wasm::runtime::async_host_call(
        "1",
        &origin,
        &target,
        &vino_codec::messagepack::serialize(&payload)?,
      )
      .await
      .map_err(crate::wasm::Error::Protocol)?;
      let packets: Vec<vino_transport::TransportWrapper> = vino_codec::messagepack::deserialize(&result)?;
      Ok(crate::wasm::prelude::ProviderOutput::new(packets))
    })
  }

  /// Make a call to the linked provider.
  #[cfg(all(not(feature = "wasm"), feature = "native"))]
  pub fn call(
    &self,
    _component: &str,
    _payload: impl Into<vino_transport::TransportMap>,
  ) -> Result<crate::native::prelude::ProviderOutput, crate::native::Error> {
    unimplemented!("Link calls from native providers is not implemented yet")
  }
}

impl std::fmt::Display for ProviderLink {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}=>{}", self.0, self.1)
  }
}
