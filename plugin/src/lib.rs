mod stream;

use std::sync::Arc;

use df::tract::{DfParams, DfTract, RuntimeParams};
use ndarray::{ArrayView2, ArrayViewMut2};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};

use stream::StreamState;

/// Exclusive owner for tract's non-`Send` working and pristine inference states.
///
/// SAFETY: `DfTract` is non-`Send` because it contains `Rc<Tensor>` and erased operation state.
/// Cloning can leave internal `Rc` aliases between these two models, so the wrapper keeps them in
/// one move unit. It never exposes either model outside the plugin, access is exclusive through
/// `&mut self`, and it deliberately does not implement `Sync`.
struct ExclusiveModels {
    current: DfTract,
    pristine: DfTract,
}

unsafe impl Send for ExclusiveModels {}

struct DeepFilterPlugin {
    params: Arc<DeepFilterParams>,
    models: Option<ExclusiveModels>,
    stream: Option<StreamState>,
    skip_next_model_reset: bool,
    model_failures: u64,
}

#[derive(Params)]
struct DeepFilterParams {
    #[id = "input_trim"]
    pub input_trim: FloatParam,

    #[id = "atten_lim"]
    pub atten_lim: FloatParam,

    #[id = "mix"]
    pub mix: FloatParam,

    #[id = "output_gain"]
    pub output_gain: FloatParam,

    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,
}

impl Default for DeepFilterParams {
    fn default() -> Self {
        Self {
            input_trim: FloatParam::new(
                "Input Trim",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_smoother(SmoothingStyle::Linear(50.0)),

            atten_lim: FloatParam::new(
                "Attenuation Limit",
                100.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit(" dB")
            .with_smoother(SmoothingStyle::Linear(50.0)),

            mix: FloatParam::new("Mix", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),

            output_gain: FloatParam::new(
                "Output Gain",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_smoother(SmoothingStyle::Linear(50.0)),

            editor_state: EguiState::from_size(400, 300),
        }
    }
}

impl Default for DeepFilterPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DeepFilterParams::default()),
            models: None,
            stream: None,
            skip_next_model_reset: false,
            model_failures: 0,
        }
    }
}

impl Plugin for DeepFilterPlugin {
    const NAME: &'static str = "DeepFilter Noise Reduction";
    const VENDOR: &'static str = "DeepFilterNet";
    const URL: &'static str = "https://github.com/Rikorose/DeepFilterNet";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                let params = &params;

                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.label("DeepFilterNet3 Implementation");
                    ui.separator();

                    ui.label("Input Stage");
                    ui.label("Input Trim");
                    ui.add(widgets::ParamSlider::for_param(&params.input_trim, setter));

                    ui.separator();
                    ui.label("Processing");
                    ui.label("Attenuation Limit");
                    ui.add(widgets::ParamSlider::for_param(&params.atten_lim, setter));
                    ui.label("Mix");
                    ui.add(widgets::ParamSlider::for_param(&params.mix, setter));

                    ui.separator();
                    ui.label("Output Stage");
                    ui.label("Output Gain");
                    ui.add(widgets::ParamSlider::for_param(&params.output_gain, setter));
                });
            },
        )
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        self.clear_model();

        if (buffer_config.sample_rate - 48_000.0).abs() > 1.0 {
            nih_log!(
                "DeepFilterNet requires 48kHz. Current: {}Hz",
                buffer_config.sample_rate
            );
            return false;
        }

        let channels = audio_io_layout
            .main_input_channels
            .map(|count| count.get() as usize)
            .unwrap_or(1);

        match self.initialize_model(channels) {
            Ok(latency_samples) => {
                context.set_latency_samples(latency_samples);
                nih_log!(
                    "DeepFilterNet initialized. channels={}, latency={} samples",
                    channels,
                    latency_samples
                );
                true
            }
            Err(error) => {
                self.clear_model();
                nih_log!("Failed to init DeepFilterNet: {:?}", error);
                false
            }
        }
    }

    fn reset(&mut self) {
        if self.skip_next_model_reset {
            // NIH-plug always calls reset immediately after initialize(). That model is already
            // pristine, so avoid cloning the large inference state on this first reset.
            self.skip_next_model_reset = false;
        } else if let Some(models) = self.models.as_mut() {
            models.current.clone_from(&models.pristine);
        }

        if let Some(stream) = self.stream.as_mut() {
            stream.reset();
        }
        self.params
            .input_trim
            .smoothed
            .reset(self.params.input_trim.value());
        self.params
            .atten_lim
            .smoothed
            .reset(self.params.atten_lim.value());
        self.params.mix.smoothed.reset(self.params.mix.value());
        self.params
            .output_gain
            .smoothed
            .reset(self.params.output_gain.value());
        self.model_failures = 0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let (Some(models), Some(stream)) = (self.models.as_mut(), self.stream.as_mut()) else {
            return ProcessStatus::Normal;
        };

        debug_assert_eq!(buffer.channels(), stream.channels());
        let channels = stream.channels();
        let hop_size = stream.hop_size();
        let params = &self.params;

        for channel_samples in buffer.iter_samples() {
            let mix = params.mix.smoothed.next();
            let input_gain = db_to_gain(params.input_trim.smoothed.next());
            let output_gain = db_to_gain(params.output_gain.smoothed.next());

            for (channel, sample) in channel_samples.into_iter().enumerate() {
                let scaled_input = *sample * input_gain;
                let (dry, wet) = stream.capture_channel(channel, scaled_input);
                *sample = mix_sample(dry, wet, mix) * output_gain;
            }

            if stream.advance_sample() {
                let attenuation = params.atten_lim.smoothed.next_step(hop_size as u32);
                models.current.set_atten_lim(attenuation);

                let model_ok = {
                    let (input_frame, output_frame) = stream.frame_buffers();
                    let input = ArrayView2::from_shape((channels, hop_size), input_frame);
                    let output = ArrayViewMut2::from_shape((channels, hop_size), output_frame);
                    match (input, output) {
                        (Ok(input), Ok(output)) => models.current.process(input, output).is_ok(),
                        _ => false,
                    }
                };

                if !model_ok {
                    self.model_failures = self.model_failures.saturating_add(1);
                }
                stream.publish_model_output(model_ok);
            }
        }

        ProcessStatus::Normal
    }
}

