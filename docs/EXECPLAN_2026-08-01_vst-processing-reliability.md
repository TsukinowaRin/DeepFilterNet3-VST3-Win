# VST streaming / latency / real-time reliability 改善

plan_id: PLAN-2026-08-01-VST-RELIABILITY
基準commit: 4c9fd6e
plan_revision: 2

<!-- execplan:original:start -->

## 目的 / 全体像

現在の`process()`を、host block単位の可変長入出力とDeepFilterNetの固定480-sample frameを正しく橋渡しするstream adapterへ置き換える。adapterは常にhost buffer全体を書き戻し、固定latencyを申告し、同じlatencyのdry signalとwet signalをmixする。audio callbackからwrapper由来のallocation、Mutex、O(N²)走査、手動unsafe auto-traitを除去する。

## 背景と見取り図

現実装は出力queueがhost block全体を満たさないと入力bufferを未変更のまま返し、後のblockへ古いwetを適用する。DeepFilterNet固有latencyはhostへ未申告で、dryはcurrent blockのためwetと整列しない。`reset()`はVecだけをclearし、model stateを復元しない。処理loopはhopごとのArray allocation、Mutex、Vec drain、`iter_samples().nth(i)`を含む。

## 作業計画

1. 固定長のinput/output frame、wet ring、dry delay lineを持つ`StreamState`を追加する。
2. sampleを一度だけ走査し、hop境界で同期推論して次のwet frameをpublishする。model error時は同じlatencyのdryへfallbackする。
3. model latency + 1 hopのadapter latencyをhostへ通知し、dry / wet / gain / mixをsample単位で処理する。
4. pristine model snapshotからreset時にstateを復元し、manual unsafe wrapperをdirect model ownershipへ置き換える。
5. StreamStateをfake identity outputでtestし、chunk分割、latency、mix、reset、error fallbackを検証する。
6. DeepFilterNetをcommit pinしたgit dependencyへ変更し、CI toolchain / locked buildとREADMEを同期する。

## 予定変更範囲

- 予定変更ファイル:
  - `plugin/src/lib.rs`
  - `plugin/Cargo.toml`, `Cargo.lock`
  - `.github/workflows/release.yml`
  - `README.md`, `README_ja.md`
  - `docs/REQS.md`, `docs/WORKLOG.md`, 本ExecPlan
- 許容する付随変更:
  - 実装を分離する`plugin/src/*.rs`とunit / integration test
  - formatterによる対象Rust sourceの整形
- 変更禁止範囲:
  - plugin / parameter / VST3 / CLAP ID
  - release packaging scriptと既存release asset
  - harness hooks / permissions / gates

## 検証と受け入れ条件

- unit testsでhost chunk sizeを変えても同じstream outputになる。
- latency、dry/wet alignment、reset、model error fallback、mono/stereo interleaveをtestする。
- `cargo fmt --all -- --check`, `cargo test --locked`, `cargo check --locked`がpassする。
- `bash scripts/security_smoke.sh`, `TEMPLATE_SMOKE_WINDOWS_TIMEOUT=20s bash scripts/smoke_template.sh`, `git diff --check`がpassする。
- final diffにunsafe Sync、process内Mutex / wrapper allocation / drain / repeated nthがなく、残るunsafe Sendが排他的所有newtypeへ限定される。

<!-- execplan:original:end -->

## 進捗

- [x] (2026-08-01 JST) 監査findingsをREQSと本計画へ正規化した。
- [x] (2026-08-01 JST) stream / latency / reset実装とpure adapter testsを追加し、full cargo checkを通した。
- [x] (2026-08-01 JST) dependency / CI / READMEを同期した（最終検証前）。
- [x] (2026-08-01 JST) Cargo / harness gate、bundle build、final diff review、checkpointを完了した。

## 現在の停止点

- 現在位置: code、tests、dependency pin、CI / README同期、全gateが完了。
- 未完了: DaVinci Resolve 20のWindows実機によるoffline export再検証（今回の非目標）。
- 次の一手: Windows 48 kHz projectでcurrent sourceのbundleを読み込み、Deliver exportが非無音か確認する。
- 次に読む文書: `docs/REQS.md`, 本ExecPlan, `plugin/src/lib.rs`
- 次に実行するコマンド: `cargo test --locked`

## 発見事項

- 観測: libDFの`DfTract::process()`自身にもTensor生成・cloneがあり、このrepoのwrapper変更だけでは完全なallocation-free推論にならない。
  根拠: pinned upstream `libDF/src/tract.rs`の`process_raw()`とsynthesis path。
