# DeepFilterNet3 VST3 for Windows

`nih_plug` と `DeepFilterNet3` を使って実装した、Windows 向けリアルタイムノイズ除去プラグインです。

- English README: [`README.md`](README.md)
- DAW 上のプラグイン名: `DeepFilter Noise Reduction`

## クイックスタート

1. Releases から最新の `windows-x86_64-setup.exe` を取得します。
2. DAWを閉じ、installerを実行してWindowsの管理者確認を承認します。
3. タスク画面のチェックボックスで VST3 / CLAP / 両方を選びます（毎回両方選択で開始。前回インストールのタスク選択は復元しません）。
4. DAWのprojectを`48 kHz`に設定し、pluginを再scanします。
5. monoまたはstereo trackへ`DeepFilter Noise Reduction`を挿します。

## 対応範囲

- 主対象: Windows x86_64
- 形式: VST3 と CLAP。installer GUI でどちらか一方、または両方を選択可能（毎回両方選択で開始。前回のタスク選択は復元しない）
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

- `deepfilter-vst-windows-x86_64-setup.exe`
- `deepfilter-vst-windows-x86_64-setup.exe.sha256`

## インストール

1. 最新Releaseの`*-setup.exe`をダウンロードして実行します。
2. 管理者確認を承認します。タスク画面のチェックボックスで VST3 / CLAP / 両方を選べます。毎回 wizard 表示時は両方選択（`UsePreviousTasks=no` により前回インストールのタスク選択は復元しない）。少なくとも一方の選択が必須です。
3. DAW側でpluginを再scanします。

installerは選択したpluginをWindowsのsystem-wide標準位置へ配置します。

- VST3: `C:\Program Files\Common Files\VST3\deepfilter-vst.vst3`
- CLAP: `C:\Program Files\Common Files\CLAP\deepfilter-vst.clap`

## アンインストール

すべてのDAWを閉じ、次のどちらかを開きます。

- Windowsの設定 → アプリ → インストールされているアプリ → `DeepFilterNet3 VST3` → アンインストール
- Start Menu → `DeepFilterNet3 VST3` → `Uninstall DeepFilterNet3 VST3`

uninstallerはこのpackageがinstallしたVST3および/またはCLAPだけを削除します。共有VST3 / CLAP directoryにある他のpluginは削除しません。再install時に未選択のformatを自動削除することはありません。

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

`plugin/Cargo.toml`でreview済みDeepFilterNet revisionを固定しており、Cargoが取得します。隣接checkoutは不要です。

セットアップ例:

```bash
git clone https://github.com/TsukinowaRin/DeepFilterNet3-VST3-Win.git
cd DeepFilterNet3-VST3-Win
cargo test --locked
cargo run --locked --package xtask --release -- bundle deepfilter-vst --release
pwsh ./scripts/package-release.ps1 -ArtifactBase deepfilter-vst-windows-x86_64
```

packagingにはInno Setup 6が必要です。`ISCC.exe`が`PATH`にない場合は`-InnoCompiler`でcompiler pathを渡します。

推奨 Rust/Cargo toolchain:

- `rustup toolchain install 1.93.0`

生成物:

- `target/bundled/deepfilter-vst.vst3`
- `target/bundled/deepfilter-vst.clap`
- `dist/deepfilter-vst-windows-x86_64-setup.exe`
- `dist/deepfilter-vst-windows-x86_64-setup.exe.sha256`

## Release 作成

- version 付き installer release: annotated tag（例: `v1.1.0`）を push（asset 基名 `DeepFilterNet3-VST3-Win-v1.1.0-windows-x86_64-setup.exe`）
- 旧 Windows 1.0 チャンネル tag: `windows` → Release 名 `v1.0.0-deepfilter-vst3-windows`（固定 channel。minor release では動かさない）
- 手動パッケージ: `pwsh ./scripts/package-release.ps1 -Version v1.1.0 -ArtifactBase DeepFilterNet3-VST3-Win-v1.1.0-windows-x86_64`
- 自動化の定義: `.github/workflows/release.yml`

## リポジトリ構成

- `plugin/`: プラグイン本体
- `xtask/`: `nih_plug_xtask` エントリポイント
- `installer/deepfilter-vst.iss`: Inno Setup installer定義
- `scripts/package-release.ps1`: installerとchecksum生成
- `scripts/test-installer.ps1`: CI専用install / uninstall smoke test
- `.github/workflows/release.yml`: GitHub Releases 自動化

## 制約

- 現在の DeepFilterNet ランタイム初期化は `48 kHz` 前提です。
- 公式サポート対象は Windows VST3 配布です。
- installerは未code signingのため、WindowsがSmartScreenの警告を表示する場合があります。

---

### DaVinci Resolve 20のオフライン出力について

既存のWindows配布版では、デリバーページからのオフライン出力が無音になる現象が確認されています。現在のsourceでは、その実装で見つかった3つの原因を修正しました。

- hostのblock sizeに関係なく、全sampleをDeepFilterNetの固定480-sample frameへ渡す
- hostが新しい処理segmentを始める際に、pluginとmodelの状態をresetする
- 48 kHzで合計1,920 sample（model latency + adapter 1 frame）のlatencyをhostへ通知し、dry経路も同じ長さ遅延する

可変block、reset、dry/wet整列、model失敗時のfallbackは自動testで確認済みです。DaVinci Resolve 20のWindows実機によるオフライン出力はまだ再検証していないため、出力互換性は未確認です。Resolveのprojectと出力音声は48 kHzにしてください。それ以外のsample rateではpluginの初期化が失敗します。

---

## ライセンス

`deepfilter-vst` は `MIT OR Apache-2.0` です。

`DeepFilterNet` 側のライセンスは upstream に従います。

- <https://github.com/Rikorose/DeepFilterNet>

## Credits

- [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) - Hendrik Schröter
- [nih-plug](https://github.com/robbert-vdh/nih-plug) - Robbert van der Helm