impl DeepFilterPlugin {
    fn initialize_model(&mut self, channels: usize) -> Result<u32, Box<dyn std::error::Error>> {
        let runtime_params = RuntimeParams::default_with_ch(channels);
        let model = DfTract::new(DfParams::default(), &runtime_params)?;
        let model_latency =
            calculate_model_latency(model.fft_size, model.hop_size, model.lookahead)
                .ok_or_else(|| std::io::Error::other("model latency overflow"))?;
        let stream = StreamState::new(channels, model.hop_size, model_latency)
            .map_err(std::io::Error::other)?;
        let total_latency = u32::try_from(stream.latency_samples())
            .map_err(|_| std::io::Error::other("plugin latency exceeds u32"))?;

        self.models = Some(ExclusiveModels {
            pristine: model.clone(),
            current: model,
        });
        self.stream = Some(stream);
        self.skip_next_model_reset = true;
        self.model_failures = 0;

        Ok(total_latency)
    }

    fn clear_model(&mut self) {
        self.models = None;
        self.stream = None;
        self.skip_next_model_reset = false;
        self.model_failures = 0;
    }
}

fn calculate_model_latency(fft_size: usize, hop_size: usize, lookahead: usize) -> Option<usize> {
    fft_size
        .checked_sub(hop_size)?
        .checked_add(lookahead.checked_mul(hop_size)?)
}

fn db_to_gain(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

fn mix_sample(dry: f32, wet: f32, mix: f32) -> f32 {
    dry * (1.0 - mix) + wet * mix
}

impl ClapPlugin for DeepFilterPlugin {
    const CLAP_ID: &'static str = "com.deepfilter.noise-reduction";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Noise reduction using DeepFilterNet3");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for DeepFilterPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"DeepFilterNR001\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Restoration];
}

nih_export_clap!(DeepFilterPlugin);
nih_export_vst3!(DeepFilterPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_calculation_matches_decibels() {
        assert!((db_to_gain(6.0) - 1.995).abs() < 0.01);
        assert!((db_to_gain(-6.0) - 0.501).abs() < 0.01);
        assert!((db_to_gain(20.0) - 10.0).abs() < 0.01);
    }

    #[test]
    fn dry_wet_mix_endpoints_and_midpoint_are_correct() {
        assert_eq!(mix_sample(1.0, -1.0, 0.0), 1.0);
        assert_eq!(mix_sample(1.0, -1.0, 0.5), 0.0);
        assert_eq!(mix_sample(1.0, -1.0, 1.0), -1.0);
    }

    #[test]
    fn model_latency_uses_stft_and_lookahead() {
        assert_eq!(calculate_model_latency(960, 480, 2), Some(1440));
        assert_eq!(calculate_model_latency(480, 960, 2), None);
        assert_eq!(calculate_model_latency(usize::MAX, 1, 2), None);
    }

    #[test]
    fn plugin_type_is_send_with_exclusive_model_boundary() {
        fn assert_send<T: Send>() {}
        assert_send::<DeepFilterPlugin>();
    }
}
