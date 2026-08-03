# Windows GUI installer / uninstaller

plan_id: PLAN-2026-08-03-WINDOWS-INSTALLER
基準commit: 4c9fd6e
plan_revision: 1

<!-- execplan:original:start -->

## 目的 / 全体像

Windows配布物の正本を手動コピー用ZIPからInno SetupのGUI installer EXEへ変更する。installerはVST3を64-bit Common Filesへ必須配置し、CLAPはGUI taskで選択できる。stable AppIdでApps & Featuresのuninstall entryを登録し、Start MenuにGUI uninstallerの入口を作る。

## 背景と見取り図

現行の`scripts/package-release.ps1`はVST3 / CLAP / READMEをZIPへ格納するだけで、install / uninstallをWindowsに登録しない。そのため利用者はProgram Filesへ手動コピーし、削除時もfileを自分で探す必要がある。GitHub-hosted `windows-2025` runnerにはInno Setupが導入済みなため、repoへcompiler binaryを持ち込まずbuildできる。

## 作業計画

1. stable AppId、64-bit install mode、VST3 / optional CLAP、docs、uninstall metadata / shortcutを持つInno Setup sourceを追加する。
2. packaging scriptをEXE + SHA-256生成に変更し、bundle / compiler不在でfail closedする。
3. ephemeral Windows CIでsilent install / registry / file / uninstallを検証するPowerShell smokeを追加する。
4. workflowを`windows-2025`とinstaller artifactへ更新し、README / release body / PROJECT_BRIEFを同期する。
5. PowerShell parser、Inno compile（CIまたはcompiler利用可能環境）、Cargo、security / template smoke、final diffを検証する。

## 予定変更範囲

- 予定変更ファイル:
  - `installer/deepfilter-vst.iss`
  - `scripts/package-release.ps1`
  - `scripts/test-installer.ps1`
  - `.github/workflows/release.yml`
  - `.github/release-body-windows.md`
  - `README.md`, `README_ja.md`
  - `docs/PROJECT_BRIEF.md`, `docs/REQS.md`, `docs/WORKLOG.md`, 本ExecPlan
- 許容する付随変更:
  - installer sourceの構文 / metadata検査用scriptの追加
  - `dist/`配下のignore済み検証生成物
- 変更禁止範囲:
  - `plugin/src/`, `plugin/Cargo.toml`, `Cargo.lock`
  - VST3 / CLAP / parameter ID、DSP実装
  - harness hooks / permissions / skill
  - existing GitHub release asset、tag、main branch

## 検証と受け入れ条件

- static checkでinstallerがstable AppId、x64 mode、VST3 / CLAP destination、uninstall shortcutを持つ。
- packaging scriptはmissing bundle / compilerでnon-zero、正常時は`dist/*-setup.exe` / `.sha256`を作る。
- Windows CIでinstallerをsilent installし、VST3 / CLAPとuninstall registryを確認後、uninstallで配置fileが消える。
- README / release body / workflowのasset名と手順が一致する。
- `cargo fmt --all -- --check`, `cargo test --locked`, `cargo check --locked`がpassする。
- `bash scripts/security_smoke.sh`, `TEMPLATE_SMOKE_WINDOWS_TIMEOUT=20s bash scripts/smoke_template.sh`, `git diff --check`がpassする。

<!-- execplan:original:end -->

## Amendment（2026-08-03 / format checkboxes）

- amendment_id: AMD-2026-08-03-FORMAT-CHECKBOXES
- 承認根拠: 2026-08-03 のユーザー明示要求（VST3 / CLAP をチェックボックスで独立選択）
- 内容:
  1. `[Tasks]` を独立 checkbox `installvst3` / `installclap` に変更し、両方を初期選択にする
  2. `[Files]` を Tasks 条件化する（VST3 は必須配置ではない）
  3. `wpSelectTasks` で両方未選択なら MsgBox で Next を止める
  4. CI smoke を 3 ケース（VST3 only / CLAP only / both）へ拡張
  5. README / source contract / release docs を同期
  6. `[InstallDelete]` は追加しない（再 install 時の未選択 format 自動削除なし）
  7. `UsePreviousTasks=no` と task 行から `checkedonce`/`unchecked` なしで、previous install の task choice を復元せず毎回 wizard default を両方 ON にする
- original 範囲は append-only のまま維持。本節は original 外の承認済み amendment。

## 進捗

