//! System (loopback) audio capture — the "audio coming out of the laptop" path: a remote
//! speaker on Zoom or Teams, a browser tab, a media player. On Windows this uses WASAPI
//! loopback, which captures the render endpoint mix — every application's output, whatever
//! it is — without any virtual audio device. On other platforms it is a stub.
//!
//! The Windows path follows the current `wasapi` event-driven shared-stream API.
//! Everything downstream (resampling, chunking, leveling via `CaptureState`) is shared
//! with the mic path and is exercised by tests.

use anyhow::Result;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::types::AudioLevel;

#[cfg(not(windows))]
pub fn run_system_loopback(
    _device_id: Option<String>,
    _capture: super::applications::SystemCapture,
    _target_rate: u32,
    _level_tx: Sender<AudioLevel>,
    _chunk_tx: crate::audio::sink::AudioSink,
    _cancel: &CancellationToken,
) -> Result<()> {
    anyhow::bail!("System (loopback) capture is only supported on Windows in this build")
}

#[cfg(windows)]
pub fn run_system_loopback(
    device_id: Option<String>,
    capture: super::applications::SystemCapture,
    target_rate: u32,
    level_tx: Sender<AudioLevel>,
    chunk_tx: crate::audio::sink::AudioSink,
    cancel: &CancellationToken,
) -> Result<()> {
    windows_impl::run(device_id, capture, target_rate, level_tx, chunk_tx, cancel)
}

#[cfg(windows)]
mod windows_impl {
    use std::collections::VecDeque;
    use std::time::{Duration, Instant};

    use anyhow::{Context, Result};
    use tokio::sync::mpsc::Sender;
    use tokio_util::sync::CancellationToken;
    use wasapi::{DeviceEnumerator, Direction, SampleType, StreamMode, WaveFormat};

    use crate::audio::applications::{ProcessGuard, ProcessIdentity, SystemCapture};
    use crate::audio::capture::CaptureState;
    use crate::audio::com::Apartment;
    use crate::audio::devices::PresenceCheck;
    use crate::types::{AudioLevel, Origin};

    /// `wasapi`'s fallible calls return `Box<dyn Error>`, which is neither `Send` nor
    /// `Sync` and so cannot flow into `anyhow` through `?` or `.context()`. This bridges
    /// them by stringifying the error behind a static context message.
    trait WasapiCtx<T> {
        fn ctx(self, msg: &'static str) -> Result<T>;
    }

    impl<T, E: std::fmt::Display> WasapiCtx<T> for std::result::Result<T, E> {
        fn ctx(self, msg: &'static str) -> Result<T> {
            self.map_err(|e| anyhow::anyhow!("{msg}: {e}"))
        }
    }

    pub fn run(
        device_id: Option<String>,
        capture: SystemCapture,
        target_rate: u32,
        level_tx: Sender<AudioLevel>,
        chunk_tx: crate::audio::sink::AudioSink,
        cancel: &CancellationToken,
    ) -> Result<()> {
        // COM must be initialised on the capture thread, and left only after every COM
        // object below is released.
        let _apartment = Apartment::mta().context("failed to initialise COM (MTA)")?;

        let Source {
            device,
            process,
            mut client,
            format,
        } = match capture {
            SystemCapture::Output => Source::output(device_id)?,
            SystemCapture::Application { process } => Source::application(process)?,
        };
        let (event, mut reader) = start(&mut client, &format)?;

        let mut state =
            CaptureState::new(Origin::System, reader.rate, target_rate, level_tx, chunk_tx);
        // The loop wakes on every device period (~100 Hz); the endpoint's state only needs
        // asking when Windows reports a change, or once a second in case it did not.
        let mut presence = PresenceCheck::new(Duration::from_secs(1));

        while !cancel.is_cancelled() {
            ensure_present(device.as_ref(), process.as_ref(), &mut presence)?;
            reader.drain(&mut state)?;

            // Wake on the audio event; short timeout keeps cancellation responsive.
            if event.wait_for_event(100).is_err() {
                // Timeout — loop and re-check cancellation.
            }
        }

        let _ = client.stop_stream();
        tracing::info!("WASAPI loopback capture stopped");
        Ok(())
    }

    /// How much audio the endpoint buffer holds, in 100 ns units: 200 ms. It used to be the
    /// device's minimum period, a few milliseconds, so a capture thread that woke one period
    /// late (starved by local Whisper's busy workers, say) found the buffer already full and
    /// lost what came next. 200 ms rides out a long stall; the event still fires every period,
    /// so it adds no latency.
    const BUFFER_DURATION_HNS: i64 = 2_000_000;

