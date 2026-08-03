Windows build of DeepFilterNet3 VST3, aligned with the upstream 1.0 release channel.

Install:

- Download and run `deepfilter-vst-windows-x86_64-setup.exe`
- Approve the administrator prompt
- On the tasks page, use checkboxes to install VST3, CLAP, or both (both start selected every run; previous install task choices are not restored; at least one format is required)

Uninstall:

- Windows Settings > Apps > Installed apps > `DeepFilterNet3 VST3` > Uninstall
- Or use Start Menu > `DeepFilterNet3 VST3` > `Uninstall DeepFilterNet3 VST3`

Notes:

- Plugin name in DAWs: `DeepFilter Noise Reduction`
- GUI parameters: `Input Trim`, `Attenuation Limit`, `Mix`, `Output Gain`
- Required sample rate: `48 kHz`
- Known issue: DaVinci Resolve 20 offline export can render silence
- The installer is not code-signed; Windows may display a SmartScreen warning
