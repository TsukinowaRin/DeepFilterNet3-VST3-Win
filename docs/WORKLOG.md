# 作業ログ

このファイルは current task の停止点と次の一手だけを残す。テンプレート配布時には「現在の状態」とエントリを空 scaffold に戻す。

## handoff の最低基準

- chat 履歴に依存せず、docs だけで再開できること。
- 「たぶんこうだろう」で埋めなくてよいこと: 観測事実、実行したコマンドと結果、判断と仮定を残す。
- 次のエージェントが最初に開く文書と、最初に打つコマンドが分かること。
- 未確認部分と失敗した試行を隠さないこと。「途中です」「だいたい終わった」は handoff にならない。

## 現在の状態

- 現在の作業:
  - v1.1.0 release（TASK-RELEASE-V1-1-0-20260803）— checkpoint pass、commit / push 直前
- 直近の状態:
  - plugin version `1.1.0`、local gates 全 pass、tag/release 未作成
- 次にやること:
  - 意図済み差分を 1 commit → branch push → workflow_dispatch `v1.1.0-ci` → 成功後 tag `v1.1.0`
- ブロッカー:
  - なし（ユーザー承認済み）
- 次に最初に読む文書:
  - `docs/REQS.md`, `docs/EXECPLAN_2026-08-03_release-v1.1.0.md`
- 次に最初に実行するコマンド:
  - `git status --short` のあと stage / commit

---

## エントリ

### 2026-08-03 JST（v1.1.0 release — preflight / checkpoint）

- 目的: VST reliability + Windows installer 差分を v1.1.0 として commit / push / tag / GitHub Release
- 承認: 2026-08-03 ユーザー「インストーラーをリリースして。コミットとプッシュも忘れずに」/ mailbox `TASK-RELEASE-V1-1-0-20260803`
- preflight:
  - branch `codex/vst-processing-reliability` @ `4c9fd6e`、upstream 未設定
  - local/remote tag `v1.1.0` なし、`gh release view v1.1.0` not found
  - `gh` login 済み（token 非表示）
  - dirty intent: VST stream/reliability、pinned DeepFilterNet、installer/package/workflow、README/docs。secret/dist/target は stage 外
- 実施:
  1. `docs/REQS.md` を release 要求へ更新
  2. `docs/EXECPLAN_2026-08-03_release-v1.1.0.md` 新規
  3. plugin version `1.0.0` → `1.1.0`、Cargo.lock 同期、README release 節の最小更新
- 検証（commit 前 checkpoint）:
  - PowerShell parser 3本 OK
  - `test-installer-source.ps1` OK
  - ISCC via `package-release.ps1 -Version v1.1.0` → setup 26,487,022 bytes、SHA-256 `48a8788f49feab7b3b61627a237aaea357b405fe12d18728a8c13f07ff51aac2`（local のみ。release upload しない）
  - Windows MSVC: `cargo fmt --all -- --check` / `cargo check --locked` / `cargo test --locked`（9 passed）
  - `security_smoke.sh` OK、`TEMPLATE_SMOKE_WINDOWS_TIMEOUT=20s smoke_template.sh` OK、`git diff --check` OK
- 明示しなかったこと: main push、既存 tag 移動、local reinstall、secrets 読取、test 弱体化
- Go / No-Go: checkpoint Go。branch CI / tag は push 後
- mailbox: batch `batch-20260803T103919Z-4dbf7b608550` / task `TASK-RELEASE-V1-1-0-20260803` / receiver `grok-release-v110--20260803T103729Z--395ccd33`
- 次の一手: commit → push → workflow_dispatch `v1.1.0-ci`

### 2026-08-03 JST（format checkboxes 監督差し戻し — UsePreviousTasks=no）

- 目的: 「初期両方選択」を previous install の有無に依存せず保証する
- 承認: commander-format-select 差し戻し instruction（batch `batch-20260803T091601Z-3b37d4c2935a`）
- 実装: `UsePreviousTasks=no`、task 行から `checkedonce`/`unchecked` 削除、source contract 更新、docs 同期
- 検証: parser / source contract / ISCC compile / harness gates OK（詳細は本 release 以前の記録）
- Go / No-Go: local 修正 Go。CI は release タスクで実行

### 2026-08-03 JST（installer format checkboxes — mailbox TASK-INSTALLER-FORMAT-CHECKBOXES）

- 目的: installer GUI で VST3 / CLAP / 両方をチェックボックス選択（初期両方選択）
- 実装: 独立 Tasks、Files 条件化、NextButtonClick ガード、3ケース CI smoke、docs 同期
- 検証: parser / source / ISCC / Windows cargo 9 passed / harness OK
- 注: 初回の「初期両方選択」は previous install 環境で保証されず、監督差し戻しで修正済み