    /// The most packets one wake reads before looking at cancellation again. A buffer of
    /// `BUFFER_DURATION_HNS` holds about twenty 10 ms packets, so this only bounds a device
    /// that never stops reporting one.
    const MAX_PACKETS_PER_WAKE: usize = 64;

    /// What loopback captures, and the client and format it opens with.
    struct Source {
        /// The output endpoint, watched for removal. `None` when capturing an application.
        device: Option<wasapi::Device>,
        /// The captured application, watched for exit. `None` when capturing an output.
        process: Option<ProcessGuard>,
        client: wasapi::AudioClient,
        format: WaveFormat,
    }

    impl Source {
        /// The mix of every application playing to the selected output, or the default one.
        fn output(device_id: Option<String>) -> Result<Self> {
            let enumerator = DeviceEnumerator::new().ctx("failed to create device enumerator")?;
            let device = match device_id {
                Some(id) => enumerator
                    .get_device(&id)
                    .ctx("selected output device unavailable")?,
                None => enumerator
                    .get_default_device(&Direction::Render)
                    .ctx("no default render device")?,
            };
            anyhow::ensure!(
                device.get_direction() == Direction::Render,
                "selected device is not an output endpoint"
            );
            let client = device
                .get_iaudioclient()
                .ctx("failed to get IAudioClient")?;
            let format = client.get_mixformat().ctx("failed to get mix format")?;
            Ok(Self {
                device: Some(device),
                process: None,
                client,
                format,
            })
        }

        /// One application's output, through a process loopback client.
        fn application(process: Option<ProcessIdentity>) -> Result<Self> {
            let identity = process.context("select an application before starting capture")?;
            let guard = ProcessGuard::selected(&identity)?;
            let client = wasapi::AudioClient::new_application_loopback_client(identity.pid, true)
                .ctx("failed to capture selected application")?;
            // Process clients have no device mix format.
            let format = WaveFormat::new(32, 32, &SampleType::Float, 48000, 2, None);
            Ok(Self {
                device: None,
                process: Some(guard),
                client,
                format,
            })
        }
    }

    /// Log the stream's format, then initialise `client` for event-driven shared capture and
    /// start it. Returns the event that signals each period and the reader for its packets.
    fn start(
        client: &mut wasapi::AudioClient,
        format: &WaveFormat,
    ) -> Result<(wasapi::Handle, Reader)> {
        let in_rate = format.get_samplespersec();
        let channels = usize::from(format.get_nchannels());
        let bits = format.get_bitspersample();
        let sample_type = format.get_subformat().unwrap_or(SampleType::Float);

        tracing::info!(
            in_rate,
            channels,
            bits,
            ?sample_type,
            "starting WASAPI loopback capture"
        );

        // Loopback = render endpoint opened for capture.
        let mode = StreamMode::EventsShared {
            autoconvert: true,
            buffer_duration_hns: BUFFER_DURATION_HNS,
        };
        client
            .initialize_client(format, &Direction::Capture, &mode)
            .ctx("failed to initialise loopback client")?;

        let event = client
            .set_get_eventhandle()
            .ctx("failed to set event handle")?;
        let capture = client
            .get_audiocaptureclient()
            .ctx("failed to get capture client")?;

        client
            .start_stream()
            .ctx("failed to start loopback stream")?;

        let reader = Reader {
            capture,
            rate: in_rate,
            channels,
            bits,
            sample_type,
            raw: VecDeque::new(),
            frame: Vec::new(),
            discontinuities: Discontinuities::default(),
        };
        Ok((event, reader))
    }

    /// Fail once what is being captured has gone. An endpoint can disappear without
    /// producing another audio event, so its state is asked whenever `presence` is due.
    /// Never reopen the new default silently: the stream stays pinned to its original
    /// endpoint.
    fn ensure_present(
        device: Option<&wasapi::Device>,
        process: Option<&ProcessGuard>,
        presence: &mut PresenceCheck,
    ) -> Result<()> {
        if presence.due() {
            anyhow::ensure!(
                device.is_none_or(|d| matches!(d.get_state(), Ok(wasapi::DeviceState::Active))),
                "selected output device disconnected or disabled"
            );
        }
        anyhow::ensure!(
            process.is_none_or(ProcessGuard::running),
            "selected application has closed; select it again"
        );
        Ok(())
    }

