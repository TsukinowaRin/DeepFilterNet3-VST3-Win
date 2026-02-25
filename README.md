# DeepFilter VST3 Plugin

A real-time noise reduction VST3 plugin using [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet).

## Features

- **AI-Based Noise Reduction**: High-quality noise suppression powered by the DeepFilterNet3 neural network.
- **Zero Configuration**: Automatically removes noise just by applying the plugin.
- **Real-time Processing**: Supports real-time processing at 48kHz.

## Requirements

- **Sample Rate**: 48kHz (Required)
- **Supported OS**: macOS (Apple Silicon / Intel), Windows, Linux
- **Supported DAWs**: DaVinci Resolve, Logic Pro, Ableton Live, Reaper, Cubase, and other VST3-compatible DAWs.

## Installation

### Pre-built Binaries

Download the latest version from [Releases](https://github.com/YOUR_USERNAME/deepfilter-vst/releases).

#### macOS

**System-wide installation:**
```bash
sudo cp -r deepfilter-vst.vst3 /Library/Audio/Plug-Ins/VST3/
```

**Or user-only installation:**
```bash
cp -r deepfilter-vst.vst3 ~/Library/Audio/Plug-Ins/VST3/
```

#### Windows

Copy the `deepfilter-vst.vst3` folder to the following path:
```text
C:\Program Files\Common Files\VST3\
```

#### Linux

```bash
cp -r deepfilter-vst.vst3 ~/.vst3/
```

---

### Build from Source

**Prerequisites:**
- Rust (1.70 or later)

**Build Instructions:**

```bash
git clone https://github.com/YOUR_USERNAME/deepfilter-vst.git
cd deepfilter-vst
cargo xtask bundle deepfilter-vst --release
```

**Build Artifact:** `target/bundled/deepfilter-vst.vst3`

## Usage

1. Install the plugin.
2. Set your DAW project sample rate to **48kHz**.
3. Apply "DeepFilter Noise Reduction" to your audio track.
4. Done! (Parameter adjustment is usually not required).

## Parameters

| Parameter | Description | Default |
| :--- | :--- | :--- |
| **Attenuation Limit** | Noise reduction amount (dB) | 100 |
| **Mix** | Dry/Wet ratio | 100% |

---

### 【WARNING】Known Issues with DaVinci Resolve 20

When using the current DeepFilterNet3-VST3 plugin (nih-plug / Rust libDF based) with DaVinci Resolve 20, **a critical issue has been confirmed where audio output becomes completely "silent" during offline rendering execution on the Deliver page**.

This is caused by fundamental incompatibility between the plugin's processing pipeline and DaVinci Resolve's offline rendering behavior. The specific issues (causes) are as follows:

*   **Sample Rate Incompatibility and Limitations**
    The Rust `libdf` library strictly requires a 48kHz sample rate, but DaVinci Resolve may change the sample rate during rendering, preventing normal processing from occurring.
*   **Poor Internal State Initialization During Transition to Offline Processing**
    When DaVinci Resolve transitions from real-time playback to offline (non-real-time) rendering, it reinitializes the plugin (re-calling `prepareToPlay`) and processes audio blocks at maximum speed. In this flow, the internal states of STFT (Short-Time Fourier Transform) such as overlap buffers, RNN hidden states, and normalization statistics are not properly reset/reinitialized, causing continuous output of silence (zeros).
*   **Poor Adaptation to Dynamic Buffer Size Changes**
    During offline rendering, DaVinci Resolve sometimes uses different buffer sizes than during real-time playback. The current library, which requires strict buffer processing, cannot properly adapt to these size variations.
*   **Latency Compensation Malfunction**
    For the approximately 40ms algorithmic latency caused by internal processing (20ms window, 10ms hop size, 2-frame lookahead), latency compensation does not function correctly in DaVinci Resolve's offline renderer environment.

Please note that this plugin currently cannot be used for export (delivery) in DaVinci Resolve 20.

---

## License

MIT License

## Credits

- [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) - Hendrik Schröter
- [nih-plug](https://github.com/robbert-vdh/nih-plug) - Robbert van der Helm
