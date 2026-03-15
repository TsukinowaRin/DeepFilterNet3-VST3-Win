Windows build of DeepFilterNet3 VST3, aligned with the upstream 1.0 release channel.

Install:

- Extract `deepfilter-vst-windows-x86_64.zip`
- Copy `deepfilter-vst.vst3` to `C:\Program Files\Common Files\VST3\`
- Optionally copy `deepfilter-vst.clap` to `C:\Program Files\Common Files\CLAP\`

Notes:

- Plugin name in DAWs: `DeepFilter Noise Reduction`
- GUI parameters: `Input Trim`, `Attenuation Limit`, `Mix`, `Output Gain`
- Required sample rate: `48 kHz`
- Known issue: DaVinci Resolve 20 offline export can render silence