    /// The capture side of a started stream, with the layout of its samples and buffers
    /// reused across wakes.
    struct Reader {
        capture: wasapi::AudioCaptureClient,
        rate: u32,
        channels: usize,
        bits: u16,
        sample_type: SampleType,
        raw: VecDeque<u8>,
        frame: Vec<f32>,
        discontinuities: Discontinuities,
    }

    impl Reader {
        /// Decode every packet the endpoint holds and feed each to `state`.
        /// `read_from_device_to_deque` reads one packet per call, so reading once per wake
        /// fell behind for good after any wake that came late: the backlog never cleared and
        /// the buffer overflowed. So this reads until `get_next_packet_size` reports none.
        fn drain(&mut self, state: &mut CaptureState) -> Result<()> {
            for _ in 0..MAX_PACKETS_PER_WAKE {
                let waiting = self
                    .capture
                    .get_next_packet_size()
                    .ctx("failed to query loopback packet size")?;
                // `None` only in exclusive mode, which loopback never opens.
                if waiting.unwrap_or(0) == 0 {
                    break;
                }
                let info = self
                    .capture
                    .read_from_device_to_deque(&mut self.raw)
                    .ctx("failed to read loopback buffer")?;
                if info.flags.data_discontinuity {
                    if let Some(earlier) = self.discontinuities.note(Instant::now()) {
                        tracing::warn!(
                            "loopback capture lost audio (data discontinuity; {earlier} more \
                             since the last report)"
                        );
                    }
                }
                self.forward_packet(info.flags.silent, state);
            }
            Ok(())
        }

        /// Decode the packet just read into `raw` and feed it to `state`.
        fn forward_packet(&mut self, silent: bool, state: &mut CaptureState) {
            if self.raw.is_empty() {
                return;
            }
            zero_if_silent(&mut self.raw, silent);
            self.frame.clear();
            decode_interleaved(&mut self.raw, self.sample_type, self.bits, &mut self.frame);
            self.raw.clear();
            state.push_samples(&self.frame, self.channels);
        }
    }

    /// A packet flagged `AUDCLNT_BUFFERFLAGS_SILENT` is to be read as silence whatever its
    /// buffer holds: Windows does not promise to have written the samples. All-zero bytes are
    /// silence in every format `decode_interleaved` reads, the float one included.
    fn zero_if_silent(raw: &mut VecDeque<u8>, silent: bool) {
        if silent {
            raw.iter_mut().for_each(|byte| *byte = 0);
        }
    }

    /// How often a data discontinuity is reported. One means audio was lost, which matters
    /// when someone asks why the captions skipped; a starved thread can produce one per
    /// packet, which would bury everything else in the log.
    const DISCONTINUITY_REPORT_INTERVAL: Duration = Duration::from_secs(10);

    /// Rate-limits the discontinuity report.
    #[derive(Debug, Default)]
    struct Discontinuities {
        last_report: Option<Instant>,
        /// Seen since the last report, and not reported yet.
        unreported: u32,
    }

    impl Discontinuities {
        /// Record one at `now`. When it is time to report, how many came between the last
        /// report and this one (zero for the first).
        fn note(&mut self, now: Instant) -> Option<u32> {
            let due = self.last_report.is_none_or(|last| {
                now.saturating_duration_since(last) >= DISCONTINUITY_REPORT_INTERVAL
            });
            if !due {
                self.unreported = self.unreported.saturating_add(1);
                return None;
            }
            self.last_report = Some(now);
            Some(std::mem::take(&mut self.unreported))
        }
    }

