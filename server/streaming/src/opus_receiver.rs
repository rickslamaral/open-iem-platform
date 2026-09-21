//! Native/headless Opus receiver core (L1 SIMULATED output).
//!
//! Decoder and playout state have no UI dependency. Network ingress uses a
//! bounded queue; queue overflow drops newest packet and never blocks caller.
//! OS audio output is represented by [`AudioOutput`] until PipeWire/ALSA
//! integration is validated on hardware (ADR-008/010).

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use opus_pure::OpusDecoder;
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU64, Ordering},
    sync::Arc,
};
use thiserror::Error;

use observability::ReceiverMetrics;

/// Maximum encoded packets retained before newest packet is dropped.
pub const RECEIVER_QUEUE_CAPACITY: usize = 32;
const MAX_PACKET_BYTES: usize = 1500;
const MAX_JITTER_CAPACITY: usize = 256;
const MAX_DECODED_SAMPLES: usize = 5760;
/// MVP Opus packetization is fixed at 20 ms, 48 kHz stereo.
const PLC_FRAME_SAMPLES: usize = 960;
/// Maximum consecutive PLC-concealed frames before fail-safe mute.
const PLC_MAX_CONSECUTIVE: u32 = 4;

/// Receiver lifecycle. `Muted` is fail-safe: no stale audio reaches output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverState {
    Connecting,
    Playing,
    Muted,
    Reconnecting,
}

/// Errors from receiver operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReceiverError {
    #[error("receiver packet queue full")]
    QueueFull,
    #[error("receiver packet queue disconnected")]
    Disconnected,
    #[error("invalid Opus packet")]
    InvalidPacket,
    #[error("audio output failed")]
    OutputFailed,
    #[error("duplicate sequence number")]
    DuplicateSequence,
}

/// Headless audio sink. Implementations must return quickly; no sink call runs
/// on network ingress, only on [`OpusReceiver::playout`].
pub trait AudioOutput {
    /// Write interleaved f32 PCM samples. `channels` is always 2 for MVP.
    ///
    /// # Errors
    ///
    /// Returns an output error when sink cannot accept samples.
    fn write(&mut self, samples: &[f32], channels: u8) -> Result<(), OutputError>;

    /// Immediately silence and discard any buffered output.
    fn mute(&mut self);
}

/// Audio sink failure marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputError;

/// Bounded packet jitter buffer. Sequence gaps are tolerated and reported by
/// `pop`; missing packets produce mute rather than fabricated audio.
#[derive(Debug)]
pub struct JitterBuffer {
    packets: VecDeque<(u64, Vec<u8>)>,
    capacity: usize,
}

impl JitterBuffer {
    /// Create bounded buffer.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            packets: VecDeque::with_capacity(capacity.min(MAX_JITTER_CAPACITY)),
            capacity: capacity.min(MAX_JITTER_CAPACITY),
        }
    }
    /// Insert packet in sequence order. Rejects malformed size and overflow.
    ///
    /// # Errors
    ///
    /// Returns [`ReceiverError::InvalidPacket`] for invalid payloads or
    /// [`ReceiverError::QueueFull`] when capacity is exhausted.
    pub fn push(&mut self, sequence: u64, packet: &[u8]) -> Result<(), ReceiverError> {
        if packet.is_empty() || packet.len() > MAX_PACKET_BYTES {
            return Err(ReceiverError::InvalidPacket);
        }
        if self.packets.iter().any(|(seq, _)| *seq == sequence) {
            return Err(ReceiverError::DuplicateSequence);
        }
        if self.packets.len() >= self.capacity {
            return Err(ReceiverError::QueueFull);
        }
        let pos = self
            .packets
            .iter()
            .position(|(seq, _)| *seq > sequence)
            .unwrap_or(self.packets.len());
        self.packets.insert(pos, (sequence, packet.to_vec()));
        Ok(())
    }
    /// Inspect lowest sequence packet without removing it.
    #[must_use]
    pub fn peek(&self) -> Option<(u64, &[u8])> {
        self.packets
            .front()
            .map(|(sequence, packet)| (*sequence, packet.as_slice()))
    }

    /// Remove lowest sequence packet.
    pub fn pop(&mut self) -> Option<(u64, Vec<u8>)> {
        self.packets.pop_front()
    }
    /// Number packets buffered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.packets.len()
    }
    /// Whether buffer has no packets.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }
}

