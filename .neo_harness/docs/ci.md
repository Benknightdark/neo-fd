# CI 與發布

## 驗證工作流

`.github/workflows/validate.yml` 在 `develop` 分支推送，以及對 `main`、`master`、`develop` 建立合併請求時觸發。

- Rust 工作在 Linux、Windows 與 macOS 矩陣執行。
- 前端工作在 Linux、Windows 與 macOS 矩陣執行。
- Rust 工作快取 `neo-fd-desktop/src-tauri` 與 `tests/backend`，先準備格式化工具與靜態檢查工具，再執行 `bash .neo_harness/verify.sh rust`。
- 前端工作使用 Node.js 20，分別依兩份鎖定檔安裝 `neo-fd-desktop` 與 `tests/frontend` 依賴，再安裝 Chromium，最後執行 `bash .neo_harness/verify.sh frontend`。
- 前端工作會執行產品 lint 與建置、前端測試專案 lint、型別檢查、單元測試及瀏覽器端對端測試。
- 驗證工作完成後，`develop` 推送事件會進入自動建立合併請求的工作。

驗證腳本只呼叫專案既有的 `cargo` 與 `npm` 命令，並以 `set -eu` 保留子命令失敗狀態。本機完整入口是 `sh .neo_harness/verify.sh`。

## 發布工作流

`.github/workflows/release.yml` 在 `main`、`master` 分支推送或 `v*` 標籤推送時觸發。它會準備版本與標籤，依作業系統及處理器架構平行建置桌面安裝包，再產生發布說明並發布結果。

發布工作流會在建置前同步 `neo-fd-desktop/package.json` 與 `neo-fd-desktop/src-tauri/tauri.conf.json` 的版本欄位。簽署憑證、發布權限及 Linux 系統依賴由工作流環境提供，未由本機專案設定完整描述。

## 已知缺口

- 發布工作流未呼叫完整本機驗證入口；它依建置流程執行前端建置，未涵蓋所有 lint 與測試。
- 不同作業系統的本機桌面建置前置條件未集中定義。
- 瀏覽器端對端測試使用 IPC 模擬，未涵蓋實際 Tauri 桌面執行期。
