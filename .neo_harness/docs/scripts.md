| 指令 | 用途 |
| :--- | :--- |
| `bun --cwd=neo-fd-desktop ci` | 依鎖定檔安裝產品前端依賴；驗證工作流使用 |
| `bun --cwd=neo-fd-desktop install` | 安裝產品前端依賴 |
| `bun --cwd=neo-fd-desktop run dev` | 啟動前端開發伺服器 |
| `bun --cwd=neo-fd-desktop run build:dev` | 以開發模式建置前端 |
| `bun --cwd=neo-fd-desktop run tauri:dev` | 啟動開發身份的桌面程式 |
| `bun --cwd=neo-fd-desktop run preview` | 預覽前端建置結果 |
| `bun --cwd=neo-fd-desktop run lint` | 執行前端靜態檢查 |
| `bun --cwd=neo-fd-desktop run format` | 格式化 `src/`；會修改檔案 |
| `bun --cwd=neo-fd-desktop run build` | 執行型別檢查並建置前端 |
| `bun --cwd=neo-fd-desktop run lint:rust` | 執行 Rust 格式與靜態檢查 |
| `bun --cwd=neo-fd-desktop run tauri:build` | 建置正式桌面安裝包 |
| `bun --cwd=neo-fd-desktop run tauri:build:dev` | 建置開發桌面安裝包 |
| `bun --cwd=neo-fd-desktop run prepare` | 初始化專案 Git hooks |
| `bun --cwd=tests/frontend ci` | 依鎖定檔安裝獨立前端測試專案依賴 |
| `bun --cwd=tests/frontend run lint` | 執行前端測試專案靜態檢查 |
| `bun --cwd=tests/frontend run typecheck` | 執行前端測試專案型別檢查 |
| `bun --cwd=tests/frontend run test:unit` | 執行前端單元測試 |
| `bun --cwd=tests/frontend run playwright install chromium` | 安裝前端端對端測試瀏覽器 |
| `bun --cwd=tests/frontend run test:e2e` | 執行前端瀏覽器端對端測試 |
| `cargo fmt --manifest-path tests/backend/Cargo.toml -- --check` | 檢查後端測試專案 Rust 格式 |
| `cargo clippy --manifest-path tests/backend/Cargo.toml --all-targets -- -D warnings` | 執行後端測試專案 Rust 靜態檢查 |
| `cargo test --manifest-path tests/backend/Cargo.toml` | 執行後端單元與端對端測試 |
| `sh .neo_harness/verify.sh` | 執行所有產品與測試專案驗證 |
| `sh .neo_harness/verify.sh frontend` | 執行產品前端建置及前端測試專案驗證 |
| `sh .neo_harness/verify.sh rust` | 執行產品與後端測試專案 Rust 驗證 |
| `sh .neo_harness/verify.sh lint` | 執行提交前的前端與 Rust 靜態檢查 |
| `neo-fd-desktop/.husky/commit-msg` | 以 Bun 執行本機 `commitlint`，檢查 Git 傳入的提交訊息檔 |

產品前端與前端測試是兩個獨立套件根目錄，各自以 Bun `1.4.3` 和 `bun.lock` 固定依賴。
