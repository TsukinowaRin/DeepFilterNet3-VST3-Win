# DeepFilterNet3 VST3 for Windows

Real-time noise reduction plugin for Windows, built with `nih_plug` and powered by `DeepFilterNet3`.

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
pwsh ./scripts/package-release.ps1 -Version v0.1.0
```

Recommended Rust/Cargo toolchain:

- `rustup default 1.93.0`

Build outputs:

- `target/bundled/deepfilter-vst.vst3`
- `target/bundled/deepfilter-vst.clap`
- `dist/DeepFilterNet3-VST3-Win-<version>-windows-x86_64.zip`

## Release Process

- Automated release: push a tag like `v0.1.0`
- Manual packaging: `pwsh ./scripts/package-release.ps1 -Version v0.1.0`
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

## License

`deepfilter-vst` is published under `MIT OR Apache-2.0`.

DeepFilterNet remains subject to its own license terms:

- <https://github.com/Rikorose/DeepFilterNet>

## Credits

- `DeepFilterNet` by Hendrik Schroter and contributors
- `nih-plug` by Robbert van der Helm
