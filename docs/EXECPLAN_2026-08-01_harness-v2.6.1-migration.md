# Universal Agent Harness v2.6.1 移行

plan_id: PLAN-2026-08-01-HARNESS-V261
基準commit: e23b332
plan_revision: 1

<!-- execplan:original:start -->

## 目的 / 全体像

指定された GitHub Release v2.6.1 の clean ZIP を真正性確認後に repo 表層へ導入する。製品固有の Rust source、README、release workflow、packaging script は保持し、共通ハーネスの 7 CLI 設定、skills、hooks、運用 scripts、docs を追加する。

## 背景と見取り図

移行前の repo は Rust workspace と Windows release automation だけで、`AGENTS.md` や共通ハーネス設定を持たない。配布 asset は 101 files を含むが、その `README.md` と scaffold docs は導入先に合わせた統合が必要。製品 README は template が明示的に置換可能としているため保持する。

## 作業計画

1. GitHub Release metadata と clean ZIP を取得し、公開 SHA-256、tag、release date、manifest、bytes を検証する。
2. `codex/harness-v2.6.1-migration` branch で、製品固有ファイルを保持して clean ZIP のハーネスファイルを導入する。
3. `.gitignore` を preserve-first で統合し、PROJECT_BRIEF / REQS / WORKLOG を repo 実態と現在タスクで初期化する。
4. shared context、template smoke、security smoke、diff を検証し、checkpoint を記録する。

## 予定変更範囲

- 予定変更ファイル:
  - v2.6.1 clean ZIP に含まれるハーネス設定、skills、docs、scripts
  - `.gitignore`（既存規則を保持した追記）
  - `docs/PROJECT_BRIEF.md`, `docs/REQS.md`, `docs/WORKLOG.md`
  - 本 ExecPlan
- 許容する付随変更:
  - 同期 script が生成する `.claude/skills/`、`kilo.jsonc`、`opencode.jsonc` の整合修正
- 変更禁止範囲:
  - `plugin/`, `xtask/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`
  - `README.md`, `README_ja.md`, `.github/`, `scripts/package-release.ps1`
  - remote、tag、GitHub Release

## 検証と受け入れ条件

- release SHA-256 と tag-based asset verifier が pass する。
- clean ZIP の配布対象が欠落せず、変更禁止範囲に diff がない。
- `python3 scripts/sync_shared_context.py --check` が pass する。
- `bash scripts/smoke_template.sh` と `bash scripts/security_smoke.sh` が pass する。
- `git diff --check` が pass する。

<!-- execplan:original:end -->

## 進捗

- [x] (2026-08-01 JST) GitHub Release asset を取得し、SHA-256 と tag-based verifier を確認した。
- [x] (2026-08-01 JST) preserve-first で clean ZIP の共通ファイルを導入した。
- [x] (2026-08-01 JST) project docs と `.gitignore` を統合し、全 gate を実行した。

## 現在の停止点

- 現在位置: 移行と checkpoint が完了。commit / push は未実施。
- 未完了: なし。commit / push は今回の非目標。
- 次の一手: ユーザーが必要とした場合だけ差分を review して commit する。
- 次に読む文書: `docs/REQS.md`, `docs/PROJECT_BRIEF.md`
- 次に実行するコマンド: `git status --short`

## 発見事項

- 観測: GitHub API の匿名 curl は 404 だが、認証済み `gh release download` で正式 asset を取得できた。
  根拠: release metadata の asset id、size 215119 bytes、SHA-256 が公開 digest と一致。
- 観測: clean ZIP の template README は製品 README と衝突する。
  根拠: `docs/HARNESS.md` はプロジェクト README への置換を許可しており、製品 README を保持するのが導入契約に合う。
- 観測: clean ZIP の 3 source 箇所に、新規追加時だけ表面化する whitespace 警告があった。
  根拠: tracked / untracked 両方を対象にした check で EOF blank 1件、trailing whitespace 2件を検出。編集元を正規化して skill mirror を同期後、警告は 0 件になった。

## 逸脱提案

<!-- execplan:deviations -->

## 判断ログ

- 判断: template `.gitignore` は上書きせず、既存内容へハーネスの local artifact 規則だけ追記する。
  理由: `/target/` と `/dist/` など製品固有の除外を保持するため。
  日付/記録者: 2026-08-01 / Codex

## 成果と振り返り

- 成果: v2.6.1 clean harness 101 path を preserve-first で導入し、7 CLI context、構造、安全策の全 gate が pass した。
- 不足: 製品 formatter は WSL / Windows とも `cargo` が PATH に無く実行不能。製品コードは無変更で、今回のハーネス gate はすべて実行済み。
- 学び: `git diff --check` は untracked file を見ないため、新規 harness migration では各 untracked file の no-index check も必要。
- 目的との差分: 配布 asset の whitespace 3 source 箇所だけを移行先で正規化した。製品 README と `.gitignore`、3 working-memory docs は計画どおり project 固有内容を保持・統合した。

## 具体手順

1. `gh release view/download` と `sha256sum`、source repo の `build_release_asset.py verify` で asset を検証する。
2. clean ZIP を一時ディレクトリへ展開し、衝突する README / `.gitignore` を除いて共通ファイルをコピーする。
3. repo 固有 docs と ignore 規則を統合する。
4. manifest inventory、同期 check、2 smoke、diff check を実行する。

## 冪等性と復旧

- 同じ v2.6.1 asset の再コピーは同一 bytes になり、project docs と `.gitignore` は再上書きしない。
- 中断後の再開手順: 本文書と `docs/REQS.md` を読み、`git status --short` 後に未完了 gate から再開する。

## 成果物とメモ

- source release: `https://github.com/TsukinowaRin/multiagent-best-template/releases/tag/v2.6.1`
- asset: `multiagent-best-template-v2.6.1-clean.zip`
- verified SHA-256: `d82c17e0323343489ee7beebe9db65a090fab0fdce30271cdacafa5a6e24aab7`
