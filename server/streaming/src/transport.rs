//! Explicit socket owner for bounded Sans-IO WebRTC output.
//!
//! `SessionRegistry` never performs network I/O. This adapter owns one UDP
//! socket and sends only the bounded datagrams handed to it by the registry.

use std::io;
use tokio::net::UdpSocket;

/// Maximum datagrams sent by one adapter pass.
pub const TRANSPORT_SEND_BUDGET: usize = 32;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TransportSendReport {
    pub attempted: usize,
    pub sent: usize,
    pub bytes: usize,
    /// Datagrams dropped while requeueing failed registry sends.
    /// Generic iterator sends cannot report unconsumed items without traversing
    /// them, so this remains zero for `send`.
    pub dropped: usize,
}

pub struct TransportAdapter {
    socket: UdpSocket,
}

impl TransportAdapter {
    /// Bind an adapter socket. All socket I/O stays inside this owner.
    ///
    /// # Errors
    ///
    /// Returns the OS bind error when the address cannot be opened.
    pub async fn bind(addr: std::net::SocketAddr) -> io::Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(addr).await?,
        })
    }

    ///
    /// # Errors
    ///
    /// Returns the OS error when the socket address cannot be queried.
    pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.socket.local_addr()
    }

    /// Send at most `budget` queued Sans-IO datagrams.
    ///
    /// # Errors
    ///
    /// Returns the OS send error.
    ///
    /// A failed send stops the pass and returns the successful prefix. This
    /// generic method consumes only the bounded prefix; use
    /// `send_from_registry` when failed registry datagrams must be requeued.
    pub async fn send(
        &self,
        outputs: impl IntoIterator<Item = str0m::net::Transmit>,
        budget: usize,
    ) -> io::Result<TransportSendReport> {
        let mut report = TransportSendReport::default();
        let limit = budget.min(TRANSPORT_SEND_BUDGET);
        let mut outputs = outputs.into_iter();
        for _ in 0..limit {
            let Some(transmit) = outputs.next() else {
                break;
            };
            report.attempted += 1;
            let expected = transmit.contents.len();
            let sent = self
                .socket
                .send_to(&transmit.contents, transmit.destination)
                .await?;
            if sent != expected {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "UDP datagram was written partially",
                ));
            }
            report.sent += 1;
            report.bytes += sent;
        }
        // Do not inspect the remainder: a generic iterator may be unbounded.
        // Registry callers use `send_from_registry`, which tracks requeue drops.
        Ok(report)
    }

    /// Drain registry output and send bounded datagrams in one ownership step.
    ///
    /// # Errors
    ///
    /// Returns the OS send error.
    pub async fn send_from_registry(
        &self,
        registry: &crate::SessionRegistry,
        budget: usize,
    ) -> io::Result<TransportSendReport> {
        let mut report = TransportSendReport::default();
        let limit = budget.min(TRANSPORT_SEND_BUDGET);
        let outputs = registry.drain_transport_outputs(limit).await;
        let mut pending = outputs.into_iter();
        for _ in 0..limit {
            let Some(transmit) = pending.next() else {
                break;
            };
            report.attempted += 1;
            let expected = transmit.contents.len();
            let result = self
                .socket
                .send_to(&transmit.contents, transmit.destination)
                .await;
            match result {
                Ok(sent) if sent == expected => {
                    report.sent += 1;
                    report.bytes += sent;
                }
                Ok(_) => {
                    let mut unsent = vec![transmit];
                    unsent.extend(pending);
                    let dropped = registry.requeue_transport_outputs(unsent).await;
                    if dropped > 0 {
                        return Err(io::Error::other(format!(
                            "transport retry queue full; dropped {dropped} datagrams"
                        )));
                    }
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "UDP datagram was written partially",
                    ));
                }
                Err(error) => {
                    let mut unsent = vec![transmit];
                    unsent.extend(pending);
                    let dropped = registry.requeue_transport_outputs(unsent).await;
                    if dropped > 0 {
                        return Err(io::Error::other(format!(
                            "transport retry queue full; dropped {dropped} datagrams"
                        )));
                    }
                    return Err(error);
                }
            }
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AudioOutput, MediaFrame, MediaWriter, OpusReceiver, OutputError, StreamMetadata};
    use observability::ReceiverMetrics;
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use str0m::net::Protocol;

    #[tokio::test]
    async fn bind_exposes_local_socket() {
        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        assert_ne!(adapter.local_addr().unwrap().port(), 0);
    }

    #[tokio::test]
    async fn empty_send_is_bounded_and_noop() {
        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let report = adapter
            .send(Vec::new(), TRANSPORT_SEND_BUDGET)
            .await
            .unwrap();
        assert_eq!(report, TransportSendReport::default());
    }

    struct CaptureOutput {
        samples: usize,
        peak: f32,
    }

    impl AudioOutput for CaptureOutput {
        fn write(&mut self, samples: &[f32], channels: u8) -> Result<(), OutputError> {
            assert_eq!(channels, 2);
            self.samples += samples.len();
            self.peak = self.peak.max(
                samples
                    .iter()
                    .map(|sample| sample.abs())
                    .fold(0.0, f32::max),
            );
            Ok(())
        }

        fn mute(&mut self) {}
    }

    #[tokio::test]
    async fn udp_loopback_delivers_opus_payload_to_headless_receiver() {
        let sink = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let destination = sink.local_addr().unwrap();
        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let frame = MediaFrame {
            metadata: StreamMetadata {
                stream_id: "loopback".into(),
                mix_index: 0,
                revision: 1,
                sequence: 9,
                sample_rate: 48_000,
                channels: 2,
                frame_duration_ms: 20,
                capture_timestamp: None,
            },
            samples: (0.2, -0.1),
        };
        let packet = MediaWriter::new().unwrap().encode(&frame).unwrap();
        let report = adapter
            .send(
                [str0m::net::Transmit {
                    proto: Protocol::Udp,
                    source: adapter.local_addr().unwrap(),
                    destination,
                    contents: packet.payload.clone().into(),
                }],
                1,
            )
            .await
            .unwrap();
        assert_eq!(report.sent, 1);

        let mut payload = vec![0; packet.payload.len()];
        let (len, _) = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            sink.recv_from(&mut payload),
        )
        .await
        .unwrap()
        .unwrap();
        let metrics = Arc::new(ReceiverMetrics::default());
        let mut receiver = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        receiver.enqueue(packet.sequence, &payload[..len]).unwrap();
        let mut output = CaptureOutput {
            samples: 0,
            peak: 0.0,
        };
        receiver.playout(&mut output).unwrap();

        assert_eq!(output.samples, 1_920);
        assert!(output.peak > 0.0);
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.packets_received, 1);
        assert_eq!(snapshot.output_failures, 0);
    }

    #[tokio::test]
    async fn send_caps_oversized_budget() {
        let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let destination = receiver.local_addr().unwrap();
        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let outputs = (0..=TRANSPORT_SEND_BUDGET)
            .map(|index| str0m::net::Transmit {
                proto: Protocol::Udp,
                source: adapter.local_addr().unwrap(),
                destination,
                contents: format!("packet-{index}").into_bytes().into(),
            })
            .collect::<Vec<_>>();
        let consumed = Arc::new(AtomicUsize::new(0));
        let outputs = outputs.into_iter().inspect({
            let consumed = Arc::clone(&consumed);
            move |_| {
                consumed.fetch_add(1, Ordering::Relaxed);
            }
        });

        let report = adapter.send(outputs, usize::MAX).await.unwrap();

        assert_eq!(consumed.load(Ordering::Relaxed), TRANSPORT_SEND_BUDGET);
        assert_eq!(report.attempted, TRANSPORT_SEND_BUDGET);
        assert_eq!(report.sent, TRANSPORT_SEND_BUDGET);
        assert_eq!(report.dropped, 0);
        let mut payload = [0_u8; 32];
        for index in 0..TRANSPORT_SEND_BUDGET {
            let (length, _) = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                receiver.recv_from(&mut payload),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(&payload[..length], format!("packet-{index}").as_bytes());
        }
    }

    #[tokio::test]
    async fn send_respects_budget() {
        let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let destination: SocketAddr = receiver.local_addr().unwrap();
        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let outputs = (0..3)
            .map(|_| str0m::net::Transmit {
                proto: Protocol::Udp,
                source: "127.0.0.1:0".parse().unwrap(),
                destination,
                contents: vec![b'x'].into(),
            })
            .collect::<Vec<_>>();
        let consumed = Arc::new(AtomicUsize::new(0));
        let outputs = outputs.into_iter().inspect({
            let consumed = Arc::clone(&consumed);
            move |_| {
                consumed.fetch_add(1, Ordering::Relaxed);
            }
        });
        let report = adapter.send(outputs, 2).await.unwrap();
        assert_eq!(consumed.load(Ordering::Relaxed), 2);
        assert_eq!(report.attempted, 2);
        assert_eq!(report.sent, 2);
        assert_eq!(report.bytes, 2);
        assert_eq!(report.dropped, 0);
    }
}
