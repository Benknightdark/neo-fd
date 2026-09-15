# 執行環境

## 已確認設定

- 產品前端專案根目錄是 `neo-fd-desktop/`，依賴鎖定於 `neo-fd-desktop/package-lock.json`。
- 前端測試專案根目錄是 `tests/frontend/`，依賴鎖定於 `tests/frontend/package-lock.json`。
- 後端測試專案根目錄是 `tests/backend/`，依賴鎖定於 `tests/backend/Cargo.lock`。
- `neo-fd-desktop/package.json` 定義開發伺服器、前端建置、靜態檢查與桌面命令；測試命令集中於 `tests/frontend/package.json` 與 `tests/backend/Cargo.toml`。
- `neo-fd-desktop/src-tauri/tauri.conf.json` 將開發前端伺服器設為 `http://localhost:1420`，建置前命令為 `npm run build`，前端輸出目錄為 `../dist`。
- 驗證工作流使用 Node.js 20 與 Rust stable；Rust 專案使用 `edition = "2021"`，並提交 `Cargo.lock`。
- Linux 驗證工作流會安裝桌面建置所需的系統套件；macOS 與 Windows 的完整前置條件未在專案中集中定義。

## 啟動與建置

從專案根目錄執行：

```text
npm --prefix neo-fd-desktop ci
npm --prefix neo-fd-desktop run dev
npm --prefix neo-fd-desktop run tauri -- dev
npm --prefix neo-fd-desktop run build
npm --prefix neo-fd-desktop run tauri -- build
npm --prefix tests/frontend ci
npm --prefix tests/frontend run test:unit
npm --prefix tests/frontend exec -- playwright install chromium
npm --prefix tests/frontend run test:e2e
cargo test --manifest-path tests/backend/Cargo.toml
sh .neo_harness/verify.sh
```

`run dev` 只啟動前端開發伺服器；`run tauri -- dev` 依 `tauri.conf.json` 的設定啟動桌面開發環境。專案未定義專用停止命令；停止方式未由設定檔固定。

## 產物與重現性

- 前端建置產物位於 `neo-fd-desktop/dist/`，由忽略規則排除。
- Rust 建置產物位於 `neo-fd-desktop/src-tauri/target/` 與 `tests/backend/target/`，由忽略規則排除。
- 依賴鎖定檔存在，但本機支援的 Node.js 版本沒有在 `package.json` 以 `engines` 欄位定義。
- 沒有容器、開發容器、環境變數範本或跨平台前置條件文件。
- 前端端對端測試會由 `tests/frontend/playwright.config.ts` 啟動 `neo-fd-desktop` 的 Vite 開發伺服器，並在瀏覽器頁面初始化時提供 IPC 模擬。
- 後端端對端測試使用系統暫存目錄，測試結束後由 `tempfile` 自動清理。
