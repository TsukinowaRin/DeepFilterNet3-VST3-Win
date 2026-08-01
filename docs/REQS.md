# 要件

## 依頼内容

- 依頼:
  - Cargo を導入し、前回完了した Universal Agent Harness v2.6.1 移行差分を commit / push する
- 背景:
  - 前回は WSL / Windows の PATH に `cargo` がなく、製品 formatter だけ未実行だった

## 目標

1. project pin に合わせた Rust 1.93.0、Cargo、rustfmt を WSL user 環境へ導入する
2. `cargo fmt --all -- --check` とハーネス gate を通す
3. migration 差分を 1 commit にまとめ、現在の topic branch を origin へ push する

## 非目標

- Rust / Cargo の Windows 側への追加導入
- プラグイン実装、dependencies、release workflow の変更
- main への直接 push、PR merge、tag 作成、GitHub Release 公開

## 制約

- Cargo は公式 rustup installer を使い、admin 権限なしで WSL user 環境へ導入する
- commit 対象は前回の v2.6.1 harness migration と今回の docs 更新だけに限定する
- commit 前に製品固有ファイルが意図せず変更されていないことを確認する
- push 先は `origin/codex/harness-v2.6.1-migration` とする

## 受け入れ条件

- [x] `rustc --version` と `cargo --version` が Rust 1.93.0 toolchain を示す
- [x] `cargo fmt --all -- --check` を実行し、既存の製品・sibling dependency の formatting diff を記録する（今回の範囲では変更しない）
- [x] `python3 scripts/sync_shared_context.py --check` が pass する
- [x] `bash scripts/security_smoke.sh` と `bash scripts/smoke_template.sh` が pass する
- [x] `git diff --check` と差分 review が pass する
- [ ] migration 差分が commit され、origin の topic branch と commit が一致する

## 仮定

- 「Cargo 入れて」は、現在作業中の WSL 環境への Rust toolchain 導入を指す
- 「プッシュ」は現在の topic branch の origin push を指し、main への統合は含まない
