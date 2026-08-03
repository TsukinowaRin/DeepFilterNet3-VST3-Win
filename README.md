# DeepFilterNet3 VST3 for Windows

Real-time noise reduction plugin for Windows, built with `nih_plug` and powered by `DeepFilterNet3`.

- Japanese version / 日本語版: [`README_ja.md`](README_ja.md)
- Plugin name in DAWs: `DeepFilter Noise Reduction`

## Quick Start

1. Download the latest `windows-x86_64-setup.exe` from Releases.
2. Close your DAW, run the installer, and approve the Windows administrator prompt.
3. On the tasks page, choose formats with checkboxes: VST3, CLAP, or both (both start selected every run; previous install choices are not restored).
4. Set your DAW project to `48 kHz` and rescan plug-ins.
5. Load `DeepFilter Noise Reduction` on a mono or stereo track.

## Scope

- Primary target: Windows x86_64
- Formats: VST3 and CLAP; the installer GUI lets you select either or both (both selected by default every run; previous task choices are not restored)
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

- `deepfilter-vst-windows-x86_64-setup.exe`
- `deepfilter-vst-windows-x86_64-setup.exe.sha256`

## Installation

1. Download and run the latest `*-setup.exe` release asset.
2. Approve the administrator prompt. On the tasks page, checkboxes let you install VST3, CLAP, or both. Both formats start selected every wizard run (`UsePreviousTasks=no`; previous install task choices are not restored). At least one format is required.
3. Rescan plug-ins in your DAW.

The installer places the selected plug-ins in the Windows system-wide locations:

- VST3: `C:\Program Files\Common Files\VST3\deepfilter-vst.vst3`
- CLAP: `C:\Program Files\Common Files\CLAP\deepfilter-vst.clap`

## Uninstallation

Close all DAWs, then use either of these GUI entries:

- Windows Settings → Apps → Installed apps → `DeepFilterNet3 VST3` → Uninstall
- Start Menu → `DeepFilterNet3 VST3` → `Uninstall DeepFilterNet3 VST3`

The uninstaller removes only the VST3 and/or CLAP files this package installed. It does not remove other plug-ins from the shared VST3 or CLAP directories. Reinstalling with fewer formats selected does not delete a previously installed format that you leave unchecked.

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

Cargo fetches the reviewed DeepFilterNet revision pinned in `plugin/Cargo.toml`. A sibling checkout is not required.

Example setup:

```bash
git clone https://github.com/TsukinowaRin/DeepFilterNet3-VST3-Win.git
cd DeepFilterNet3-VST3-Win
cargo test --locked
cargo run --locked --package xtask --release -- bundle deepfilter-vst --release
pwsh ./scripts/package-release.ps1 -ArtifactBase deepfilter-vst-windows-x86_64
```

Packaging requires Inno Setup 6. Pass its compiler path with `-InnoCompiler` when `ISCC.exe` is not on `PATH`.

Recommended Rust/Cargo toolchain:

- `rustup toolchain install 1.93.0`

Build outputs:

- `target/bundled/deepfilter-vst.vst3`
- `target/bundled/deepfilter-vst.clap`
- `dist/deepfilter-vst-windows-x86_64-setup.exe`
- `dist/deepfilter-vst-windows-x86_64-setup.exe.sha256`

## Release Process

- Versioned installer release: push an annotated tag such as `v1.1.0` (asset base `DeepFilterNet3-VST3-Win-v1.1.0-windows-x86_64-setup.exe`)
- Legacy Windows 1.0 channel tag: `windows` → release name `v1.0.0-deepfilter-vst3-windows` (fixed channel; do not retarget for minor releases)
- Manual packaging: `pwsh ./scripts/package-release.ps1 -Version v1.1.0 -ArtifactBase DeepFilterNet3-VST3-Win-v1.1.0-windows-x86_64`
- Release automation is defined in `.github/workflows/release.yml`

## Repository Layout

- `plugin/`: plugin implementation
- `xtask/`: `nih_plug_xtask` entry point
- `installer/deepfilter-vst.iss`: Inno Setup installer definition
- `scripts/package-release.ps1`: installer and checksum generator
- `scripts/test-installer.ps1`: CI-only install/uninstall smoke test
- `.github/workflows/release.yml`: GitHub Releases automation

## Known Limitations

- `48 kHz` is mandatory because the current DeepFilterNet runtime is initialized only for that sample rate.
- Official support is focused on Windows VST3 delivery.
- The installer is not code-signed yet, so Windows may display a SmartScreen warning.

---

### DaVinci Resolve 20 offline export status

Released Windows builds have been observed to render silence during offline export on the Deliver page. The current source addresses three causes found in that implementation:

- host blocks of any size are bridged to DeepFilterNet's fixed 480-sample frames without dropping or shifting samples;
- plugin and model state are reset when the host starts a new processing segment;
- the 1,920-sample total latency at 48 kHz (model latency plus one adapter frame) is reported to the host, and the dry path is delayed by the same amount.

These changes pass automated chunking, reset, dry/wet alignment, and fallback tests. DaVinci Resolve 20 offline export has not yet been retested on Windows, so export compatibility remains unconfirmed. Keep the Resolve project and delivery audio at 48 kHz; the plugin rejects other sample rates.

---

## License

`deepfilter-vst` is published under `MIT OR Apache-2.0`.

DeepFilterNet remains subject to its own license terms:

- <https://github.com/Rikorose/DeepFilterNet>

## Credits

- [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) - Hendrik Schröter
- [nih-plug](https://github.com/robbert-vdh/nih-plug) - Robbert van der Helm