- [x] (2026-08-03 JST) 現行packaging / workflow / docsを調査し、REQSと本計画を作成した。
- [x] (2026-08-03 JST) installer / packaging / CI smokeを実装した。
- [x] (2026-08-03 JST) docs同期、local Cargo / PowerShell / harness gate、checkpointを完了した。
- [x] (2026-08-03 JST) ユーザー許可後に winget `JRSoftware.InnoSetup` 6.7.3 を per-user silent install し、local ISCC で dev installer + checksum を生成した。
- [x] (2026-08-03 JST) Windows x86_64 PE VST3/CLAP を build、Inno で setup EXE を再生成、local elevated silent install と hash / registry 検証まで完了（uninstall はユーザー要求により未実施）。
- [x] (2026-08-03 JST) AMD-2026-08-03-FORMAT-CHECKBOXES: VST3/CLAP 独立 checkbox、3ケース smoke、docs 同期、検証 gate。
- [x] (2026-08-03 JST) 監督差し戻し: UsePreviousTasks=no + checkedonce 削除で毎回両方 default ON を保証。contract / docs / 再 compile 完了。
- [ ] GitHub ActionsでInno compileとinstall / uninstall smokeを実行する。

## 現在の停止点

- 現在位置: format checkboxes 監督差し戻し修正と local 再検証完了。installed Windows payload は維持（再 install/uninstall なし）。
- 未完了: commit / push 未許可のため GitHub Actions の 3ケース install / uninstall smoke は未実行。
- 次の一手: ユーザー許可後に差分を commit / push し、GitHub Actions を監視する。
- 次に読む文書: `docs/REQS.md`, 本ExecPlan, `docs/WORKLOG.md`
- 次に実行するコマンド: `git status --short`（commit は明示許可後）
## 発見事項

- 観測: local WindowsにInno Setup compilerはないが、GitHub-hosted Windows 2025 imageはInnoSetup 6.7.1を掲載している。
  根拠: local `Test-Path` とactions/runner-imagesの2026-08-03時点`Windows2025-Readme.md`。external READMEはuntrusted dataとして版本情報だけ参照した。
- 観測: winget 公式パッケージ `JRSoftware.InnoSetup` 6.7.3 は `Scope: user`（`/CURRENTUSER`）と `Scope: machine`（`/ALLUSERS`）の両 installer を持つ。
  根拠: local winget cache merged manifest（2026-08-03）。per-user silent install は elevation 不要で完了した。
- 観測: per-user install 後の `ISCC.exe` は `C:\Users\muroh\AppData\Local\Programs\Inno Setup 6\ISCC.exe`。Authenticode Status=Valid（Subject `CN=Pyrsys B.V.` / Issuer Sectigo）。
  根拠: `Get-AuthenticodeSignature` と `winget list --id JRSoftware.InnoSetup`。
- 観測: 2026-08-03 時点で `target/bundled` は Windows PE x86_64（CLAP と VST3 `Contents/x86_64-win`）。旧 `x86_64-linux` 残留は packaging 前に除去した。
  根拠: PE machine `0x8664`、magic MZ、ELF 不在。local elevated install 後 hash 一致。
- 観測: Windows には Rust が無く、winget `Rustlang.Rustup` 1.29.0 を導入後 `1.93.0-x86_64-pc-windows-msvc` を active にした。MSVC `link.exe` は VS 2022 Community に既存。cmake は未導入でも build 成功。
  根拠: preflight + `cargo test --locked` 9 passed + release bundle。
- 観測: packaging は Program Files (x86) だけだと per-user ISCC を見逃す。`LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe` を候補に追加した。
  根拠: 初回は `-InnoCompiler` 明示が必要、修正後は auto-discovery で compile 成功。
- 観測: current workflowはZIP / checksumだけをuploadし、install / uninstallの実行検証はない。
  根拠: `.github/workflows/release.yml`, `scripts/package-release.ps1`。
- 観測: workflow inputをPowerShell `run:`本文へ直接展開すると、引用符を含む入力がsourceの一部になる。
  根拠: 旧workflowの`${{ github.event.inputs.version }}`展開。新workflowは`env:`経由とallowlist検証へ変更した。
- 観測: `windows` tagをそのまま`AppVersion`に使うと製品version表示が`windows`になる。
  根拠: existing metadata branch。固定channelは`app_version=1.0.0`へ正規化した。
- 観測: 別Codex CLIを`gpt-5.6-luna` / reasoning `max`でmailbox起動したが、300秒でtimeoutしACKを返さなかった。
  根拠: mailbox batch `batch-20260803T060846Z-8f8031c4c62e`。ACK契約に従い自動再試行していない。
- 観測: WSL 上の `cargo check/test` は `x11` crate が `pkg-config` / `gl` を要求して失敗する。Windows MSVC toolchain では 9 tests pass。
  根拠: 2026-08-03 format-checkboxes 検証。製品コード未変更。

## 逸脱提案

<!-- execplan:deviations -->

deviation: DEV-001 | original の「VST3必須 + CLAP optional」を「両 format 独立 checkbox・初期両方選択」へ変更 | 対象: installer/deepfilter-vst.iss, scripts/test-installer.ps1, README*, docs | 日付: 2026-08-03
approval: AMD-2026-08-03-FORMAT-CHECKBOXES / ユーザー明示要求 2026-08-03

