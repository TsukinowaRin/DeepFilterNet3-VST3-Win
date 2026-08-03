# 要件

## 依頼内容

- ユーザー明示要求（2026-08-03）: 「インストーラーをリリースして。コミットとプッシュも忘れずに」
- mailbox task: `TASK-RELEASE-V1-1-0-20260803`（receiver `grok-release-v110` / batch `batch-20260803T103919Z-4dbf7b608550`）
- 対象: 現在の `codex/vst-processing-reliability` 上の意図済み未 commit 差分を **v1.1.0** として commit / push / tag / GitHub Release 公開する

## 目標

1. dirty 差分の intent を確認し、plugin version を `1.0.0` → `1.1.0` に上げ、Cargo.lock を同期する
2. checkpoint gate を pass させたうえで 1 つの release commit を作る
3. `origin/codex/vst-processing-reliability` へ通常 push する（force 禁止）
4. `workflow_dispatch` で non-publishing CI（`version=v1.1.0-ci`）を成功させる（3ケース installer smoke 含む）
5. branch CI 成功後だけ annotated tag `v1.1.0` を push し、tag workflow で GitHub Release を公開する
6. release asset 2 件（setup EXE + sha256）を download 検証する

## 非目標 / DENY

- main / master への push / merge
- 既存 tag `windows` / `v0.1.0` / `v0.1.1` の移動・削除・force push
- 既存 release asset の上書き・削除
- remote に `v1.1.0` tag / release が既にある場合の上書き
- secrets / token / credential の読取・表示
- local Windows plugin の再 install / uninstall、admin 昇格、DAW kill
- test 弱体化、無関係 dependency 導入、破壊的 delete
- branch CI 失敗前の tag push
- local `dist/` 生成物の release upload（asset は tag workflow が同じ commit から build したものだけ）
- native subagent / 別モデルへの再委任

## 制約

- branch: `codex/vst-processing-reliability`
- version 判断: VST reliability 改善 + installer 機能追加 → minor **v1.1.0**
- release path: workflow の versioned tag path（`windows` 固定 tag は触らない）
- commit 前 checkpoint が pass しなければ push しない
- branch workflow 成功前に tag を push しない
- 公開後の外部状態（Release URL / SHA / hash）は追加 commit せず最終応答と mailbox 結果に記載

## 受け入れ条件

- [ ] plugin `Cargo.toml` / `Cargo.lock` の deepfilter-vst version が `1.1.0`
- [ ] 意図済み差分のみを 1 commit にまとめ worktree clean
- [ ] origin branch SHA が local commit と一致
- [ ] workflow_dispatch `v1.1.0-ci` の全 job 成功（3ケース install/uninstall smoke）
- [ ] annotated tag `v1.1.0` が成功 commit を指し、tag CI 成功
- [ ] GitHub Release `v1.1.0` が published / non-draft / non-prerelease
- [ ] asset 2 件の名前・size・sha256 が一致し EXE が MZ/PE

## 仮定

- repo: WSL `/mnt/c/Git_WorkSpace/DeepFilterNet3-VST3-Win` / Windows `C:\Git_WorkSpace\DeepFilterNet3-VST3-Win`
- preflight: local/remote tag `v1.1.0` なし、GitHub Release `v1.1.0` なし、`gh` auth 利用可（token 非表示）
- ユーザー承認済み: commit / push / tag / release 公開
- WSL の pkg-config/gl 失敗は既知。Windows MSVC cargo gates は必須
- local install/uninstall は行わない
