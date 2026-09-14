# Neo FD 專案規範

Neo FD 是桌面檔案掃描工具，用來搜尋資料夾中的敏感內容；檔案走訪與內容比對位於 `neo-fd-desktop/src-tauri/src/scanner.rs`。

## 開始工作

1. 閱讀 `.neo_harness/ARCHITECTURE.md` 與任務相關文件。
2. 閱讀目前實作、設定與測試，建立變更前基準。
3. 跨模組、遷移、權限、CI 或可能跨工作階段的任務，依 `.neo_harness/PLANS.md` 建立執行計畫。
4. 以最小範圍修改，完成後執行相關快速檢查與完整驗證。

## 專案知識

| 主題 | 來源 |
| :--- | :--- |
| 架構與依賴邊界 | `.neo_harness/ARCHITECTURE.md` |
| 執行計畫 | `.neo_harness/PLANS.md` 與 `.neo_harness/docs/exec-plans/` |
| 專案命令 | `.neo_harness/docs/scripts.md` |
| 執行環境 | `.neo_harness/docs/runtime.md` |
| CI 與發布 | `.neo_harness/docs/ci.md` |
| 可靠性 | `.neo_harness/docs/reliability.md` |
| 安全邊界 | `.neo_harness/docs/security.md` |
| 品質規則 | `.neo_harness/docs/quality.md` |

## 驗證

完整命令清單：`.neo_harness/docs/scripts.md`

統一驗證命令：`sh .neo_harness/verify.sh`

前端或 Rust 子系統可分別執行：

- `sh .neo_harness/verify.sh frontend`
- `sh .neo_harness/verify.sh rust`

## 工作規則

- 前端程式碼位於 `neo-fd-desktop/src/`；原生命令與事件橋接位於 `neo-fd-desktop/src-tauri/src/lib.rs`。
- 核心掃描邏輯位於 `neo-fd-desktop/src-tauri/src/scanner.rs`，不得直接依賴使用者介面或視窗執行期。
- 前端 IPC 契約與原生命令的名稱、參數及結果結構必須保持一致。
- 掃描器目前限制單行讀取為 65,536 位元組、略過二進位檔案，檔案讀取命令限制為 5 MB。
- 提交訊息遵循 Conventional Commits 1.0.0；相關檢查由 `neo-fd-desktop/.husky/` 與 `commitlint.config.js` 定義。

## 完成條件

- 要求的可觀察行為已完成，且相關測試或其他決定性證據存在。
- `sh .neo_harness/verify.sh` 成功。
- 架構、文件與執行計畫已依變更同步。
- 未把未驗證的命令、架構或環境假設寫成既成事實。
