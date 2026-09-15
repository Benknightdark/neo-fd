# 內建敏感規則左右邊界判定

## 目的與可觀察成果

兩個內建敏感資料規則各自獨立掃描，只有在匹配內容左右兩側均為 Unicode 空白或 Unicode 標點，或位於單行起點／終點時才回報。掃描結果的 `matched_text` 只包含身分證字號或姓名本身，不包含邊界字元；自定義 Regex 維持目前行為。

## 專案背景與重要路徑

- `neo-fd-desktop/src/stores/scan.ts` 保存內建規則，並將啟用規則送往 IPC。
- `neo-fd-desktop/src/api/ipc.ts` 定義前端的 Tauri 命令型別與呼叫介面。
- `neo-fd-desktop/src-tauri/src/lib.rs` 反序列化掃描參數並建立 Rust 掃描器。
- `neo-fd-desktop/src-tauri/src/scanner.rs` 逐行執行 Regex 匹配並產生 `ScanResult`。
- `.neo_harness/ARCHITECTURE.md`、`.neo_harness/docs/reliability.md` 保存架構與失敗行為說明。

## 目前與期望行為

目前前端透過 `[name, pattern]` 二元素 tuple 傳送規則，Rust 掃描器對每個 Regex 的完整匹配直接產生結果，因此身分證字號或三字姓名嵌在其他文字中也會被回報。

期望將規則參數擴充為 `[name, pattern, requiresBoundary]`，兩個內建規則傳送 `true`，自定義規則傳送 `false`。啟用邊界的規則仍以原本的 Regex 找出核心內容，再以匹配位置檢查最近的左右字元是否符合 `\\s` 或 `\\p{P}`；缺少左側或右側字元時視為有效邊界。

## 架構與限制

- `scan_directory` 的命令名稱、掃描結果欄位、檔案走訪、取消掃描與最大結果數行為保持不變。
- 前端與 Rust 的 IPC 型別必須同步更新，避免二元素與三元素 tuple 不一致。
- Rust `regex` 版本為 `1.12.3`，不支援 look-around；邊界檢查必須在匹配後以位置與字元判定完成。
- `scanner.rs` 不得依賴前端或視窗執行期；不得引入新的檔案讀取限制或改變自定義規則語意。
- Unicode 標點使用 Regex 的 `\\p{P}`，Unicode 空白使用 `\\s`；不把一般 Unicode 符號視為標點。

## 里程碑及進度

- [x] 建立工作樹基準、閱讀架構與品質文件，確認前端 14 項測試及 Rust 2 項測試目前通過。
- [x] 建立本持久執行計畫。
- [x] 更新前端規則型別、預設旗標與 IPC tuple。
- [x] 更新 Rust 命令邊界、掃描器邊界判定與單元測試。
- [x] 同步架構／可靠性文件並執行完整驗證。
- [x] 將本計畫移至 `completed/` 並記錄完成結果。

## 發現與決策紀錄

1. 「符合其中兩個規則」確認為兩條內建規則各自獨立套用，不要求同一行同時符合兩條規則。
2. 「標題符號」依確認解讀為 Unicode 標點。
3. 「包住」確認為左右兩側都要符合；行首與行尾算自然邊界。
4. `matched_text` 必須維持敏感內容本身，因為結果表與內容檢視器使用該欄位顯示及高亮匹配內容。
5. 邊界需求以獨立旗標傳遞，而不是把不被 Rust `regex` 支援的 look-around 寫入 Regex；這也讓自定義規則維持原有完整匹配語意。

## 具體命令與預期結果

實作期間：

- `cargo fmt --manifest-path neo-fd-desktop/src-tauri/Cargo.toml`
- `cargo check --manifest-path neo-fd-desktop/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path neo-fd-desktop/src-tauri/Cargo.toml --all-targets -- -D warnings`
- `cargo test -q --manifest-path neo-fd-desktop/src-tauri/Cargo.toml`
- `npm --prefix neo-fd-desktop run lint`
- `npm --prefix neo-fd-desktop run build`
- `npm --prefix neo-fd-desktop run test -- --reporter=dot`

完成驗證：

- `sh .neo_harness/verify.sh`

預期所有命令成功；完整驗證必須涵蓋前端 lint、型別建置、前端測試、Rust 格式、Clippy 與 Rust 測試。

## 自動與手動驗證

- 前端測試確認兩條內建規則傳送 `requiresBoundary: true`，自定義規則傳送 `false`。
- Rust 測試確認行首、行尾、Unicode 空白、中英文 Unicode 標點可匹配；嵌入其他文字、只有單側符合及非標點符號邊界不可匹配。
- Rust 測試確認相鄰匹配不因共用分隔字元遺漏，且 `matched_text` 不包含邊界。
- Rust 測試保留既有無限制與最大結果數情境。
- 手動檢查結果表與內容檢視器仍只高亮敏感內容，不高亮周邊空白或標點。

## 冪等、失敗復原與回復方式

- 先修改前端型別與 Rust 命令參數，再修改掃描器，避免暫時留下兩端資料形狀不一致的完成狀態。
- 若任一驗證失敗，保留 active plan，修正並重新執行失敗命令；不得將未驗證變更標為完成。
- 回復時只移除本計畫新增的規則旗標、IPC tuple 欄位、邊界檢查、測試與文件變更，不觸碰其他工作樹內容。

## 完成結果與剩餘工作

實作已完成。`sh .neo_harness/verify.sh` 成功，包含前端 lint、型別建置、15 項前端測試、Rust 格式檢查、Clippy 與 4 項 Rust 測試。建置仍輸出來自 `node_modules/@vueuse/core` 的既有 `/* #__PURE__ */` annotation 警告，但未造成驗證失敗。未提交工作樹審查未發現可執行的重大問題；本計畫已移至 `.neo_harness/docs/exec-plans/completed/`。
