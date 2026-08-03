# プロジェクト概要

## この repo について

- プロダクト / ライブラリ / サービス:
  - DeepFilterNet3 を使う Windows x86_64 向けリアルタイムノイズ除去 VST3 / CLAP プラグイン
- 主目的:
  - DAW 上で 48 kHz の音声をリアルタイムにノイズ除去し、Windows 用バイナリを配布する
- 主な利用者:
  - VST3 / CLAP 対応 DAW を使う Windows ユーザーと、Rust でプラグインをビルドする開発者
  - installer GUI で VST3 / CLAP / 両方をチェックボックス選択できる（毎回両方選択で開始。`UsePreviousTasks=no` で previous task は復元しない）

## 入口

- 最初に読むもの:
  - `AGENTS.md`, `docs/PROJECT_BRIEF.md`, `docs/REQS.md`
- 主要ディレクトリ:
  - `plugin/`: `nih_plug` ベースのプラグイン本体
  - `xtask/`: VST3 / CLAP bundle の生成入口
  - `scripts/`: 製品 release packaging とハーネス運用スクリプト
  - `.github/workflows/`: Windows build / release automation
- ハーネス構成の正本:
  - `docs/HARNESS.md`

## Build / Test / Run

- Build:
  - `cargo run --locked --package xtask --release -- bundle deepfilter-vst --release`
  - Windows installer: `pwsh ./scripts/package-release.ps1 -ArtifactBase deepfilter-vst-windows-x86_64`
- Test / Smoke:
  - 製品: `cargo test --locked`
  - ハーネス構造の点検: `bash scripts/smoke_template.sh` / `bash scripts/security_smoke.sh`
- Lint / Format:
  - `cargo fmt --all -- --check`

## 制約

- 許可する変更:
  - 依頼に直接関係する製品コード、設定、docs、tests
  - skill 編集は `.agents/skills/` のみ、mirror は `python3 scripts/sync_shared_skills.py` 経由
- 禁止する変更:
  - `.claude/settings.local.json` などユーザーローカル設定の変更
  - ユーザーの明示許可がない tag push、release 公開、既存配布資産の差し替え
- プラットフォーム制約:
  - 製品の主対象は Windows x86_64。サンプルレートは 48 kHz 必須
  - DeepFilterNet sourceは`plugin/Cargo.toml`のgit revisionを正本とし、Cargoが取得する
  - WSL↔Windows 相互運用は `scripts/win_*.sh` / `scripts/wsl_exec.*` 経由

## 環境メモ

- 必要なランタイム:
  - python3 (3.11+), bash, git（ハーネス script 用）
  - Rust 1.93.0、PowerShell、Inno Setup 6（Windows installer build）
- パッケージマネージャ:
  - Cargo / rustup
- 外部サービス:
  - GitHub Actions / GitHub Releases（公開時のみ）
