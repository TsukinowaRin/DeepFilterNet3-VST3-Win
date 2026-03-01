use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState, widgets};
use std::sync::{Arc, Mutex};


use df::tract::{DfParams, DfTract, RuntimeParams};
use ndarray::Array2;

// DfTract を Mutex でラップしてスレッドセーフに
struct DfWrapper(Mutex<Option<DfTract>>);

// Send と Sync を手動で実装（Mutex で保護されているため安全）
unsafe impl Send for DfWrapper {}
unsafe impl Sync for DfWrapper {}

struct DeepFilterPlugin {
    params: Arc<DeepFilterParams>,
    df_model: DfWrapper,
    input_buffer: Mutex<Vec<f32>>,
    output_buffer: Mutex<Vec<f32>>,
    hop_size: usize,
    is_initialized: bool,
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
                FloatRange::Linear { min: -24.0, max: 24.0 },
            )
            .with_unit(" dB")
            .with_smoother(SmoothingStyle::Linear(50.0)),

            atten_lim: FloatParam::new(
                "Attenuation Limit",
                100.0,
                FloatRange::Linear { min: 0.0, max: 100.0 },
            )
            .with_unit(" dB")
            .with_smoother(SmoothingStyle::Linear(50.0)),

            mix: FloatParam::new(
                "Mix",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            output_gain: FloatParam::new(
                "Output Gain",
                0.0,
                FloatRange::Linear { min: -24.0, max: 24.0 },
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
            df_model: DfWrapper(Mutex::new(None)),
            input_buffer: Mutex::new(Vec::new()),
            output_buffer: Mutex::new(Vec::new()),
            hop_size: 480,
            is_initialized: false,
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
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // DeepFilterNet は 48kHz のみサポート
        if (buffer_config.sample_rate - 48000.0).abs() > 1.0 {
            nih_log!("DeepFilterNet requires 48kHz. Current: {}Hz", buffer_config.sample_rate);
            return false;
        }

        let num_channels = audio_io_layout
            .main_input_channels
            .map(|c| c.get() as usize)
            .unwrap_or(1);

        match self.init_model(num_channels) {
            Ok(hop) => {
                self.hop_size = hop;
                self.is_initialized = true;
                nih_log!("DeepFilterNet initialized. hop_size={}", hop);
                true
            }
            Err(e) => {
                nih_log!("Failed to init DeepFilterNet: {:?}", e);
                false
            }
        }
    }

    fn reset(&mut self) {
        if let Ok(mut buf) = self.input_buffer.lock() {
            buf.clear();
        }
        if let Ok(mut buf) = self.output_buffer.lock() {
            buf.clear();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if !self.is_initialized {
            return ProcessStatus::Normal;
        }

        let mix = self.params.mix.smoothed.next();
        let atten = self.params.atten_lim.smoothed.next();
        let input_gain_db = self.params.input_trim.smoothed.next();
        let output_gain_db = self.params.output_gain.smoothed.next();
        
        let input_gain = 10.0f32.powf(input_gain_db / 20.0);
        let output_gain = 10.0f32.powf(output_gain_db / 20.0);

        let num_samples = buffer.samples();
        let num_channels = buffer.channels();
        let hop = self.hop_size;

        // 入力収集（インターリーブで保存）
        {
            let mut input_buf = self.input_buffer.lock().unwrap();
            for i in 0..num_samples {
                for channel in buffer.iter_samples().nth(i).unwrap() {
                     input_buf.push(*channel * input_gain);
                }
            }
        }


        // DeepFilterNet でフレーム処理
        {
            let mut input_buf = self.input_buffer.lock().unwrap();
            let mut output_buf = self.output_buffer.lock().unwrap();
            let mut model_guard = self.df_model.0.lock().unwrap();

            if let Some(ref mut df_model) = *model_guard {
                df_model.set_atten_lim(atten);

                // input_buf は [sample1_ch1, sample1_ch2, ..., sampleN_ch1, sampleN_ch2] のようにインターリーブされている前提
                // 1フレームに必要なサンプル数 = hop * num_channels
                let required_samples = hop * num_channels;

                while input_buf.len() >= required_samples {
                    // (channels, hop) の形状を作成
                    let mut in_frame = Array2::zeros((num_channels, hop));
                    let mut out_frame = Array2::zeros((num_channels, hop));

                    // input_buf (Interleaved) -> in_frame (Planar: ch, time)
                    for t in 0..hop {
                        for ch in 0..num_channels {
                            in_frame[[ch, t]] = input_buf[t * num_channels + ch];
                        }
                    }

                    match df_model.process(in_frame.view(), out_frame.view_mut()) {
                        Ok(_) => {
                            // out_frame (Planar) -> output_buf (Interleaved)
                            for t in 0..hop {
                                for ch in 0..num_channels {
                                    output_buf.push(out_frame[[ch, t]]);
                                }
                            }
                        }
                        Err(_) => {
                            // エラー時は入力をバイパス
                            output_buf.extend_from_slice(&input_buf[..required_samples]);
                        }
                    }

                    input_buf.drain(..required_samples);
                }
            }
        }


        // 出力書き込み
        {
            let mut output_buf = self.output_buffer.lock().unwrap();
            // output_buf もインターリーブされている
            let required_out_samples = num_samples * num_channels;

            if output_buf.len() >= required_out_samples {
                for (sample_idx, channel_samples) in buffer.iter_samples().enumerate() {
                    // sample_idx は時間のインデックス
                    // output_buf は [t0_c0, t0_c1, t1_c0, t1_c1, ...]
                    
                    let mut ch_idx = 0;
                    for sample in channel_samples {
                        let processed = output_buf[sample_idx * num_channels + ch_idx];
                        let dry = *sample * input_gain; 
                        *sample = (dry * (1.0 - mix) + processed * mix) * output_gain;
                        ch_idx += 1;
                    }
                }
                output_buf.drain(..required_out_samples);
            }

        }

        ProcessStatus::Normal
    }
}

impl DeepFilterPlugin {
    fn init_model(&mut self, channels: usize) -> Result<usize, Box<dyn std::error::Error>> {
        let df_params = DfParams::default();
        let rt_params = RuntimeParams::default_with_ch(channels);
        let df = DfTract::new(df_params, &rt_params)?;
        let hop = df.hop_size;

        *self.df_model.0.lock().unwrap() = Some(df);
        // バッファサイズは (hop * channels) を考慮して少し多めに確保
        let buf_capacity = hop * channels * 4;
        *self.input_buffer.lock().unwrap() = Vec::with_capacity(buf_capacity);
        *self.output_buffer.lock().unwrap() = Vec::with_capacity(buf_capacity);


        Ok(hop)
    }
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
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Restoration,
    ];
}

