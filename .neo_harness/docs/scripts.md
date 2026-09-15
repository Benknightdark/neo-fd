| 指令 | 用途 |
| :--- | :--- |
| `npm --prefix neo-fd-desktop ci` | 依鎖定檔安裝前端依賴；驗證工作流使用 |
| `npm --prefix neo-fd-desktop install` | 安裝前端依賴 |
| `npm --prefix neo-fd-desktop run dev` | 啟動前端開發伺服器 |
| `npm --prefix neo-fd-desktop run tauri -- dev` | 啟動桌面開發環境 |
| `npm --prefix neo-fd-desktop run preview` | 預覽前端建置結果 |
| `npm --prefix neo-fd-desktop run lint` | 執行前端靜態檢查 |
| `npm --prefix neo-fd-desktop run format` | 格式化 `src/`；會修改檔案 |
| `npm --prefix neo-fd-desktop run build` | 執行型別檢查並建置前端 |
| `npm --prefix neo-fd-desktop run test` | 執行前端單元測試 |
| `npm --prefix neo-fd-desktop run test:e2e` | 執行端對端測試；目前未發現 `e2e/` 測試目錄 |
| `npm --prefix neo-fd-desktop run lint:rust` | 執行 Rust 格式與靜態檢查 |
| `npm --prefix neo-fd-desktop run lint:all` | 執行前端與 Rust 靜態檢查 |
| `npm --prefix neo-fd-desktop run test:rust` | 執行 Rust 測試 |
| `npm --prefix neo-fd-desktop run test:all` | 執行前端與 Rust 測試 |
| `npm --prefix neo-fd-desktop run tauri -- build` | 建置桌面安裝包 |
| `npm --prefix neo-fd-desktop run prepare` | 初始化專案 Git hooks |
| `cd neo-fd-desktop/src-tauri && cargo fmt --check` | 檢查 Rust 格式；驗證工作流使用 |
| `cd neo-fd-desktop/src-tauri && cargo clippy -- -D warnings` | 執行 Rust 靜態檢查並將警告視為錯誤；驗證工作流使用 |
| `cd neo-fd-desktop/src-tauri && cargo test` | 執行 Rust 測試；驗證工作流使用 |
| `npm --prefix neo-fd-desktop run lint:all` | 執行前端與 Rust 靜態檢查 |
| `sh .neo_harness/verify.sh` | 執行前端與 Rust 的完整本機驗證 |
| `sh .neo_harness/verify.sh frontend` | 執行前端 lint、建置與單元測試 |
| `sh .neo_harness/verify.sh rust` | 執行 Rust 格式、靜態檢查與測試 |
| `npm run --prefix neo-fd-desktop lint:all` | 提交前 Hook 的靜態檢查入口 |
| `npx --prefix neo-fd-desktop --no -- commitlint --cwd neo-fd-desktop --edit "$1"` | 提交訊息 Hook 的檢查入口 |
