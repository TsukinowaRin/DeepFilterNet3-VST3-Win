# DeepFilterNet3 VST3 for Windows

`nih_plug` と `DeepFilterNet3` を使って実装した、Windows 向けリアルタイムノイズ除去プラグインです。

- English README: [`README.md`](README.md)
- DAW 上のプラグイン名: `DeepFilter Noise Reduction`

## クイックスタート

1. Releases から最新の `windows-x86_64.zip` を取得します。
2. 展開します。
3. `deepfilter-vst.vst3` を `C:\Program Files\Common Files\VST3\` にコピーします。
4. DAW のプロジェクトを `48 kHz` に設定します。
5. mono または stereo トラックへ `DeepFilter Noise Reduction` を挿します。

## 対応範囲

- 主対象: Windows x86_64
- 主形式: VST3
- 追加成果物: 同じソースから CLAP も生成
- 必須サンプルレート: 48 kHz
- 対応チャンネル: mono / stereo

## 特徴

- DeepFilterNet3 ベースのリアルタイムノイズ除去
- モノラル化しない True Stereo 処理
- `Input Trim`、`Attenuation Limit`、`Mix`、`Output Gain` を搭載
- `nih_plug_egui` によるシンプルな GUI

## ダウンロード

最新版の配布物:

- <https://github.com/TsukinowaRin/DeepFilterNet3-VST3-Win/releases>

想定している配布ファイル名:

- `DeepFilterNet3-VST3-Win-<version>-windows-x86_64.zip`
- `DeepFilterNet3-VST3-Win-<version>-windows-x86_64.zip.sha256`

## インストール

1. 最新の Release ZIP をダウンロードして展開します。
2. `deepfilter-vst.vst3` を `C:\Program Files\Common Files\VST3\` にコピーします。
3. CLAP 版も使う場合は `deepfilter-vst.clap` を `C:\Program Files\Common Files\CLAP\` にコピーします。
4. DAW 側でプラグインを再スキャンします。

## パラメータ

| パラメータ | 説明 | 範囲 | デフォルト |
| :--- | :--- | :--- | :--- |
| `Input Trim` | 処理前ゲイン | `-24 dB .. +24 dB` | `0 dB` |
| `Attenuation Limit` | ノイズ抑制の上限 | `0 dB .. 100 dB` | `100 dB` |
| `Mix` | Dry/Wet ブレンド | `0% .. 100%` | `100%` |
| `Output Gain` | 最終出力ゲイン | `-24 dB .. +24 dB` | `0 dB` |

## 動作条件

- VST3 を読み込める Windows ホスト
- DAW プロジェクトのサンプルレートが `48 kHz`
- mono / stereo 入力

## ソースからビルド

現状の構成では、このリポジトリの隣に upstream の `DeepFilterNet` ソースが必要です。

```text
workspace/
  DeepFilterNet3-VST3-Win/
  DeepFilterNet/
```

セットアップ例:

```bash
git clone https://github.com/TsukinowaRin/DeepFilterNet3-VST3-Win.git
git clone https://github.com/Rikorose/DeepFilterNet.git
cd DeepFilterNet3-VST3-Win
cargo test
cargo xtask bundle deepfilter-vst --release
pwsh ./scripts/package-release.ps1 -Version v0.1.1
```

推奨 Rust/Cargo toolchain:

- `rustup default 1.93.0`

生成物:

- `target/bundled/deepfilter-vst.vst3`
- `target/bundled/deepfilter-vst.clap`
- `dist/DeepFilterNet3-VST3-Win-<version>-windows-x86_64.zip`

## Release 作成

- 自動リリース: `v0.1.1` のようなタグを push
- 手動パッケージ: `pwsh ./scripts/package-release.ps1 -Version v0.1.1`
- 自動化の定義: `.github/workflows/release.yml`

## リポジトリ構成

- `plugin/`: プラグイン本体
- `xtask/`: `nih_plug_xtask` エントリポイント
- `scripts/package-release.ps1`: Release ZIP 生成
- `.github/workflows/release.yml`: GitHub Releases 自動化

## 制約

- 現在の DeepFilterNet ランタイム初期化は `48 kHz` 前提です。
- 公式サポート対象は Windows VST3 配布です。
- ソースビルドには上記の sibling `DeepFilterNet` 配置が必要です。

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

`deepfilter-vst` は `MIT OR Apache-2.0` です。

`DeepFilterNet` 側のライセンスは upstream に従います。

- <https://github.com/Rikorose/DeepFilterNet>

## Credits

- [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) - Hendrik Schröter
- [nih-plug](https://github.com/robbert-vdh/nih-plug) - Robbert van der Helm