/// Native/headless receiver. `SIMULATED` means output sink is supplied by caller.
pub struct OpusReceiver {
    ingress: Sender<(u64, u64, [u8; MAX_PACKET_BYTES], usize)>,
    ingress_rx: Receiver<(u64, u64, [u8; MAX_PACKET_BYTES], usize)>,
    jitter: JitterBuffer,
    decoder: OpusDecoder,
    state: ReceiverState,
    next_sequence: Option<u64>,
    pcm: Vec<f32>,
    dropped_packets: u64,
    /// Count of PLC-concealed frames since last good decode.
    plc_consecutive: u32,
    /// Total PLC frames generated (cumulative, never resets).
    plc_frames_total: u64,
    generation: Arc<AtomicU64>,
    output_failed: bool,
    metrics: Option<Arc<ReceiverMetrics>>,
}

impl OpusReceiver {
    /// Create 48 kHz stereo receiver with bounded ingress and jitter queues.
    ///
    /// # Errors
    ///
    /// Returns [`ReceiverError::InvalidPacket`] if decoder initialization fails.
    pub fn new() -> Result<Self, ReceiverError> {
        let (tx, rx) = bounded(RECEIVER_QUEUE_CAPACITY);
        let decoder = OpusDecoder::new(48_000, 2).map_err(|_| ReceiverError::InvalidPacket)?;
        Ok(Self {
            ingress: tx,
            ingress_rx: rx,
            jitter: JitterBuffer::new(RECEIVER_QUEUE_CAPACITY),
            decoder,
            state: ReceiverState::Connecting,
            next_sequence: None,
            pcm: vec![0.0; MAX_DECODED_SAMPLES * 2],
            dropped_packets: 0,
            plc_consecutive: 0,
            plc_frames_total: 0,
            generation: Arc::new(AtomicU64::new(0)),
            output_failed: false,
            metrics: None,
        })
    }
    /// Attach observability metrics to this receiver.
    #[must_use]
    pub fn with_metrics(mut self, metrics: Arc<ReceiverMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Enqueue encoded payload without waiting. `sequence` must be a monotonic
    /// extended RTP sequence number; rollover must be extended by transport layer.
    ///
    ///
    /// # Errors
    ///
    /// Returns an error for invalid payloads, full queue, or disconnected receiver.
    pub fn enqueue(&self, sequence: u64, packet: &[u8]) -> Result<(), ReceiverError> {
        if packet.is_empty() || packet.len() > MAX_PACKET_BYTES {
            if let Some(ref m) = self.metrics {
                m.record_dropped();
            }
            return Err(ReceiverError::InvalidPacket);
        }
        let generation = self.generation.load(Ordering::Acquire);
        let mut payload = [0_u8; MAX_PACKET_BYTES];
        payload[..packet.len()].copy_from_slice(packet);
        self.ingress
            .try_send((generation, sequence, payload, packet.len()))
            .map_err(|e| match e {
                TrySendError::Full(_) => {
                    if let Some(ref m) = self.metrics {
                        m.record_dropped();
                    }
                    ReceiverError::QueueFull
                }
                TrySendError::Disconnected(_) => ReceiverError::Disconnected,
            })
    }
    /// Decode one fixed 20 ms packet and write PCM. Missing packet uses bounded PLC.
    /// Variable-duration Opus packets fail closed so PLC budget remains time-bounded.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid Opus data or failed output.
    #[allow(clippy::too_many_lines)]
    pub fn playout<O: AudioOutput>(&mut self, output: &mut O) -> Result<(), ReceiverError> {
        let generation = self.generation.load(Ordering::Acquire);
        while let Ok(packet) = self.ingress_rx.try_recv() {
            if packet.0 != generation {
                // Reconnect can race with an ingress sender after the drain above.
                // Count late packets here instead of silently losing telemetry.
                if let Some(ref m) = self.metrics {
                    m.record_dropped();
                }
                continue;
            }
            match self.jitter.push(packet.1, &packet.2[..packet.3]) {
                Err(ReceiverError::DuplicateSequence) => {
                    self.dropped_packets = self.dropped_packets.saturating_add(1);
                    if let Some(ref m) = self.metrics {
                        m.record_late();
                    }
                }
                Err(_) => {
                    self.dropped_packets = self.dropped_packets.saturating_add(1);
                    if let Some(ref m) = self.metrics {
                        m.record_dropped();
                    }
                }
                Ok(()) => {
                    if let Some(ref m) = self.metrics {
                        m.record_received();
                    }
                }
            }
        }
        if self.output_failed {
            self.state = ReceiverState::Muted;
            output.mute();
            return Err(ReceiverError::OutputFailed);
        }
        let Some((sequence, _)) = self.jitter.peek() else {
            self.state = ReceiverState::Muted;
            output.mute();
            return Ok(());
        };
        if let Some(expected) = self.next_sequence {
            if sequence < expected {
                let _ = self.jitter.pop();
                self.dropped_packets = self.dropped_packets.saturating_add(1);
                if let Some(ref m) = self.metrics {
                    m.record_late();
                }
                return Ok(());
            }
            if sequence > expected {
                // Packet missing. Attempt PLC up to PLC_MAX_CONSECUTIVE frames.
                if self.plc_consecutive < PLC_MAX_CONSECUTIVE {
                    // Consume budget before decoding; decoder failure must not reopen PLC.
                    self.plc_consecutive += 1;
                    self.next_sequence = Some(expected.saturating_add(1));
                    let Ok(samples) = self.decoder.decode(&[], PLC_FRAME_SAMPLES, &mut self.pcm)
                    else {
                        self.state = ReceiverState::Muted;
                        self.output_failed = true;
                        if let Some(ref m) = self.metrics {
                            m.record_output_failure();
                        }
                        output.mute();
                        return Err(ReceiverError::InvalidPacket);
                    };
                    if samples != PLC_FRAME_SAMPLES
                        || samples.checked_mul(2).is_none()
                        || samples * 2 > self.pcm.len()
                    {
                        self.state = ReceiverState::Muted;
                        self.output_failed = true;
                        if let Some(ref m) = self.metrics {
                            m.record_output_failure();
                        }
                        output.mute();
                        return Err(ReceiverError::InvalidPacket);
                    }
                    self.plc_frames_total = self.plc_frames_total.saturating_add(1);
                    if let Some(ref m) = self.metrics {
                        m.record_plc_frame(self.plc_consecutive);
                    }
                    if output.write(&self.pcm[..samples * 2], 2).is_err() {
                        self.state = ReceiverState::Muted;
                        self.output_failed = true;
                        if let Some(ref m) = self.metrics {
                            m.record_output_failure();
                        }
                        output.mute();
                        return Err(ReceiverError::OutputFailed);
                    }
                    self.state = ReceiverState::Playing;
                    // Expected sequence already advanced before decode.
                } else {
                    // Preserve received packet for explicit reconnect/resynchronization.
                    self.next_sequence = Some(sequence);
                    self.state = ReceiverState::Muted;
                    self.output_failed = true;
                    if let Some(ref m) = self.metrics {
                        m.record_output_failure();
                    }
                    output.mute();
                    return Err(ReceiverError::OutputFailed);
                }
                return Ok(());
            }
        }
        let Some((sequence, packet)) = self.jitter.pop() else {
            self.state = ReceiverState::Muted;
            output.mute();
            return Ok(());
        };
        let samples = self
            .decoder
            .decode(&packet, MAX_DECODED_SAMPLES, &mut self.pcm)
            .map_err(|_| {
                self.state = ReceiverState::Muted;
                self.output_failed = true;
                if let Some(ref m) = self.metrics {
                    m.record_output_failure();
                }
                self.next_sequence = Some(sequence.saturating_add(1));
                output.mute();
                ReceiverError::InvalidPacket
            })?;
        if samples != PLC_FRAME_SAMPLES
            || samples > MAX_DECODED_SAMPLES
            || samples.checked_mul(2).is_none()
            || samples * 2 > self.pcm.len()
        {
            self.state = ReceiverState::Muted;
            self.output_failed = true;
            if let Some(ref m) = self.metrics {
                m.record_output_failure();
            }
            self.next_sequence = Some(sequence.saturating_add(1));
            output.mute();
            return Err(ReceiverError::InvalidPacket);
        }
        output.write(&self.pcm[..samples * 2], 2).map_err(|_| {
            self.state = ReceiverState::Muted;
            self.output_failed = true;
            if let Some(ref m) = self.metrics {
                m.record_output_failure();
            }
            self.next_sequence = Some(sequence.saturating_add(1));
            output.mute();
            ReceiverError::OutputFailed
        })?;
        self.next_sequence = Some(sequence.saturating_add(1));
        self.plc_consecutive = 0;
        self.state = ReceiverState::Playing;
        Ok(())
    }
    /// Mark transport failure; subsequent playout stays muted until reconnect.
    pub fn reconnect<O: AudioOutput>(&mut self, output: &mut O) {
        self.state = ReceiverState::Reconnecting;
        output.mute();
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.next_sequence = None;
        self.output_failed = false;
        self.plc_consecutive = 0;
        if let Some(ref m) = self.metrics {
            m.record_reconnect();
        }
        // Discard stale ingress packets and account for each rejected packet.
        while self.ingress_rx.try_recv().is_ok() {
            if let Some(ref m) = self.metrics {
                m.record_dropped();
            }
        }
    }
    /// Number packets rejected by jitter admission (overflow or duplicate).
    #[must_use]
    pub fn dropped_packets(&self) -> u64 {
        self.dropped_packets
    }

    /// Current fail-safe lifecycle state.
    #[must_use]
    pub fn state(&self) -> ReceiverState {
        self.state
    }

    /// Consecutive PLC frames generated since last good decode.
    #[must_use]
    pub fn plc_consecutive(&self) -> u32 {
        self.plc_consecutive
    }

    /// Total PLC frames generated (cumulative, never resets on reconnect).
    #[must_use]
    pub fn plc_frames_total(&self) -> u64 {
        self.plc_frames_total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Sink {
        frames: usize,
    }
    impl AudioOutput for Sink {
        fn write(&mut self, s: &[f32], _: u8) -> Result<(), OutputError> {
            self.frames += s.len();
            Ok(())
        }
        fn mute(&mut self) {}
    }
    #[test]
    fn jitter_orders_packets() {
        let mut j = JitterBuffer::new(2);
        j.push(2, b"b").unwrap();
        j.push(1, b"a").unwrap();
        assert_eq!(j.pop().unwrap().0, 1);
    }
    #[test]
    fn jitter_is_bounded() {
        let mut j = JitterBuffer::new(1);
        j.push(1, b"a").unwrap();
        assert_eq!(j.push(2, b"b"), Err(ReceiverError::QueueFull));
    }

    #[test]
    fn zero_capacity_jitter_rejects_packets() {
        let mut j = JitterBuffer::new(0);
        assert_eq!(j.push(1, b"a"), Err(ReceiverError::QueueFull));
        assert!(j.is_empty());
    }
    #[test]
    fn empty_playout_mutes() {
        let mut r = OpusReceiver::new().unwrap();
        let mut s = Sink { frames: 0 };
        r.playout(&mut s).unwrap();
        assert_eq!(r.state(), ReceiverState::Muted);
    }
    #[test]
    fn reconnect_clears_state() {
        let mut r = OpusReceiver::new().unwrap();
        r.reconnect(&mut Sink { frames: 0 });
        assert_eq!(r.state(), ReceiverState::Reconnecting);
    }

    #[test]
    fn reconnect_preserves_queued_packet_for_resynchronization() {
        let mut r = OpusReceiver::new().unwrap();
        let pkt = make_opus_packet();
        r.enqueue(1, &pkt).unwrap();
        r.enqueue(7, &pkt).unwrap();
        let mut s = Sink { frames: 0 };
        r.playout(&mut s).unwrap();
        for _ in 0..4 {
            r.playout(&mut s).unwrap();
        }
        assert_eq!(r.playout(&mut s), Err(ReceiverError::OutputFailed));
        r.reconnect(&mut s);
        assert_eq!(r.playout(&mut s), Ok(()));
        assert_eq!(r.state(), ReceiverState::Playing);
    }

    fn make_opus_packet() -> Vec<u8> {
        use opus_pure::{Application, OpusEncoder};
        let mut enc = OpusEncoder::new(48_000, 2, Application::Audio).unwrap();
        let pcm = vec![0.0f32; 960 * 2]; // 20 ms stereo silence
        let mut out = vec![0u8; 4000];
        let n = enc.encode(&pcm, 960, &mut out).unwrap();
        out[..n].to_vec()
    }

    #[test]
    fn plc_triggers_on_missing_packet() {
        // Enqueue seq=1, then seq=3 (seq=2 missing). After decoding seq=1,
        // next call sees seq=3 > expected=2 → PLC fires.
        let r = OpusReceiver::new().unwrap();
        let pkt = make_opus_packet();
        r.enqueue(1, &pkt).unwrap();
        r.enqueue(3, &pkt).unwrap(); // seq=2 is missing
        let mut r = r;
        let mut s = Sink { frames: 0 };
        // First playout: seq=1 decodes normally.
        r.playout(&mut s).unwrap();
        assert_eq!(r.state(), ReceiverState::Playing);
        assert_eq!(r.plc_consecutive(), 0);
        // Second playout: seq=3 arrives but expected=2 → PLC.
        r.playout(&mut s).unwrap();
        assert_eq!(r.state(), ReceiverState::Playing, "PLC should keep Playing");
        assert_eq!(r.plc_consecutive(), 1, "one PLC frame generated");
        assert_eq!(r.plc_frames_total(), 1);
    }

    #[test]
    fn plc_reset_on_good_decode() {
        let r = OpusReceiver::new().unwrap();
        let pkt = make_opus_packet();
        // seq=1 good, seq=2 missing → PLC, seq=3 good → reset
        r.enqueue(1, &pkt).unwrap();
        r.enqueue(3, &pkt).unwrap();
        let mut r = r;
        let mut s = Sink { frames: 0 };
        r.playout(&mut s).unwrap(); // decode seq=1
        r.playout(&mut s).unwrap(); // PLC for missing seq=2
        assert_eq!(r.plc_consecutive(), 1);
        // Next playout: PLC still fires (expected advances to 3, but seq=3 in jitter → decode good)
        // After advancing expected from 2 to 3, seq=3 == expected=3 → good decode
        r.playout(&mut s).unwrap(); // decode seq=3
        assert_eq!(r.plc_consecutive(), 0, "PLC counter reset on good decode");
    }

    #[test]
    fn plc_budget_exhaustion_mutes() {
        let r = OpusReceiver::new().unwrap();
        let pkt = make_opus_packet();
        // seq=1 good, then gap of 5 missing packets (2,3,4,5,6), seq=7 arrives.
        // PLC fires 4 times, then budget exhausted → mute + resync.
        r.enqueue(1, &pkt).unwrap();
        r.enqueue(7, &pkt).unwrap();
        let mut r = r;
        let mut s = Sink { frames: 0 };
        r.playout(&mut s).unwrap(); // decode seq=1
                                    // PLC frames 1..4
        for i in 1..=4 {
            r.playout(&mut s).unwrap();
            if i < 4 {
                assert_eq!(
                    r.state(),
                    ReceiverState::Playing,
                    "PLC frame {i} should play"
                );
            }
        }
        // 5th call: budget exhausted → latched mute
        assert_eq!(r.playout(&mut s), Err(ReceiverError::OutputFailed));
        assert_eq!(r.state(), ReceiverState::Muted, "budget exhausted → muted");
        assert_eq!(
            r.plc_consecutive(),
            4,
            "budget remains exhausted until good decode"
        );
        assert_eq!(
            r.playout(&mut s),
            Err(ReceiverError::OutputFailed),
            "budget exhaustion must remain latched"
        );
    }

    #[test]
    fn plc_counter_reset_on_reconnect() {
        let r = OpusReceiver::new().unwrap();
        let pkt = make_opus_packet();
        r.enqueue(1, &pkt).unwrap();
        r.enqueue(3, &pkt).unwrap();
        let mut r = r;
        let mut s = Sink { frames: 0 };
        r.playout(&mut s).unwrap();
        r.playout(&mut s).unwrap(); // PLC
        assert_eq!(r.plc_consecutive(), 1);
        r.reconnect(&mut s);
        assert_eq!(r.plc_consecutive(), 0, "reconnect must reset PLC counter");
    }

    #[test]
    fn metrics_record_output_failure_once_when_plc_budget_exhausts() {
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let mut r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let pkt = make_opus_packet();
        r.enqueue(1, &pkt).unwrap();
        r.enqueue(7, &pkt).unwrap();
        let mut s = Sink { frames: 0 };
        r.playout(&mut s).unwrap();
        for _ in 0..4 {
            r.playout(&mut s).unwrap();
        }
        assert_eq!(r.playout(&mut s), Err(ReceiverError::OutputFailed));
        assert_eq!(metrics.snapshot().output_failures, 1);
        assert_eq!(r.playout(&mut s), Err(ReceiverError::OutputFailed));
        assert_eq!(metrics.snapshot().output_failures, 1);
    }

    #[test]
    fn metrics_record_output_failure_on_decoder_error() {
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let mut r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let mut s = Sink { frames: 0 };
        r.enqueue(1, &[0xff]).unwrap();

        assert_eq!(r.playout(&mut s), Err(ReceiverError::InvalidPacket));
        assert_eq!(metrics.snapshot().output_failures, 1);
    }

    #[test]
    fn metrics_record_output_failure_on_output_error() {
        struct FailingSink;
        impl AudioOutput for FailingSink {
            fn write(&mut self, _: &[f32], _: u8) -> Result<(), OutputError> {
                Err(OutputError)
            }
            fn mute(&mut self) {}
        }
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let mut r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let pkt = make_opus_packet();
        r.enqueue(1, &pkt).unwrap();
        assert_eq!(
            r.playout(&mut FailingSink),
            Err(ReceiverError::OutputFailed)
        );
        assert_eq!(metrics.snapshot().output_failures, 1);
    }

    #[test]
    fn metrics_record_received_on_good_packet() {
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let pkt = make_opus_packet();
        r.enqueue(1, &pkt).unwrap();
        let mut r = r;
        let mut s = Sink { frames: 0 };
        r.playout(&mut s).unwrap();
        let snap = metrics.snapshot();
        assert_eq!(
            snap.packets_received, 1,
            "good packet must increment received"
        );
        assert_eq!(snap.packets_dropped, 0);
    }

    #[test]
    fn metrics_record_dropped_for_stale_generation_after_reconnect_race() {
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let mut r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let pkt = make_opus_packet();
        let mut s = Sink { frames: 0 };

        r.reconnect(&mut s);
        // Inject old-generation ingress after reconnect drain. This exercises
        // the stale-generation branch in playout deterministically.
        let mut payload = [0_u8; MAX_PACKET_BYTES];
        payload[..pkt.len()].copy_from_slice(&pkt);
        r.ingress
            .try_send((0, 1, payload, pkt.len()))
            .expect("test stale packet enqueue");
        r.playout(&mut s).unwrap();

        assert_eq!(metrics.snapshot().packets_dropped, 1);
    }

    #[test]
    fn metrics_record_dropped_on_invalid_ingress_packet() {
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let oversized = vec![0_u8; MAX_PACKET_BYTES + 1];

        assert_eq!(r.enqueue(1, &[]), Err(ReceiverError::InvalidPacket));
        assert_eq!(r.enqueue(2, &oversized), Err(ReceiverError::InvalidPacket));

        let snap = metrics.snapshot();
        assert_eq!(snap.packets_received, 0);
        assert_eq!(snap.packets_dropped, 2);
    }

    #[test]
    fn metrics_record_dropped_on_overflow() {
        // Strategy: fill jitter to (capacity-1) via round-1 playout, then push
        // 2 more packets; the second overflows jitter and must be metered as dropped.
        //
        // Jitter capacity = RECEIVER_QUEUE_CAPACITY = 32.
        // Round 1: enqueue seq 1..=32, playout drains all into jitter then pops seq=1
        //          (playing one frame). After round 1: jitter has 31 entries.
        // Round 2: enqueue seq 33 (fits, jitter=32) and seq 34 (overflows).
        //          playout drains both; seq 33 increments received, seq 34 increments dropped.
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let mut r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let pkt = make_opus_packet();
        let mut s = Sink { frames: 0 };
        // Round 1: fill ingress with seq 1..=32.
        for i in 1..=(RECEIVER_QUEUE_CAPACITY as u64) {
            r.enqueue(i, &pkt).ok();
        }
        r.playout(&mut s).unwrap(); // drains ingress → jitter(31 remaining), pops seq=1
        let snap = metrics.snapshot();
        assert_eq!(snap.packets_received, RECEIVER_QUEUE_CAPACITY as u64);
        assert_eq!(snap.packets_dropped, 0);
        // Round 2: enqueue seq 33 (fills jitter to 32) and seq 34 (overflows).
        r.enqueue(33, &pkt).ok();
        r.enqueue(34, &pkt).ok();
        r.playout(&mut s).unwrap();
        let snap = metrics.snapshot();
        assert_eq!(
            snap.packets_received,
            RECEIVER_QUEUE_CAPACITY as u64 + 1,
            "seq 33 is the 33rd received packet"
        );
        assert_eq!(snap.packets_dropped, 1, "seq 34 should overflow jitter");
    }

    #[test]
    fn metrics_record_late_on_stale_packet() {
        // A packet replayed at the same sequence after it was already played
        // must increment late_packets, not packets_dropped.
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let mut r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let pkt = make_opus_packet();
        let mut s = Sink { frames: 0 };
        r.enqueue(1, &pkt).unwrap();
        r.playout(&mut s).unwrap();
        // Replay same sequence: should be stale.
        r.enqueue(1, &pkt).unwrap();
        r.playout(&mut s).unwrap();
        let snap = metrics.snapshot();
        assert_eq!(
            snap.late_packets, 1,
            "replayed packet must be counted as late"
        );
        assert_eq!(
            snap.packets_dropped, 0,
            "late packet must not increment packets_dropped"
        );
    }

    #[test]
    fn metrics_record_dropped_on_ingress_overflow() {
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let pkt = make_opus_packet();
        for sequence in 1..=RECEIVER_QUEUE_CAPACITY as u64 {
            r.enqueue(sequence, &pkt).unwrap();
        }
        assert_eq!(r.enqueue(33, &pkt), Err(ReceiverError::QueueFull));
        assert_eq!(metrics.snapshot().packets_dropped, 1);
    }

    #[test]
    fn metrics_record_reconnect_drops_stale_ingress_packets() {
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let mut r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let pkt = make_opus_packet();
        let mut s = Sink { frames: 0 };

        r.enqueue(1, &pkt).unwrap();
        r.enqueue(2, &pkt).unwrap();
        r.reconnect(&mut s);

        assert_eq!(metrics.snapshot().packets_dropped, 2);
    }

    #[test]
    fn metrics_record_reconnect_on_reconnect_call() {
        let metrics = Arc::new(observability::ReceiverMetrics::default());
        let mut r = OpusReceiver::new()
            .unwrap()
            .with_metrics(Arc::clone(&metrics));
        let mut s = Sink { frames: 0 };
        assert_eq!(metrics.snapshot().reconnect_count, 0);
        r.reconnect(&mut s);
        assert_eq!(
            metrics.snapshot().reconnect_count,
            1,
            "reconnect must increment counter"
        );
        r.reconnect(&mut s);
        assert_eq!(metrics.snapshot().reconnect_count, 2);
    }
}