    /// Decode raw interleaved endpoint bytes into f32 samples in [-1, 1].
    /// `make_contiguous` rotates the deque in place instead of copying it out.
    fn decode_interleaved(bytes: &mut VecDeque<u8>, ty: SampleType, bits: u16, out: &mut Vec<f32>) {
        let buf: &[u8] = bytes.make_contiguous();
        match (ty, bits) {
            (SampleType::Float, 32) => {
                for c in buf.as_chunks::<4>().0 {
                    out.push(f32::from_le_bytes(*c));
                }
            }
            (SampleType::Int, 16) => {
                for c in buf.as_chunks::<2>().0 {
                    out.push(f32::from(i16::from_le_bytes(*c)) / 32768.0);
                }
            }
            (SampleType::Int, 32) => {
                for c in buf.as_chunks::<4>().0 {
                    let v = i32::from_le_bytes(*c);
                    out.push(v as f32 / 2_147_483_648.0);
                }
            }
            (_, b) => {
                // Unexpected format: emit silence of the right length rather than panic.
                let bytes_per = (b / 8).max(1) as usize;
                for _ in 0..(buf.len() / bytes_per) {
                    out.push(0.0);
                }
                tracing::warn!("unsupported loopback sample format: {ty:?}/{b}-bit");
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn decode(bytes: &[u8], ty: SampleType, bits: u16) -> Vec<f32> {
            let mut raw: VecDeque<u8> = bytes.iter().copied().collect();
            let mut out = Vec::new();
            decode_interleaved(&mut raw, ty, bits, &mut out);
            out
        }

        fn bytes<const N: usize>(samples: impl IntoIterator<Item = [u8; N]>) -> Vec<u8> {
            samples.into_iter().flatten().collect()
        }

        #[test]
        fn float_samples_are_read_as_they_are() {
            let input = [0.0_f32, 0.5, -1.0, 1.0, -0.25];
            let raw = bytes(input.map(f32::to_le_bytes));
            assert_eq!(decode(&raw, SampleType::Float, 32), input);
        }

        #[test]
        fn integer_samples_are_scaled_into_the_unit_range() {
            for (raw, bits, expected) in [
                (
                    bytes([0_i16, 16384, -32768, 32767].map(i16::to_le_bytes)),
                    16,
                    vec![0.0, 0.5, -1.0, 32767.0 / 32768.0],
                ),
                (
                    bytes([0_i32, 1 << 30, i32::MIN].map(i32::to_le_bytes)),
                    32,
                    vec![0.0, 0.5, -1.0],
                ),
            ] {
                assert_eq!(decode(&raw, SampleType::Int, bits), expected, "{bits}-bit");
            }
            // The largest i32 is not quite 1.0, but f32 cannot tell.
            let top = decode(&i32::MAX.to_le_bytes(), SampleType::Int, 32);
            assert!((top[0] - 1.0).abs() < 1e-6, "{top:?}");
        }

        /// A trailing partial sample is dropped rather than read past.
        #[test]
        fn a_partial_sample_is_left_out() {
            let mut raw = bytes([0.5_f32].map(f32::to_le_bytes));
            raw.extend([1, 2]);
            assert_eq!(decode(&raw, SampleType::Float, 32), [0.5]);
            assert_eq!(decode(&[0, 64, 7], SampleType::Int, 16), [0.5]);
        }

        /// An unexpected format gives silence of the right length, never a panic.
        #[test]
        fn an_unknown_format_decodes_to_silence_of_the_same_length() {
            for (ty, bits, len, samples) in [
                (SampleType::Int, 24, 12, 4),
                (SampleType::Float, 64, 16, 2),
                (SampleType::Int, 8, 5, 5),
                // A zero bit depth counts as one byte per sample rather than dividing by zero.
                (SampleType::Int, 0, 3, 3),
            ] {
                assert_eq!(
                    decode(&vec![0x7f; len], ty, bits),
                    vec![0.0; samples],
                    "{ty:?}/{bits}"
                );
            }
        }

        #[test]
        fn a_silent_packet_is_zeroed_whatever_its_buffer_holds() {
            let loud = bytes([1.0_f32, -1.0].map(f32::to_le_bytes));
            let mut raw: VecDeque<u8> = loud.iter().copied().collect();
            zero_if_silent(&mut raw, true);
            assert_eq!(raw.len(), loud.len(), "the packet keeps its length");
            let mut out = Vec::new();
            decode_interleaved(&mut raw, SampleType::Float, 32, &mut out);
            assert_eq!(out, [0.0, 0.0]);

            // A packet that is not flagged is left alone.
            let mut raw: VecDeque<u8> = loud.iter().copied().collect();
            zero_if_silent(&mut raw, false);
            assert_eq!(raw.iter().copied().collect::<Vec<_>>(), loud);
        }

        #[test]
        fn discontinuities_are_reported_at_most_once_per_interval_with_a_count() {
            let start = Instant::now();
            let at = |ms: u64| start + Duration::from_millis(ms);
            let mut seen = Discontinuities::default();
            // The first is reported at once.
            assert_eq!(seen.note(at(0)), Some(0));
            // A starved thread's burst is counted, not logged.
            for ms in [10, 20, 30, 9_999] {
                assert_eq!(seen.note(at(ms)), None);
            }
            // The next report carries the count, which then starts again.
            assert_eq!(seen.note(at(10_000)), Some(4));
            assert_eq!(seen.note(at(10_001)), None);
            assert_eq!(seen.note(at(25_000)), Some(1));
        }
    }
}
