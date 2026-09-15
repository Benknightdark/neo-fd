# 執行環境

## 已確認設定

- 前端專案根目錄是 `neo-fd-desktop/`，依賴鎖定於 `neo-fd-desktop/package-lock.json`。
- `neo-fd-desktop/package.json` 定義開發伺服器、前端建置、測試、靜態檢查與桌面命令。
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
```

`run dev` 只啟動前端開發伺服器；`run tauri -- dev` 依 `tauri.conf.json` 的設定啟動桌面開發環境。專案未定義專用停止命令；停止方式未由設定檔固定。

## 產物與重現性

- 前端建置產物位於 `neo-fd-desktop/dist/`，由忽略規則排除。
- Rust 建置產物位於 `neo-fd-desktop/src-tauri/target/`，由忽略規則排除。
- 依賴鎖定檔存在，但本機支援的 Node.js 版本沒有在 `package.json` 以 `engines` 欄位定義。
- 沒有容器、開發容器、環境變數範本或跨平台前置條件文件。
- 目前沒有 `neo-fd-desktop/e2e/` 目錄，因此端對端測試命令沒有專案測試檔案可執行。