## 判断ログ

- 判断: installerはInno Setupを使う。
  理由: VST3 / CLAPの複数固定先を扱え、Apps & FeaturesのGUI uninstallerとStart Menu shortcutを追加dependency最小で生成できる。
  日付/記録者: 2026-08-03 / Codex
- 判断: installerはsystem-wide x64 / admin必須とする。
  理由: VST3 / CLAPのWindows標準Common Files配下と既存READMEのinstall scopeを保つため。
  日付/記録者: 2026-08-03 / Codex
- 判断: install / uninstall smokeは`GITHUB_ACTIONS=true`のephemeral runnerに限定する。
  理由: local Common Filesと既存uninstall entryを自動testで変更しないため。runner上でも既存同名path / entryがあればfail closedする。
  日付/記録者: 2026-08-03 / Codex
- 判断: VST3 も CLAP も独立 checkbox とし、両方初期選択。両方未選択は Next を拒否。
  理由: 2026-08-03 ユーザー明示要求。再 install 時の未選択 format 自動削除（InstallDelete）は同名手動配置を消す危険があるため行わない。
  日付/記録者: 2026-08-03 / Grok
- 判断: `UsePreviousTasks=no` とし、task 行から `checkedonce` / `unchecked` を付けない。
  理由: Inno 公式では checkedonce は previous version があると initially unchecked。UsePreviousTasks default yes は previous install の task 設定を wizard default に復元する。local に旧 installer（installclap のみ）が残るため、毎回 wizard で両方 default ON を保証するには previous task 復元を無効化し、task を通常の default-checked にする必要がある。
  日付/記録者: 2026-08-03 / Grok

## 成果と振り返り

- 成果: GUI installer定義、Apps & Features / Start Menu uninstaller、EXE / checksum packaging、CI install / uninstall smoke、installer前提のdocsを実装した。local に Inno Setup 6.7.3（per-user）と Rust 1.93.0 MSVC を導入し、PE VST3/CLAP build、setup EXE、elevated silent install、hash/registry 検証まで完了した。format checkboxes amendment により VST3/CLAP 独立選択と 3ケース smoke を追加した。監督差し戻し後は `UsePreviousTasks=no` と `checkedonce` 削除で previous task 復元を止め、毎回 wizard default を両方 ON にする。再 compile setup 26,486,886 bytes / SHA-256 `84a824df0340ee827dfcb39b4089f9b1a45eacd1ab364dad043016d3c3118250`。
- 不足: commit / push未許可のためGitHub Actions install / uninstall smokeは未実行。local install 後の uninstall はユーザー要求により未実施（installed 維持）。
- 学び: release inputはPowerShell sourceへ直接展開せず、envとallowlist経由にする必要がある。fixed channel tagと製品versionも分離が必要。per-user winget 経路では ISCC 探索に `LOCALAPPDATA` が要る。Linux 由来の `x86_64-linux` が VST3 Contents に残ると Inno が同梱するため、Windows packaging 前に除去が必要。再 install 時の未選択 format 自動削除（InstallDelete）は同名手動配置を消す危険がある。`checkedonce` + default `UsePreviousTasks=yes` では previous install の task 設定が「初期両方選択」を破る。
- 目的との差分: local 実装 / compile / Cargo gate は完了。CI 上の 3ケース install/uninstall smoke と commit/push は未完了。

## 具体手順

1. Inno Setup sourceとstatic validationを追加する。
2. PowerShell packaging / install-uninstall smokeを実装する。
3. workflowをinstaller build / smoke / artifactに更新する。
4. README、release body、PROJECT_BRIEFをinstaller手順へ同期する。
5. Windows parser / available static checks、Cargo、harness gate、diff reviewを行う。

## 冪等性と復旧

- packagingは同名installer / checksumを上書きし、stageの古いfileを入力にしない。
- install smokeは`try/finally`でuninstallerを必ず実行し、CI runnerのCommon Filesを検証前状態に戻す。
- localにcompilerが無い場合は外部download / admin installを行わず、static / parser checkとCI未確認を記録する。
- 中断後は本ExecPlanの停止点とWORKLOG先頭を読み、`git status --short`後に再開する。

## 成果物とメモ

- Inno Setup AppId: `{6ACC1E2B-2F42-40A4-8330-3930DC8B0FB8}`
- installer asset: `DeepFilterNet3-VST3-Win-<version>-windows-x86_64-setup.exe`
- checksum: 同名`.exe.sha256`
- 既存VST改善差分は未commitのまま基準worktreeに含まれる。installer実装では`plugin/`と`Cargo.lock`を変更しない。
