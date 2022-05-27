use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::{StreamExt, StreamMap};
use tracing::*;
use vino_packet::v1::Packet as ComponentPacket;
use vino_packet::{Packet, PacketWrapper};
use wasmflow_streams::PacketStream;

type Error = Box<dyn std::error::Error + Send + Sync>;

type Result = std::result::Result<(), Error>;

/// The native PortSender trait. This trait encapsulates sending messages out of native ports.
pub trait PortSender {
  /// Get the port buffer that the sender can push to.
  fn get_port(&self) -> std::result::Result<&PortChannel, Error>;

  /// Get the port's name.
  fn get_port_name(&self) -> &str;

  /// Send a message.
  fn send(&self, data: impl Into<ComponentPacket>) -> Result {
    self.push(Packet::V1(data.into()))
  }

  /// Send a message then close the port.
  fn done(&self, data: impl Into<ComponentPacket>) -> Result {
    self.send(data)?;
    self.send_message(Packet::V1(ComponentPacket::done()))
  }

  /// Send a complete [Packet] then close the port.
  fn push(&self, output: Packet) -> Result {
    self.get_port()?.send(PacketWrapper {
      payload: output,
      port: self.get_port_name().to_owned(),
    })?;
    Ok(())
  }

  /// Send a complete [Packet].
  fn send_message(&self, packet: Packet) -> Result {
    self.get_port()?.send(PacketWrapper {
      payload: packet,
      port: self.get_port_name().to_owned(),
    })?;
    Ok(())
  }

  /// Send a payload then close the port.
  fn done_message(&self, packet: Packet) -> Result {
    self.send_message(packet)?;
    self.send_message(ComponentPacket::done().into())
  }

  /// Send an exception.
  fn send_exception(&self, payload: String) -> Result {
    self.get_port()?.send(PacketWrapper {
      payload: ComponentPacket::exception(payload).into(),
      port: self.get_port_name().to_owned(),
    })?;
    Ok(())
  }

  /// Send an exception then close the port.
  fn done_exception(&self, payload: String) -> Result {
    self.send_exception(payload)?;
    self.send_message(ComponentPacket::done().into())
  }

  /// Signal that a job is finished with the port.
  fn close(&self) -> Result {
    self.send_message(ComponentPacket::done().into())
  }
}

/// A [PortChannel] wraps an unbounded channel with a port name.
#[must_use]
#[derive(Debug, Clone)]
pub struct PortChannel {
  /// Port name.
  pub name: String,
  incoming: Option<UnboundedSender<PacketWrapper>>,
}

impl PortChannel {
  /// Constructor for a [PortChannel].
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      incoming: None,
    }
  }

  /// Initialize the [PortChannel] and return a receiver.
  pub fn open(&mut self) -> UnboundedReceiverStream<PacketWrapper> {
    let (tx, rx) = unbounded_channel();
    self.incoming = Some(tx);
    UnboundedReceiverStream::new(rx)
  }

  /// Drop the incoming channel, closing the upstream.
  pub fn close(&mut self) {
    self.incoming.take();
  }

  /// Returns true if the port still has an active upstream.
  #[must_use]
  pub fn is_closed(&self) -> bool {
    self.incoming.is_none()
  }

  /// Send a messages to the channel.
  pub fn send(&self, msg: PacketWrapper) -> Result {
    let incoming = self
      .incoming
      .as_ref()
      .ok_or_else::<Error, _>(|| "Send channel closed".into())?;
    incoming.send(msg)?;
    Ok(())
  }

  /// Merge a list of [PortChannel]s into a TransportStream.
  pub fn merge_all(buffer: &mut [&mut PortChannel]) -> PacketStream {
    let (tx, rx) = unbounded_channel::<PacketWrapper>();

    let mut channels = StreamMap::new();
    for channel in buffer {
      channels.insert(channel.name.clone(), channel.open());
    }

    tokio::spawn(async move {
      while let Some((_, msg)) = channels.next().await {
        match tx.send(msg.into()) {
          Ok(_) => {}
          Err(e) => {
            error!("Unexpected error sending to aggregated stream: {}", e);
          }
        };
      }
    });

    PacketStream::new(Box::new(UnboundedReceiverStream::new(rx)))
  }
}

