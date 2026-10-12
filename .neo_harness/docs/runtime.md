# 執行環境

## 已確認設定

- 產品前端專案根目錄是 `neo-fd-desktop/`，以 Bun `1.4.3` 與 `neo-fd-desktop/bun.lock` 鎖定依賴。
- 前端測試專案根目錄是 `tests/frontend/`，以 Bun `1.4.3` 與 `tests/frontend/bun.lock` 鎖定依賴；兩個前端專案獨立管理。
- 後端測試專案根目錄是 `tests/backend/`，依賴鎖定於 `tests/backend/Cargo.lock`。
- `neo-fd-desktop/package.json` 定義開發伺服器、前端建置、靜態檢查與正式／開發桌面命令；測試命令集中於 `tests/frontend/package.json` 與 `tests/backend/Cargo.toml`。
- `neo-fd-desktop/src-tauri/tauri.conf.json` 將開發前端伺服器設為 `http://localhost:1420`，建置前命令為 `bun run build`，前端輸出目錄為 `../dist`。
- `neo-fd-desktop/src-tauri/tauri.dev.conf.json` 透過 Tauri 設定合併機制覆寫開發名稱、identifier、視窗標題、開發建置命令及開發圖示。
- 前端驗證與發布工作流使用 Bun `1.4.3` 及 Node.js 20；Node.js 20 執行相容性工具與 release 版本同步程式。Rust 專案使用 stable 工具鏈、`edition = "2021"`，並提交 `Cargo.lock`。
- Linux 驗證工作流會安裝桌面建置所需的系統套件；macOS 與 Windows 的完整前置條件未在專案中集中定義。

## 啟動與建置

從專案根目錄執行：

```text
bun --cwd=neo-fd-desktop ci
bun --cwd=neo-fd-desktop run dev
bun --cwd=neo-fd-desktop run tauri:dev
bun --cwd=neo-fd-desktop run build
bun --cwd=neo-fd-desktop run build:dev
bun --cwd=neo-fd-desktop run tauri:build
bun --cwd=neo-fd-desktop run tauri:build:dev
bun --cwd=tests/frontend ci
bun --cwd=tests/frontend run test:unit
bun --cwd=tests/frontend run playwright install chromium
bun --cwd=tests/frontend run test:e2e
cargo test --manifest-path tests/backend/Cargo.toml
sh .neo_harness/verify.sh
```

`run dev` 只啟動前端開發伺服器；`tauri:dev` 依 `tauri.dev.conf.json` 啟動開發身份的桌面程式；`tauri:build` 使用正式設定；`tauri:build:dev` 使用開發設定建立測試安裝包。專案未定義專用停止命令；停止方式未由設定檔固定。

## 產物與重現性

- 前端建置產物位於 `neo-fd-desktop/dist/`，由忽略規則排除。
- Rust 建置產物位於 `neo-fd-desktop/src-tauri/target/` 與 `tests/backend/target/`，由忽略規則排除。
- 兩個前端專案各自提交 `bun.lock`，並在 `package.json` 宣告 Bun `1.4.3`；CI 依 `packageManager` 欄位設定 Bun 版本。
- 沒有容器、開發容器、環境變數範本或跨平台前置條件文件。
- 前端端對端測試會由 `tests/frontend/playwright.config.ts` 啟動 `neo-fd-desktop` 的 Vite 開發伺服器，並在瀏覽器頁面初始化時提供 IPC 模擬。
- 後端端對端測試使用系統暫存目錄，測試結束後由 `tempfile` 自動清理。
