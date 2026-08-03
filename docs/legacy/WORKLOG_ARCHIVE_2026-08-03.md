# WORKLOG archive — 2026-08-03

### 2026-08-01 JST（Universal Agent Harness v2.6.1 移行 — 完了）

- 目的: 指定 GitHub Release の clean harness を製品固有ファイルを保持して導入する
- source: v2.6.1 release asset、215119 bytes、101 files、SHA-256 `d82c17e…aab7`
- 完了: tag `v2.6.1` と release date `20260731` を含む asset verifier、branch 作成、101/101 path 導入、PROJECT_BRIEF / REQS 初期化、`.gitignore` 統合
- 保持: Rust source / manifests、製品 README 2本、`.github/`、`scripts/package-release.ps1`
- 配布物調整: upstream asset の whitespace 3 source 箇所を正規化し、`.claude/skills/` mirror を同期
- 検証: `sync_shared_context.py --check`、`security_smoke.sh`、full `smoke_template.sh`、Windows optional を省いた修正後 smoke、tracked / untracked whitespace check、変更禁止範囲の diff check が pass
- 未確認: `cargo fmt --all -- --check` は WSL / Windows とも `cargo` が PATH に無く実行不能。製品コードは無変更
- 未完了: なし。commit / push は非目標のため未実施
- ブロッカー: なし
- 次の一手: 必要なら `git status --short` から commit review を開始する

### 2026-08-01 JST（VST processing reliability改善 — checkpoint完了）

- 目的: host block時間軸、latency、dry/wet、reset、real-time safety、smoothing、tests、dependency pinの監査findingsを改善する
- 実装: `plugin/src/stream.rs`の固定容量adapter、`plugin/src/lib.rs`のhost latency通知 / aligned dry / model reset / sample smoothing / lock・wrapper allocation除去。DeepFilterNetは`d375b2d8309e0935d165700c91da9de862a99c31`へ固定し、CIはRust 1.93.0と`--locked`を使う
- 検証: `cargo fmt --all -- --check`、`cargo test --locked`（9 passed）、`cargo check --locked`、release bundle生成、`security_smoke.sh`、`smoke_template.sh`、`git diff --check`がpass。bundleは`target/bundled/deepfilter-vst.{clap,vst3}`へ生成
- build環境: WSLにlibGL development symlinkが無かったため初回test linkは`-lGL`で失敗。adminを使わずUbuntu packageを`/tmp/deepfilter-builddeps/root`へ展開し、`LIBRARY_PATH` / `LD_LIBRARY_PATH` / `RUSTFLAGS=-L native=...`を指定して再実行した
- 判断: `DfTract`がnon-`Send`なため、working / pristineと内部`Rc`別名を1つの`ExclusiveModels`へ閉じた`unsafe impl Send`だけを残し、`Sync`は削除した。`DfTract::clone`、tract operation state、`Tensor::deep_clone`のupstream実装を確認し、pristine modelによるresetを採用
- 残リスク: libDF / tract内部の推論allocation、排他所有newtypeの最小unsafe `Send`、DaVinci Resolve 20のWindows実機offline export未検証
- Go / No-Go: source / Linux bundle検証はGo。Windows release公開はResolve実機検証までNo-Go
- branch / commit: `codex/vst-processing-reliability`（基準`4c9fd6e`）。未commit・未push
- 次の一手: Windows 48 kHz projectでcurrent sourceのbundleを読み込み、Deliver exportが非無音か確認する

### 2026-08-03 JST（Windows GUI installer / uninstaller — local checkpoint完了）

- 目的: 手動コピーZIPをGUI installerへ置き換え、Windows Apps & FeaturesとStart Menuからuninstall可能にする
- 実装: `installer/deepfilter-vst.iss`、EXE / checksum生成用`package-release.ps1`、CI限定`test-installer.ps1`、source contract test、`windows-2025` workflow、README / release body / PROJECT_BRIEFを追加・更新
- security境界: installerはadminを明示しCommon Filesの固定pathだけへ配置。smokeはGitHub Actions以外を拒否し、既存同名path / uninstall entryを検出したら停止。workflow inputはenv + allowlist検証経由
- 検証済み: PowerShell parser 3本、installer source contract、invalid version / local smoke拒否、compiler不在のfail closed、YAML parse、`cargo fmt --all -- --check`、`cargo test --locked`（9 passed）、`cargo check --locked`、security / template smoke、`git diff --check`
- branch / publish: `codex/vst-processing-reliability`の未commit VST改善差分の上に追加。commit / push / releaseは未実行
- 次の一手: ユーザー許可後にcommit / pushし、GitHub Actionsのinstaller jobを監視する
- 退避理由: WORKLOG 直近3エントリ制限。後続 TASK-INSTALLER-FORMAT-CHECKBOXES 完了時に archive

### 2026-08-03 JST（local Inno Setup install + compile validation — mailbox TASK-INNO-LOCAL）

- 目的: ユーザー許可済みの公式 winget 経路で Inno Setup を入れ、dev installer を compile して checksum まで検証する
- 実施:
  1. read-only: `winget show --id JRSoftware.InnoSetup --exact` → 6.7.3 / jrsoftware.org。manifest は `Scope: user`（`/CURRENTUSER`）と `Scope: machine`（`/ALLUSERS`）
  2. install: `winget install --id JRSoftware.InnoSetup --exact --scope user --accept-package-agreements --accept-source-agreements --disable-interactivity --silent` → exit 0、admin 不要（`AGENT_ADMIN_APPROVED` 未使用）
  3. 検証: `winget list` に 6.7.3。`ISCC.exe` = `C:\Users\muroh\AppData\Local\Programs\Inno Setup 6\ISCC.exe`。`Get-AuthenticodeSignature` Status=Valid（Subject `CN=Pyrsys B.V.` / Issuer Sectigo R36 / Thumbprint `E0AB19C8D38CBF9C44709925122A7A02F8C70CB7`）。秘密鍵・secrets は未読
  4. packaging: `scripts/package-release.ps1 -Version dev` で `dist/DeepFilterNet3-VST3-Win-dev-windows-x86_64-setup.exe`（29,172,025 bytes）と `.sha256` を生成。`Get-FileHash` 一致: `ef0d9289ad532e51c6f931fa96d58fa6ad235e70950f2f8fb95f98b8b0bffe08`
  5. packaging 修正: per-user ISCC を auto-discover するため `LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe` 候補を追加。再 compile でも auto-discovery 成功
- 明示しなかったこと: 生成 EXE の実行、Common Files 変更、commit / push / publish / tag、existing release asset 触手、secrets 読取、test 弱体化
- 根拠（当時非配布）: 当時 `target/bundled` は Linux ELF。後続 task で PE に置換し install 済み
- 検証済み: PowerShell parser 3本、installer source contract、checksum、security_smoke、smoke_template、git diff --check
- mailbox: batch `batch-20260803T064548Z-87b363dc9a67` / task `TASK-INNO-LOCAL-20260803` / receiver `grok-builder--20260803T064505Z--a2029df9`
- 退避理由: WORKLOG 直近3エントリ制限（checkpoint 差し戻し batch-20260803T092601Z-e11200231158）