#[cfg(test)]
mod tests {

  use vino_packet::v1::Packet;
  use vino_transport::{TransportStream, TransportWrapper};

  use super::*;
  struct StringSender {
    port: PortChannel,
  }
  impl PortSender for StringSender {
    fn get_port(&self) -> std::result::Result<&PortChannel, Error> {
      Ok(&self.port)
    }

    fn get_port_name(&self) -> &str {
      &self.port.name
    }
  }

  struct I64Sender {
    port: PortChannel,
  }
  impl PortSender for I64Sender {
    fn get_port(&self) -> std::result::Result<&PortChannel, Error> {
      Ok(&self.port)
    }

    fn get_port_name(&self) -> &str {
      &self.port.name
    }
  }

  #[test_log::test(tokio::test)]
  async fn test_merge() -> Result {
    // This sets up the ports, sends data on them, then
    // drops the ports, thus closing them.
    let aggregated = {
      let mut port1 = StringSender {
        port: PortChannel::new("test1"),
      };
      let mut port2 = I64Sender {
        port: PortChannel::new("test2"),
      };

      let aggregated = PortChannel::merge_all(&mut [&mut port1.port, &mut port2.port]);

      port1.send(Packet::success(&"First"))?;
      port2.send(Packet::success(&1u8))?;
      port1.done(Packet::success(&"Second"))?;
      port2.done(Packet::success(&2u8))?;

      aggregated
    };
    let mut aggregated = TransportStream::new(aggregated.map(|pw| pw.into()));

    let mut messages: Vec<TransportWrapper> = aggregated.drain_port("test1").await;
    assert_eq!(messages.len(), 2);
    assert_eq!(aggregated.buffered_size(), (1, 2));
    let payload: String = messages.remove(0).deserialize().unwrap();
    println!("Payload a1: {}", payload);
    assert_eq!(payload, "First");
    let payload: String = messages.remove(0).deserialize().unwrap();
    println!("Payload a2: {}", payload);
    assert_eq!(payload, "Second");

    let mut messages: Vec<TransportWrapper> = aggregated.drain_port("test2").await;
    assert_eq!(messages.len(), 2);
    assert_eq!(aggregated.buffered_size(), (0, 0));
    let payload: i64 = messages.remove(0).deserialize().unwrap();
    println!("Payload b1: {}", payload);
    assert_eq!(payload, 1);
    let payload: i64 = messages.remove(0).deserialize().unwrap();
    println!("Payload b2: {}", payload);
    assert_eq!(payload, 2);

    Ok(())
  }

  #[test_log::test(tokio::test)]
  async fn test_send() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.send(Packet::success(&"first"))?;

    let message: TransportWrapper = rx.next().await.unwrap().into();
    let payload: String = message.payload.deserialize().unwrap();

    assert_eq!(payload, "first");

    Ok(())
  }

  #[test_log::test(tokio::test)]
  async fn test_done() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.done(Packet::success(&"done"))?;

    let message: TransportWrapper = rx.next().await.unwrap().into();
    let payload: String = message.payload.deserialize().unwrap();

    assert_eq!(payload, "done");
    let message = rx.next().await.unwrap();
    assert_eq!(message.payload, Packet::done().into());
    Ok(())
  }

  #[test_log::test(tokio::test)]
  async fn test_exception() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.send_exception("exc".to_owned())?;

    let message = rx.next().await.unwrap();

    assert_eq!(message.payload, Packet::exception("exc").into());

    Ok(())
  }

  #[test_log::test(tokio::test)]
  async fn test_done_exception() -> Result {
    let mut port1 = StringSender {
      port: PortChannel::new("test1"),
    };
    let mut rx = port1.port.open();

    port1.done_exception("exc".to_owned())?;

    let message = rx.next().await.unwrap();

    assert_eq!(message.payload, Packet::exception("exc").into());
    let message = rx.next().await.unwrap();
    assert_eq!(message.payload, Packet::done().into());
    Ok(())
  }
}
