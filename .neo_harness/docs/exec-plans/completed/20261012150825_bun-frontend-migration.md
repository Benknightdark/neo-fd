# 前端套件流程遷移至 Bun

## 目的與可觀察成果

將 Neo FD 桌面產品與前端測試專案的套件安裝及命令執行改為 Bun。

完成後應可觀察到：

- `neo-fd-desktop/` 與 `tests/frontend/` 各自在 `package.json` 宣告 Bun `1.4.3`，並維護自己的 `bun.lock`；CI 依宣告設定 Bun 版本。
- 前端安裝、驗證、Tauri 前置命令、Playwright 與 commitlint hook 不再呼叫 npm 或 npx。
- CI 與 release 工作流可使用 Bun 安裝及執行前端流程；Node.js 20 保留給 release 版本同步與相容性需求。
- 開發文件描述新的 Bun 命令與鎖定檔。
- Rust、IPC 契約及應用程式公開介面不變。

## 專案背景與重要路徑

- 產品套件與命令：`neo-fd-desktop/package.json`
- 前端測試套件與命令：`tests/frontend/package.json`
- Tauri 命令設定：`neo-fd-desktop/src-tauri/tauri.conf.json`、`neo-fd-desktop/src-tauri/tauri.dev.conf.json`
- 統一驗證入口：`.neo_harness/verify.sh`
- 驗證與發布工作流：`.github/workflows/validate.yml`、`.github/workflows/release.yml`
- 開發文件：`README.md`、`.neo_harness/ARCHITECTURE.md`、`.neo_harness/docs/`

兩個前端目錄是獨立套件專案，現有 `package-lock.json` 均為 lockfile v3；不建立根 workspace。專案沒有 Bun 設定或鎖定檔。

## 目前與期望行為

目前兩個前端套件專案以 npm lockfile 安裝依賴，腳本、驗證入口、Tauri 設定、Playwright、Git hook 及 CI／release 工作流有 npm 或 npx 命令。期望上述維護中的前端流程改用 Bun，並保留 Node.js 20 執行 release 版本同步程式。

## 架構與其他限制

- 兩個 `package.json` 都宣告 Bun `1.4.3`；CI 與 release 工作流依此設定 Bun 版本。
- 產品與測試專案各自維護 `bun.lock`，移除對應的 npm lockfile。
- CI 使用凍結安裝，保留現有三平台矩陣與 Playwright Chromium 安裝條件。
- 不批次修改已完成執行計畫中的歷史 npm 命令；同步仍在 active 的桌面 flavor 計畫。
- Git hook 維持從產品套件執行已安裝的 commitlint，並確保提交訊息路徑在切換工作目錄後仍正確。

## 里程碑及進度

- [x] 盤點套件專案、npm 引用、工作流與文件；確認工作樹原先乾淨。
- [x] 設定兩份 package manifest 與 Bun lockfile，移除 npm lockfile。
- [x] 遷移 Tauri、Playwright、Git hook、驗證腳本與 GitHub Actions。
- [x] 同步 README、Harness 文件及仍進行中的執行計畫。
- [x] 執行前端與完整驗證，記錄結果並封存本計畫。

## 發現與決策紀錄

- 2026-10-12：使用者選擇完整遷移產品前端與測試專案，改用 `bun.lock`，並保留 CI 的 Node.js 20。
- 2026-10-12：兩份既有 npm lockfile 均為 v3；兩個前端專案維持獨立管理。
- 2026-10-12：Bun `1.4.3` 從兩份 npm lockfile 轉換出 `bun.lock`；移除舊 lockfile 後，兩個專案的凍結安裝均通過。
- 2026-10-12：CI 與 release 改用 `oven-sh/setup-bun@v2` 讀取產品 manifest 的 `packageManager` 版本；Node.js 20 保留。
- 2026-10-12：前端驗證、commitlint hook smoke test、正式及開發 flavor 的 Tauri `--no-bundle` 建置、完整 Harness 驗證皆通過。

## 具體命令與預期結果

在兩個 package root 分別轉換 lockfile：

```text
bunx --package=bun@1.4.3 bun install --lockfile-only
```

由專案根目錄執行安裝與驗證：

```text
bun --cwd=neo-fd-desktop ci
bun --cwd=tests/frontend ci
sh .neo_harness/verify.sh frontend
bun --cwd=neo-fd-desktop run tauri:build -- --no-bundle
bun --cwd=neo-fd-desktop run tauri:build:dev -- --no-bundle
sh .neo_harness/verify.sh
```

lockfile 轉換命令分別在兩個 package root 執行；預期各產生與 manifest 同步的 `bun.lock`。驗證命令由專案根目錄執行，且已全部成功。

日常使用範例：

```text
bun --cwd=neo-fd-desktop run tauri:dev
bun --cwd=neo-fd-desktop run tauri:build
bun --cwd=neo-fd-desktop run tauri:build:dev
```

預期各命令使用對應 Tauri flavor 與 Bun 前端前置命令。

## 自動與手動驗證

`sh .neo_harness/verify.sh frontend` 與完整 `sh .neo_harness/verify.sh` 均使用 Bun `1.4.3` 成功。前端結果為 8 個單元測試檔、22 項單元測試及 2 項瀏覽器端對端測試全部通過；完整入口中的產品與後端 Rust 檢查及測試也全部通過。正式與開發 flavor 的 Tauri `--no-bundle` 建置均成功。有效 Conventional Commit 暫存訊息通過 commitlint hook smoke test。

前端建置仍顯示既有 `@vueuse/core` PURE 註解位置警告；警告不影響建置結果。

## 冪等、失敗復原與回復方式

- Bun lockfile 由現有 lockfile 轉換；如轉換或凍結安裝失敗，先保留 npm lockfile 並檢查差異，不刪除依賴來源。
- 如遷移後驗證失敗，可回復本次變更，還原兩份已追蹤 npm lockfile 及相關命令設定。
- 本次變更不修改 Rust 程式、掃描資料或 IPC 行為。

## 完成結果與剩餘工作

遷移已完成。兩個套件專案使用 Bun `1.4.3` 與各自的 `bun.lock`，套件命令、Tauri 設定、Playwright、Git hook、CI、release 及開發文件均已同步；沒有剩餘工作。
