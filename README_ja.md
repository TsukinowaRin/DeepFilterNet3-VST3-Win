# DeepFilter VST3 Plugin

[DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet) を使用したリアルタイムノイズ除去VST3プラグインです。

## 特徴

- **AIベースのノイズ除去**: DeepFilterNet3ニューラルネットワークによる高品質なノイズ抑制
- **トゥルー・ステレオ対応**: L/Rチャンネルを個別に処理することで、モノラルにダウンミックスされることなくステレオ音像（空間の広がり）を維持します。
- **設定不要**: 適用するだけで自動的にノイズを除去。必要に応じて微調整も可能です。
- **リアルタイム処理**: 48kHzでのリアルタイム処理に対応

## 動作要件

- **サンプルレート**: 48kHz（必須）
- **対応OS**: macOS (Apple Silicon / Intel), Windows, Linux
- **対応DAW**: DaVinci Resolve, Logic Pro, Ableton Live, Reaper, Cubase など VST3対応DAW

## インストール

### ビルド済みプラグイン

[Releases](https://github.com/YOUR_USERNAME/deepfilter-vst/releases) から最新のZIPファイルをダウンロードし、展開してください。

#### macOS

**システム全体にインストール:**
```bash
sudo cp -r deepfilter-vst.vst3 /Library/Audio/Plug-Ins/VST3/
```

**または ユーザー専用:**
```bash
cp -r deepfilter-vst.vst3 ~/Library/Audio/Plug-Ins/VST3/
```

#### Windows

`deepfilter-vst.vst3` フォルダを以下のパスにコピーしてください。
```text
C:\Program Files\Common Files\VST3\
```

#### Linux

```bash
cp -r deepfilter-vst.vst3 ~/.vst3/
```

---

### ソースからビルド

**必要なもの:**
- Rust (1.70以上)

**ビルド手順:**

```bash
git clone https://github.com/YOUR_USERNAME/deepfilter-vst.git
cd deepfilter-vst
cargo xtask bundle deepfilter-vst --release
```

**ビルド成果物:** `target/bundled/deepfilter-vst.vst3`
成果物をZIPにまとめる場合は以下のPowerShellコマンドが利用できます：
```powershell
Compress-Archive -Path "DeepFilterNet3-VST3-Win\target\bundled\*" -DestinationPath "DeepFilterNet3-VST3-Win-Release.zip" -Force
```

## 使用方法

1. プラグインをインストールします。
2. DAWのプロジェクト設定（サンプルレート）を **48kHz** に設定します。
3. オーディオトラックに「DeepFilter Noise Reduction」を適用します。
4. 必要に応じてInput TrimやOutput Gainを調整します。

## パラメータ

| パラメータ | 説明 | 範囲 / デフォルト |
| :--- | :--- | :--- |
| **Input Trim** | 処理前の入力ゲイン補正 | -24dB 〜 +24dB (デフォルト: 0dB) |
| **Attenuation Limit** | ノイズの最大抑制量 | 0dB 〜 100dB (デフォルト: 100dB) |
| **Mix** | Dry/Wet比率 (パラレル処理用) | 0% 〜 100% (デフォルト: 100%) |
| **Output Gain** | 処理後の最終ゲイン補正 | -24dB 〜 +24dB (デフォルト: 0dB) |

## ライセンス

MIT License

## クレジット

- [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) - Hendrik Schröter
- [nih-plug](https://github.com/robbert-vdh/nih-plug) - Robbert van der Helm