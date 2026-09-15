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
| `npm --prefix neo-fd-desktop run lint:rust` | 執行 Rust 格式與靜態檢查 |
| `npm --prefix neo-fd-desktop run tauri -- build` | 建置桌面安裝包 |
| `npm --prefix neo-fd-desktop run prepare` | 初始化專案 Git hooks |
| `npm --prefix tests/frontend ci` | 依鎖定檔安裝獨立前端測試專案依賴 |
| `npm --prefix tests/frontend run lint` | 執行前端測試專案靜態檢查 |
| `npm --prefix tests/frontend run typecheck` | 執行前端測試專案型別檢查 |
| `npm --prefix tests/frontend run test:unit` | 執行前端單元測試 |
| `npm --prefix tests/frontend exec -- playwright install chromium` | 安裝前端端對端測試瀏覽器 |
| `npm --prefix tests/frontend run test:e2e` | 執行前端瀏覽器端對端測試 |
| `cargo fmt --manifest-path tests/backend/Cargo.toml -- --check` | 檢查後端測試專案 Rust 格式 |
| `cargo clippy --manifest-path tests/backend/Cargo.toml --all-targets -- -D warnings` | 執行後端測試專案 Rust 靜態檢查 |
| `cargo test --manifest-path tests/backend/Cargo.toml` | 執行後端單元與端對端測試 |
| `sh .neo_harness/verify.sh` | 執行所有產品與測試專案驗證 |
| `sh .neo_harness/verify.sh frontend` | 執行產品前端建置及前端測試專案驗證 |
| `sh .neo_harness/verify.sh rust` | 執行產品與後端測試專案 Rust 驗證 |
| `sh .neo_harness/verify.sh lint` | 執行提交前的前端與 Rust 靜態檢查 |
| `npx --prefix neo-fd-desktop --no -- commitlint --cwd neo-fd-desktop --edit "$1"` | 提交訊息 Hook 的檢查入口 |
