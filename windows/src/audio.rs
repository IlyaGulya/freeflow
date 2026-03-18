//! WASAPI audio capture via CPAL.
//!
//! `AudioCapture::start()` opens the default input device at 16 kHz mono,
//! streams f32 samples through an `mpsc` channel, and returns an
//! `AudioCapture` handle.  Calling `.stop()` on the handle signals the
//! capture thread to finish and returns all collected samples.
//!
//! Platform guard: the real implementation is compiled only on Windows.
//! On other hosts a no-op stub is provided so `cargo check` succeeds.

// The UnboundedSender type is re-exported for use by callers on all platforms.
#[allow(unused_imports)]
use tokio::sync::mpsc::UnboundedSender;

// ---------------------------------------------------------------------------
// Windows real implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod platform {
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicBool, Ordering};

    use anyhow::{Context, Result};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{SampleRate, SupportedStreamConfigRange, BufferSize};
    use tokio::sync::mpsc::UnboundedSender;
    use log::{info, warn, error};

    use wrenflow_core::audio::resample_to_16khz;

    /// Target capture sample rate.  WASAPI honours this when the device
    /// supports it; otherwise CPAL selects the nearest supported rate and
    /// we resample to 16 kHz in software.
    const PREFERRED_SAMPLE_RATE: u32 = 16_000;
    const CHANNELS: u16 = 1;

    pub struct AudioCapture {
        // Keeps the CPAL stream alive until we explicitly drop it.
        _stream: cpal::Stream,
        stop_flag: Arc<AtomicBool>,
        samples: Arc<Mutex<Vec<f32>>>,
        // Device sample rate so we can resample if needed.
        device_sample_rate: u32,
    }

    impl AudioCapture {
        /// Open the default WASAPI input device and start streaming.
        ///
        /// Samples are sent to `tx` as they arrive and also accumulated
        /// internally so that `stop()` can return the full recording.
        pub fn start(tx: UnboundedSender<Vec<f32>>) -> Result<Self> {
            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .context("no default input device available")?;

            info!("audio device: {}", device.name().unwrap_or_default());

            // Find the best supported config: prefer 16kHz mono, fall back to
            // 48kHz or whatever the device offers.
            let config = pick_best_config(&device)?;
            let device_sample_rate = config.sample_rate().0;
            info!(
                "capturing at {}Hz, {} ch",
                device_sample_rate,
                config.channels()
            );

            let stop_flag = Arc::new(AtomicBool::new(false));
            let samples = Arc::new(Mutex::new(Vec::<f32>::new()));

            let samples_cb = samples.clone();
            let stop_cb = stop_flag.clone();

            let stream = device.build_input_stream(
                &config.into(),
                move |data: &[f32], _info: &cpal::InputCallbackInfo| {
                    if stop_cb.load(Ordering::Relaxed) {
                        return;
                    }
                    // Mix to mono if device is stereo
                    let mono: Vec<f32> = match config.channels() {
                        1 => data.to_vec(),
                        ch => {
                            let ch = ch as usize;
                            data.chunks_exact(ch)
                                .map(|frame| frame.iter().sum::<f32>() / ch as f32)
                                .collect()
                        }
                    };
                    // Accumulate
                    samples_cb.lock().unwrap().extend_from_slice(&mono);
                    // Stream to caller (best-effort, ignore send errors)
                    let _ = tx.send(mono);
                },
                |err| error!("cpal stream error: {err}"),
                None, // latency hint
            )?;

            stream.play().context("failed to start audio stream")?;

            Ok(AudioCapture {
                _stream: stream,
                stop_flag,
                samples,
                device_sample_rate,
            })
        }

        /// Stop the stream and return all accumulated samples resampled to 16 kHz.
        pub fn stop(self) -> Vec<f32> {
            self.stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
            // _stream is dropped here, which stops the CPAL stream.
            let raw = self.samples.lock().unwrap().clone();

            if self.device_sample_rate == PREFERRED_SAMPLE_RATE {
                raw
            } else {
                warn!(
                    "resampling {} Hz → 16 kHz ({} samples)",
                    self.device_sample_rate,
                    raw.len()
                );
                resample_to_16khz(&raw, self.device_sample_rate, PREFERRED_SAMPLE_RATE)
            }
        }
    }

    fn pick_best_config(device: &cpal::Device) -> Result<cpal::SupportedStreamConfig> {
        let mut configs: Vec<SupportedStreamConfigRange> = device
            .supported_input_configs()
            .context("could not enumerate input configs")?
            .collect();

        // Sort: prefer fewer channels, then prefer sample rates closest to 16kHz
        configs.sort_by_key(|c| {
            let rate_dist = (c.min_sample_rate().0 as i32 - PREFERRED_SAMPLE_RATE as i32).unsigned_abs();
            (c.channels(), rate_dist)
        });

        if let Some(range) = configs.first() {
            // Use the range's min rate; clamp to preferred if in range
            let rate = PREFERRED_SAMPLE_RATE
                .clamp(range.min_sample_rate().0, range.max_sample_rate().0);
            Ok(range.clone().with_sample_rate(cpal::SampleRate(rate)))
        } else {
            // Absolute fallback: use device default config
            device
                .default_input_config()
                .context("no input configs available")
        }
    }
}

// ---------------------------------------------------------------------------
// Non-Windows stub
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
mod platform {
    use anyhow::Result;
    use tokio::sync::mpsc::UnboundedSender;

    #[allow(dead_code)]
    pub struct AudioCapture;

    #[allow(dead_code)]
    impl AudioCapture {
        pub fn start(_tx: UnboundedSender<Vec<f32>>) -> Result<Self> {
            Ok(AudioCapture)
        }

        pub fn stop(self) -> Vec<f32> {
            Vec::new()
        }
    }
}

// Re-export the platform implementation under the module's public name.
#[allow(unused_imports)]
pub use platform::AudioCapture;
