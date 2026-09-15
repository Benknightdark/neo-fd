# 前後端測試專案遷移執行計畫

## 目的與可觀察成果

在本專案新增獨立的前端與後端測試專案，各自提供單元測試與端對端測試；完整遷移既有測試，移除 `neo-fd-desktop` 內的舊測試實作、設定與測試依賴。前端端對端測試以瀏覽器搭配模擬 IPC 驗證使用者流程，後端端對端測試透過公開服務邊界驗證實際檔案系統行為。

可觀察成果如下：

- `tests/frontend` 可獨立安裝、型別檢查、執行單元測試與瀏覽器端對端測試。
- `tests/backend` 可獨立執行 Rust 單元測試與檔案系統端對端測試。
- 前端與後端測試均納入 CI 及 `.neo_harness/verify.sh`。
- Tauri 命令維持既有名稱、參數、結果與事件契約。
- 舊的前端測試目錄、Rust 內嵌測試、測試設定、測試腳本與測試依賴均自 `neo-fd-desktop` 移除。

## 專案背景與重要路徑

- 前端產品程式：`neo-fd-desktop/src/`
- Tauri 橋接與命令：`neo-fd-desktop/src-tauri/src/lib.rs`
- 掃描核心：`neo-fd-desktop/src-tauri/src/scanner.rs`
- 前端測試專案：`tests/frontend/`
- 後端測試專案：`tests/backend/`
- 驗證入口：`.neo_harness/verify.sh`
- CI：`.github/workflows/validate.yml`
- Harness 架構與規範：`.neo_harness/ARCHITECTURE.md`、`.neo_harness/docs/`

## 目前與期望行為

目前測試位於產品前端 `src/**/*.test.ts`、`src/**/*.spec.ts` 與 Rust 掃描器內嵌測試；測試命令、Vitest、Playwright 及相關測試依賴也位於產品專案。Playwright 尚無可執行案例。

期望行為是將既有前端案例完整移至 `tests/frontend/unit/`，將掃描器案例與擴充案例移至 `tests/backend/unit/`，並新增前端掃描及檔案操作流程與後端實際暫存檔案流程的端對端案例。產品專案只保留產品建置與靜態檢查所需設定。

## 架構與其他限制

- `neo-fd-desktop/src-tauri/src/backend.rs` 提供不依賴 Tauri 執行期的公開後端服務邊界。
- `lib.rs` 的 Tauri 命令只負責 IPC 參數轉換、事件傳遞及執行期組裝。
- `scanner.rs` 不得依賴使用者介面或視窗執行期。
- 前端 IPC 契約維持既有型別與命令名稱；瀏覽器測試在頁面初始化前建立 Tauri API 模擬。
- 後端端對端測試使用暫存目錄，不操作專案或使用者資料。
- 保留掃描器單行 65,536 位元組、略過二進位檔案及檔案讀取 5 MB 限制。
- 新增或修改的文件、註解與測試說明使用繁體中文；程式符號與必要技術名稱保留原文。
- 不納入目前工作樹中與本任務無關的變更。

## 里程碑及進度

- [x] 建立並驗證執行計畫。
- [x] 建立 Tauri 無關的後端公開服務邊界。
- [x] 建立獨立前端測試專案並遷移單元測試。
- [x] 建立前端瀏覽器端對端測試與 IPC 模擬。
- [x] 建立獨立後端測試專案並遷移及擴充測試。
- [x] 移除產品專案內舊測試實作、設定與依賴。
- [x] 更新驗證入口、CI、架構文件與操作文件。
- [x] 執行完整驗證、程式碼審查並移動計畫至完成目錄。

## 發現與決策紀錄

- 既有前端基準為 5 個測試檔、17 個測試案例；既有 Rust 基準為 4 個測試案例。
- 前端端對端測試採瀏覽器加模擬 IPC，涵蓋真實頁面組合與 IPC 參數觀察，不啟動實際 Tauri 桌面執行期。
- 後端端對端測試以公開服務邊界搭配實際暫存檔案系統，涵蓋掃描、讀取、寫入與刪除。
- 四個測試套件均在 CI 中執行；產品命令不保留舊測試入口，統一由根目錄 Harness 與獨立測試專案提供。

## 具體命令與預期結果

- `npm --prefix tests/frontend ci`：安裝獨立前端測試依賴。
- `npm --prefix tests/frontend run lint`：前端測試專案靜態檢查成功。
- `npm --prefix tests/frontend run typecheck`：前端測試專案型別檢查成功。
- `npm --prefix tests/frontend run test:unit`：前端單元測試全部成功。
- `npm --prefix tests/frontend exec -- playwright install chromium`：安裝前端端對端測試瀏覽器。
- `npm --prefix tests/frontend run test:e2e`：前端瀏覽器端對端測試全部成功。
- `cargo fmt --manifest-path tests/backend/Cargo.toml -- --check`：後端測試專案格式檢查成功。
- `cargo clippy --manifest-path tests/backend/Cargo.toml --all-targets -- -D warnings`：後端測試專案靜態檢查成功。
- `cargo test --manifest-path tests/backend/Cargo.toml`：後端單元與端對端測試全部成功。
- `sh .neo_harness/verify.sh`：所有前端、後端、靜態檢查與端對端測試成功。

## 自動與手動驗證

自動驗證包含前端 Biome、TypeScript、Vitest、Playwright，以及產品與測試專案的 Rust 格式、Clippy、Cargo 測試。手動檢查包含確認舊測試檔案與設定不再位於 `neo-fd-desktop`、確認 IPC 命令契約未變更、確認 CI 在 Linux、Windows、macOS 的安裝與命令路徑一致。

## 冪等、失敗復原與回復方式

重複執行依賴安裝、格式檢查與測試不應改變產品原始碼；Playwright 報告與暫存測試資料必須位於忽略或系統暫存位置。若任一驗證失敗，保留失敗輸出並修正對應測試或設定後重跑；若需回復，僅回復本計畫新增或修改的檔案，不使用廣泛的破壞性 Git 命令。

## 完成結果與剩餘工作

所有里程碑已完成。`sh .neo_harness/verify.sh` 成功；前端單元測試 6 個檔案共 19 件、前端瀏覽器端對端測試 2 件、後端單元測試 8 件、後端檔案系統端對端測試 2 件均成功。產品與兩個測試專案的格式、靜態檢查、型別檢查及產品建置均成功；程式碼審查未發現需阻擋交付的問題。

保留的測試邊界是瀏覽器端對端測試使用 IPC 模擬，後端端對端測試使用公開服務與暫存檔案系統；目前仍未涵蓋實際 Tauri 桌面執行期、檔案權限、符號連結及競態條件。這些是已記錄的後續測試缺口，不影響本計畫既定完成條件。