- 観測: NIH-plugは`initialize()`のlatency通知と、sample単位`Smoother::next()` / block単位`next_block()`を提供する。
  根拠: pinned nih-plug commit `28b149e`の`InitContext` / `Smoother`契約。
- 観測: `DfTract`をdirect fieldにすると`Rc<Tensor>`と`dyn OpState`がnon-`Send`なため、NIH-plugの`Plugin: Send`をcompile時に満たせない。
  根拠: user-local X11/OpenGL build環境での`cargo check --locked`が該当auto-traitを正確に報告した。
- 観測: `DfTract`のderived `Clone`はtract operation stateをcloneし、`Tensor::clone()`は`deep_clone()`を呼ぶ一方、model plan / `TValue`の`Rc`別名は残り得る。
  根拠: pinned DeepFilterNet `libDF/src/tract.rs`、tract-core 0.21.4 `src/plan.rs`、tract-data 0.21.4 `src/tensor.rs`。working / pristineは1つの`ExclusiveModels`に閉じ、thread間を常に一緒に移動する。

## 逸脱提案

<!-- execplan:deviations -->

deviation: DEV-001 | dependency pinとlocked buildの正本をproject fast pathへ同期 | 対象: docs/PROJECT_BRIEF.md | 日付: 2026-08-01
approval: DEV-001 | ユーザーのVST監査・計画・改善依頼に含まれる関連docs同期として採用 | 日付: 2026-08-01

## 判断ログ

- 判断: async workerではなく同期推論を維持する。
  理由: offline rendererはwall-clockより速くcallbackを呼ぶため、worker方式はqueue underrunを新たに作る。wrapper allocationは除去し、libDF内部allocationは残リスクとして分離する。
  日付/記録者: 2026-08-01 / Codex
- 判断: adapter latencyを1 hop固定で追加する。
  理由: host block sizeと無関係に、完全なmodel frameを受け取る前の出力を決定できるようにするため。
  日付/記録者: 2026-08-01 / Codex
- 判断: dependencyはpath + CI cloneではなくgit revへ固定する。
  理由: local / CI / releaseで同じDeepFilterNet sourceを選ぶため。
  日付/記録者: 2026-08-01 / Codex
- 判断: unsafeを完全除去せず、`Sync`を削除して排他的所有newtypeの`Send`だけを残す。
  理由: worker thread化はoffline rendererがwall-clockより速い場合にqueue underrunを作る。同期処理とmodelの全alias同時移動を維持しつつ、参照を外へ出さない境界が最小変更であるため。
  日付/記録者: 2026-08-01 / Codex
- 判断: `docs/PROJECT_BRIEF.md`もdependency / locked build契約へ同期する。
  理由: AGENTS / checkpointが要求するproject fast pathの同期であり、実装の正しい再開に必要な付随docs変更だから。
  日付/記録者: 2026-08-01 / Codex

## 成果と振り返り

- 成果: host block非依存のstreaming、1,920-sample latency通知とdry整列、model / adapter reset、aligned fallback、sample単位smoothingを実装した。DeepFilterNet / Rust / lockfileのrelease再現性も固定した。
- 不足: DaVinci Resolve 20実機のoffline exportは未検証。libDF / tract内部の推論allocationは残る。
- 学び: offline rendererではwall-clock前提のworkerより、callbackに完結する固定stream adapterの方がunderrunを避けられる。
- 目的との差分: `DfTract`がnon-`Send`なためunsafe完全除去はできず、排他所有newtypeの`unsafe impl Send`だけを残した。

## 具体手順

1. StreamStateのring / delay操作とpure unit testsを書く。
2. Plugin lifecycleへmodel initialize / latency / resetを接続する。
3. process loopを単一sample走査へ置換する。
4. dependency / workflow / READMEをpin契約へ更新しlockfileを再生成する。
5. format、test、check、harness smoke、diff reviewを順に実行する。

## 冪等性と復旧

- 中断後の再開手順: 本ExecPlanの停止点と先頭のWORKLOGを読み、`git status --short`後に未完了checkboxから再開する。
- code変更がcompileしない場合は対象fileだけを修正し、既存migration commitsやmainを巻き戻さない。
- dependency pinが取得不能ならpath dependencyへ戻さず、取得失敗を記録して停止する。

## 成果物とメモ

- audit対象product commit: `e23b332`
- pinned nih-plug: `28b149ec4d62757d0b448809148a0c3ca6e09a95`
- pinned DeepFilterNet: `d375b2d8309e0935d165700c91da9de862a99c31`