nih_export_clap!(DeepFilterPlugin);
nih_export_vst3!(DeepFilterPlugin);

#[cfg(test)]
mod tests {
    use super::*;
    use nih_plug::prelude::*;

    #[test]
    fn test_input_trim_gain_calculation() {
        // Test +6dB
        let mut params = DeepFilterParams::default();
        params.input_trim = FloatParam::new(
            "Input Trim",
            6.0,
            FloatRange::Linear { min: -24.0, max: 24.0 },
        );
        let gain = 10.0f32.powf(params.input_trim.value() / 20.0);
        assert!((gain - 1.995).abs() < 0.01, "Expected approx 2.0 for +6dB, got {}", gain);

        // Test -6dB
        params.input_trim = FloatParam::new(
            "Input Trim",
            -6.0,
            FloatRange::Linear { min: -24.0, max: 24.0 },
        );
        let gain = 10.0f32.powf(params.input_trim.value() / 20.0);
        assert!((gain - 0.501).abs() < 0.01, "Expected approx 0.5 for -6dB, got {}", gain);
    }

    #[test]
    fn test_output_gain_calculation() {
        let mut params = DeepFilterParams::default();
        // Test +20dB
        params.output_gain = FloatParam::new(
            "Output Gain",
            20.0,
            FloatRange::Linear { min: -24.0, max: 24.0 },
        );
        let gain = 10.0f32.powf(params.output_gain.value() / 20.0);
        assert!((gain - 10.0).abs() < 0.01, "Expected 10.0 for +20dB, got {}", gain);
    }

    #[test]
    fn test_mix_calculation() {
        let mut params = DeepFilterParams::default();
        
        // Test 50% Mix
        params.mix = FloatParam::new(
            "Mix",
            0.5,
            FloatRange::Linear { min: 0.0, max: 1.0 },
        );
        let mix = params.mix.value();
        assert!((mix - 0.5).abs() < 0.001);

        // Logic check: dry * (1.0 - mix) + wet * mix
        let dry = 1.0;
        let wet = 0.0;
        let out = dry * (1.0 - mix) + wet * mix;
        assert!((out - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_stereo_separation() {
        // This test verifies the interleaving/de-interleaving logic used in process()
        // to ensure it supports multi-channel processing without mono mixdown.
        // Note: We are testing the logic logic because constructing a full nih_plug::Buffer
        // without a host is complex.
        
        let num_channels = 2;
        let hop = 480;
        
        // Simulate Input: Interleaved [L, R, L, R ...]
        let mut input_interleaved = Vec::new();
        for _ in 0..hop {
            input_interleaved.push(1.0); // L = 1.0
            input_interleaved.push(0.0); // R = 0.0
        }
        
        // Logic from `process` (De-interleave)
        let mut in_frame = Array2::zeros((num_channels, hop));
        for t in 0..hop {
            for ch in 0..num_channels {
                in_frame[[ch, t]] = input_interleaved[t * num_channels + ch];
            }
        }
        
        // Verify De-interleave
        for t in 0..hop {
            assert_eq!(in_frame[[0, t]], 1.0, "Left channel should be 1.0");
            assert_eq!(in_frame[[1, t]], 0.0, "Right channel should be 0.0");
        }
        
        // Simulate Process (Identity for test)
        let mut out_frame = in_frame.clone(); // Pass-through simulation
        
        // Logic from `process` (Re-interleave)
        let mut output_interleaved = Vec::new();
        for t in 0..hop {
            for ch in 0..num_channels {
                output_interleaved.push(out_frame[[ch, t]]);
            }
        }
        
        // Verify Re-interleave
        for i in 0..hop*num_channels {
            if i % 2 == 0 {
                assert_eq!(output_interleaved[i], 1.0, "Output Left should be 1.0");
            } else {
                assert_eq!(output_interleaved[i], 0.0, "Output Right should be 0.0");
            }
        }
    }
}


