/// Fixed-capacity bridge between arbitrary host blocks and fixed-size model frames.
///
/// The model output is delayed by one model frame so every host callback can return a
/// deterministic sample without waiting for a complete frame. The dry signal uses the
/// full adapter + model latency, keeping partial wet mixes phase aligned.
pub(crate) struct StreamState {
    channels: usize,
    hop_size: usize,
    latency_samples: usize,
    input_frame: Vec<f32>,
    output_frame: Vec<f32>,
    hop_fill: usize,
    wet_ring: Vec<f32>,
    wet_valid: Vec<bool>,
    wet_position: usize,
    dry_delay: Vec<f32>,
    dry_position: usize,
}

impl StreamState {
    pub(crate) fn new(
        channels: usize,
        hop_size: usize,
        model_latency: usize,
    ) -> Result<Self, &'static str> {
        if channels == 0 || hop_size == 0 {
            return Err("channels and hop size must be non-zero");
        }

        let latency_samples = model_latency
            .checked_add(hop_size)
            .ok_or("latency overflow")?;
        let frame_len = channels
            .checked_mul(hop_size)
            .ok_or("frame size overflow")?;
        let delay_len = channels
            .checked_mul(latency_samples)
            .ok_or("delay size overflow")?;

        Ok(Self {
            channels,
            hop_size,
            latency_samples,
            input_frame: vec![0.0; frame_len],
            output_frame: vec![0.0; frame_len],
            hop_fill: 0,
            wet_ring: vec![0.0; frame_len],
            wet_valid: vec![false; hop_size],
            wet_position: 0,
            dry_delay: vec![0.0; delay_len],
            dry_position: 0,
        })
    }

    pub(crate) fn channels(&self) -> usize {
        self.channels
    }

    pub(crate) fn hop_size(&self) -> usize {
        self.hop_size
    }

    pub(crate) fn latency_samples(&self) -> usize {
        self.latency_samples
    }

    /// Capture one channel from the current sample frame and return aligned dry/wet values.
    pub(crate) fn capture_channel(&mut self, channel: usize, input: f32) -> (f32, f32) {
        debug_assert!(channel < self.channels);

        self.input_frame[channel * self.hop_size + self.hop_fill] = input;

        let dry_index = channel * self.latency_samples + self.dry_position;
        let delayed_dry = self.dry_delay[dry_index];
        self.dry_delay[dry_index] = input;

        let wet_index = channel * self.hop_size + self.wet_position;
        let wet = if self.wet_valid[self.wet_position] {
            self.wet_ring[wet_index]
        } else {
            delayed_dry
        };

        (delayed_dry, wet)
    }

    /// Advance one complete sample frame. Returns true when a model frame is ready.
    pub(crate) fn advance_sample(&mut self) -> bool {
        self.hop_fill += 1;
        self.wet_position = (self.wet_position + 1) % self.hop_size;
        self.dry_position = (self.dry_position + 1) % self.latency_samples;

        if self.hop_fill == self.hop_size {
            self.hop_fill = 0;
            true
        } else {
            false
        }
    }

    pub(crate) fn frame_buffers(&mut self) -> (&[f32], &mut [f32]) {
        (&self.input_frame, &mut self.output_frame)
    }

    /// Publish a complete model frame. Failed frames fall back to the aligned dry signal.
    pub(crate) fn publish_model_output(&mut self, valid: bool) {
        for time in 0..self.hop_size {
            let ring_time = (self.wet_position + time) % self.hop_size;
            self.wet_valid[ring_time] = valid;
            for channel in 0..self.channels {
                self.wet_ring[channel * self.hop_size + ring_time] =
                    self.output_frame[channel * self.hop_size + time];
            }
        }
    }

    pub(crate) fn reset(&mut self) {
        self.input_frame.fill(0.0);
        self.output_frame.fill(0.0);
        self.wet_ring.fill(0.0);
        self.wet_valid.fill(false);
        self.dry_delay.fill(0.0);
        self.hop_fill = 0;
        self.wet_position = 0;
        self.dry_position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::StreamState;

    fn run_identity(channels: usize, hop: usize, chunks: &[usize], input: &[f32]) -> Vec<f32> {
        let mut state = StreamState::new(channels, hop, 0).unwrap();
        let mut output = Vec::with_capacity(input.len());
        let mut frame_offset = 0;

        for &chunk in chunks {
            for _ in 0..chunk {
                for channel in 0..channels {
                    let sample = input[(frame_offset * channels) + channel];
                    let (dry, wet) = state.capture_channel(channel, sample);
                    debug_assert_eq!(dry, wet);
                    output.push(wet);
                }

                if state.advance_sample() {
                    let (model_input, model_output) = state.frame_buffers();
                    model_output.copy_from_slice(model_input);
                    state.publish_model_output(true);
                }
                frame_offset += 1;
            }
        }

        assert_eq!(frame_offset * channels, input.len());
        output
    }

    #[test]
    fn chunking_does_not_change_the_stream() {
        let frames = 4096;
        let input: Vec<f32> = (0..frames).map(|sample| sample as f32 + 1.0).collect();
        let reference = run_identity(1, 480, &[frames], &input);

        for chunk_size in [64, 128, 256, 480, 512, 1024] {
            let mut chunks = vec![chunk_size; frames / chunk_size];
            let remainder = frames % chunk_size;
            if remainder > 0 {
                chunks.push(remainder);
            }
            assert_eq!(run_identity(1, 480, &chunks, &input), reference);
        }

        assert!(reference[..480].iter().all(|sample| *sample == 0.0));
        assert_eq!(&reference[480..], &input[..frames - 480]);
    }

    #[test]
    fn stereo_channels_remain_separate() {
        let frames = 1200;
        let mut input = Vec::with_capacity(frames * 2);
        for frame in 0..frames {
            input.push(frame as f32 + 1.0);
            input.push(-(frame as f32 + 1.0));
        }

        let output = run_identity(2, 480, &[128, 512, 64, 496], &input);
        for frame in 480..frames {
            assert_eq!(output[frame * 2], input[(frame - 480) * 2]);
            assert_eq!(output[frame * 2 + 1], input[(frame - 480) * 2 + 1]);
        }
    }

    #[test]
    fn failed_model_frame_falls_back_to_aligned_dry() {
        let mut state = StreamState::new(1, 4, 0).unwrap();
        let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mut output = Vec::new();

        for sample in input {
            let (_, wet) = state.capture_channel(0, sample);
            output.push(wet);
            if state.advance_sample() {
                state.publish_model_output(false);
            }
        }

        assert_eq!(output, vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn reset_discards_all_previous_segment_state() {
        let mut state = StreamState::new(1, 4, 3).unwrap();
        for sample in [1.0, 2.0, 3.0, 4.0] {
            state.capture_channel(0, sample);
            if state.advance_sample() {
                let (model_input, model_output) = state.frame_buffers();
                model_output.copy_from_slice(model_input);
                state.publish_model_output(true);
            }
        }

        state.reset();
        for _ in 0..state.latency_samples() {
            let (dry, wet) = state.capture_channel(0, 0.0);
            assert_eq!((dry, wet), (0.0, 0.0));
            if state.advance_sample() {
                state.publish_model_output(false);
            }
        }
    }

    #[test]
    fn rejects_invalid_or_overflowing_sizes() {
        assert!(StreamState::new(0, 480, 0).is_err());
        assert!(StreamState::new(1, 0, 0).is_err());
        assert!(StreamState::new(usize::MAX, 2, 0).is_err());
        assert!(StreamState::new(2, 1, usize::MAX).is_err());
    }
}
