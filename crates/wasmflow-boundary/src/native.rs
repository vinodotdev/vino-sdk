#[cfg(feature = "v1")]
/// Utility functions that return v1 packets.
pub mod v1 {
  use serde::de::DeserializeOwned;

  use crate::incoming::IncomingPayload;

  /// Convert an [vino_transport::Invocation] into an [IncomingPayload].
  pub fn from_invocation<C, S>(
    invocation: vino_transport::Invocation,
  ) -> Result<IncomingPayload<vino_packet::v1::PacketMap, C, S>, Box<dyn std::error::Error + Send + Sync>>
  where
    C: std::fmt::Debug + DeserializeOwned,
    S: std::fmt::Debug + DeserializeOwned,
  {
    let (payload, config, state) = invocation.into_v1_parts().map_err(Box::new)?;

    Ok(IncomingPayload::new(0, payload, config, state))
  }
}
