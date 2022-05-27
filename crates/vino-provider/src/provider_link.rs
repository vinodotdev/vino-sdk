use serde::{Deserialize, Serialize};
use vino_entity::Entity;
use vino_packet::PacketMap;
#[cfg(not(target_arch = "wasm32"))]
use wasmflow_component::guest::native::BoxedFuture;
#[cfg(target_arch = "wasm32")]
use wasmflow_component::guest::wasm::BoxedFuture;

use crate::ProviderOutput;

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
  pub fn call(
    &self,
    component: &str,
    payload: impl Into<PacketMap>,
  ) -> BoxedFuture<Result<ProviderOutput, crate::error::Error>> {
    let payload = payload.into();
    let origin = self.get_origin_url();
    let target = Entity::component(self.1.namespace(), component).url();
    Box::pin(async move {
      let stream = link_call(&origin, &target, &payload).await?;

      Ok(stream)
    })
  }
}

impl std::fmt::Display for ProviderLink {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}=>{}", self.0, self.1)
  }
}

#[cfg(target_arch = "wasm32")]
async fn link_call(origin: &str, target: &str, payload: &PacketMap) -> Result<ProviderOutput, crate::error::Error> {
  let bytes = vino_codec::messagepack::serialize(payload)?;
  println!("bytes for host call {:?}", bytes);
  let result = wasmflow_component::guest::wasm::runtime::async_host_call("1", &origin, &target, &bytes)
    .await
    .map_err(crate::error::Error::Protocol)?;
  println!("post host call {:?}", result);
  let packets: Vec<vino_packet::PacketWrapper> = vino_codec::messagepack::deserialize(&result)?;
  Ok(crate::ProviderOutput::new(tokio_stream::iter(packets)))
}

#[cfg(not(target_arch = "wasm32"))]
async fn link_call(_origin: &str, _target: &str, _payload: &PacketMap) -> Result<ProviderOutput, crate::error::Error> {
  unimplemented!("Link calls from native providers is not implemented yet")
}
