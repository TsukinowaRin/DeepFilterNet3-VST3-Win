# DeepFilterNet3 VST3 for Windows

Real-time noise reduction plugin for Windows, built with `nih_plug` and powered by `DeepFilterNet3`.

- Japanese version / 日本語版: [`README_ja.md`](README_ja.md)
- Plugin name in DAWs: `DeepFilter Noise Reduction`

## Quick Start

1. Download the latest `windows-x86_64.zip` from Releases.
2. Extract it.
3. Copy `deepfilter-vst.vst3` to `C:\Program Files\Common Files\VST3\`.
4. Set your DAW project to `48 kHz`.
5. Load `DeepFilter Noise Reduction` on a mono or stereo track.

## Scope

- Primary target: Windows x86_64
- Primary format: VST3
- Additional artifact: CLAP is bundled from the same source tree
- Required sample rate: 48 kHz
- Supported channel layouts: mono / stereo

## Features

- DeepFilterNet3-based real-time denoising
- True stereo processing without mono downmix
- `Input Trim`, `Attenuation Limit`, `Mix`, `Output Gain`
- Simple GUI implemented with `nih_plug_egui`

## Download

Download the latest release assets from:

- <https://github.com/TsukinowaRin/DeepFilterNet3-VST3-Win/releases>

Recommended asset names:

- `DeepFilterNet3-VST3-Win-<version>-windows-x86_64.zip`
- `DeepFilterNet3-VST3-Win-<version>-windows-x86_64.zip.sha256`

## Installation

1. Download and extract the latest release ZIP.
2. Copy `deepfilter-vst.vst3` to `C:\Program Files\Common Files\VST3\`.
3. If you want the CLAP build as well, copy `deepfilter-vst.clap` to `C:\Program Files\Common Files\CLAP\`.
4. Rescan plugins in your DAW.

## Parameters

| Parameter | Description | Range | Default |
| :--- | :--- | :--- | :--- |
| `Input Trim` | Gain before denoising | `-24 dB .. +24 dB` | `0 dB` |
| `Attenuation Limit` | Maximum noise reduction amount | `0 dB .. 100 dB` | `100 dB` |
| `Mix` | Dry/Wet blend | `0% .. 100%` | `100%` |
| `Output Gain` | Final output gain | `-24 dB .. +24 dB` | `0 dB` |

## Requirements

- Windows host with VST3 support
- DAW project sample rate set to `48 kHz`
- Mono or stereo input

## Build From Source

This repository currently expects the upstream `DeepFilterNet` source tree to exist next to this repository:

```text
workspace/
  DeepFilterNet3-VST3-Win/
  DeepFilterNet/
```

Example setup:

```bash
git clone https://github.com/TsukinowaRin/DeepFilterNet3-VST3-Win.git
git clone https://github.com/Rikorose/DeepFilterNet.git
cd DeepFilterNet3-VST3-Win
cargo test
cargo xtask bundle deepfilter-vst --release
pwsh ./scripts/package-release.ps1 -Version v0.1.1
```

Recommended Rust/Cargo toolchain:

- `rustup default 1.93.0`

Build outputs:

- `target/bundled/deepfilter-vst.vst3`
- `target/bundled/deepfilter-vst.clap`
- `dist/DeepFilterNet3-VST3-Win-<version>-windows-x86_64.zip`

## Release Process

- Automated release: push a tag like `v0.1.1`
- Manual packaging: `pwsh ./scripts/package-release.ps1 -Version v0.1.1`
- Release automation is defined in `.github/workflows/release.yml`

## Repository Layout

- `plugin/`: plugin implementation
- `xtask/`: `nih_plug_xtask` entry point
- `scripts/package-release.ps1`: release ZIP generator
- `.github/workflows/release.yml`: GitHub Releases automation

## Known Limitations

- `48 kHz` is mandatory because the current DeepFilterNet runtime is initialized only for that sample rate.
- Official support is focused on Windows VST3 delivery.
- Building from source requires the sibling `DeepFilterNet` checkout described above.

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

`deepfilter-vst` is published under `MIT OR Apache-2.0`.

DeepFilterNet remains subject to its own license terms:

- <https://github.com/Rikorose/DeepFilterNet>

## Credits

- [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) - Hendrik Schröter
- [nih-plug](https://github.com/robbert-vdh/nih-plug) - Robbert van der Helm
