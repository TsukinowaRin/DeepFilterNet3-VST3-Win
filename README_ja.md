# DeepFilter VST3 Plugin

[DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet) を使用したリアルタイムノイズ除去VST3プラグインです。

## 特徴

- **AIベースのノイズ除去**: DeepFilterNet3ニューラルネットワークによる高品質なノイズ抑制
- **設定不要**: 適用するだけで自動的にノイズを除去
- **リアルタイム処理**: 48kHzでのリアルタイム処理に対応

## 動作要件

- **サンプルレート**: 48kHz（必須）
- **対応OS**: macOS (Apple Silicon / Intel), Windows, Linux
- **対応DAW**: DaVinci Resolve, Logic Pro, Ableton Live, Reaper, Cubase など VST3対応DAW

## インストール

### ビルド済みプラグイン

[Releases](https://github.com/YOUR_USERNAME/deepfilter-vst/releases) からダウンロードしてください。

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

## 使用方法

1. プラグインをインストールします。
2. DAWのプロジェクト設定（サンプルレート）を **48kHz** に設定します。
3. オーディオトラックに「DeepFilter Noise Reduction」を適用します。
4. 完了！（パラメータ調整は通常不要です）

## パラメータ

| パラメータ | 説明 | デフォルト |
| :--- | :--- | :--- |
| **Attenuation Limit** | ノイズ抑制量 (dB) | 100 |
| **Mix** | Dry/Wet比率 | 100% |

---

### 【警告】DaVinci Resolve 20における既知の問題点

現在のDeepFilterNet3-VST3プラグイン（nih-plug / Rust libDFベース）をDaVinci Resolve 20で使用した場合、**デリバーページでのオフラインレンダリング実行時に音声出力が完全に「無音」になる致命的な問題**が確認されています。

これは、本プラグインの処理パイプラインとDaVinci Resolveのオフラインレンダリングの挙動との間にある、根本的な非互換性に起因するものです。具体的な問題点（原因）は以下の通りです。

*   **サンプルレートの非互換と制限**
    Rustの`libdf`ライブラリは厳密に48kHzのサンプルレートを要求しますが、DaVinci Resolveはレンダリング中にサンプルレートを変更する可能性があり、処理が正常に行われません。
*   **オフライン処理への移行時の内部ステート初期化不良**
    DaVinci Resolveがリアルタイム再生からオフライン（非リアルタイム）レンダリングへ移行する際、プラグインを再初期化（`prepareToPlay`の再呼び出し）し、最高速度でオーディオブロックの処理を行います。このフローにおいて、STFT（短時間フーリエ変換）の内部状態（オーバーラップバッファ等）やRNNの隠れ状態、正規化の統計情報が適切にリセット・再初期化されないため、無音（ゼロ）が出力され続けます。
*   **動的なバッファサイズ変更への適応不全**
    オフラインレンダリング時、DaVinci Resolveはリアルタイム再生時とは異なるバッファサイズを使用することがあります。厳格なバッファ処理を要求する現行のライブラリは、このサイズ変動に適切に適応できません。
*   **レイテンシー補正の不具合**
    内部処理（20msのウィンドウ、10msのホップサイズ、2フレームの先読み）によって発生する約40msのアルゴリズムレイテンシーに対し、DaVinci Resolveのオフラインレンダラー環境下ではレイテンシー補正が正しく機能しません。

現在、DaVinci Resolve 20でのエクスポート（デリバー）において本プラグインを使用することはできませんのでご注意ください。

---

## ライセンス

MIT License

## クレジット

- [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) - Hendrik Schröter
- [nih-plug](https://github.com/robbert-vdh/nih-plug) - Robbert van der Helm
