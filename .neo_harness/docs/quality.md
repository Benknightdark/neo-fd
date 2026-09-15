# 品質規則

## 靜態檢查與建置

- `neo-fd-desktop/tsconfig.json` 啟用 strict、未使用區域與參數檢查，以及未處理的 `switch` 分支檢查。
- `neo-fd-desktop/biome.json` 啟用格式化與推薦 lint 規則；`npm run lint` 檢查 `src/`。
- `npm run build` 先執行 `vue-tsc --noEmit`，再執行前端建置。
- Rust 由 `cargo fmt --check` 與 `cargo clippy -- -D warnings` 檢查；警告視為失敗。
- 本次前端建置成功，但顯示依賴套件的 `/* #__PURE__ */` 註解位置警告；目前未將此警告視為建置失敗。

## 測試與提交檢查

- 前端測試專案位於 `tests/frontend/`，以 `unit/**/*.spec.ts` 執行 Vue 元件、Store、工具與 IPC 輔助函式單元測試。
- 前端瀏覽器端對端測試位於 `tests/frontend/e2e/`，使用 Playwright、Vite 開發伺服器及頁面初始化時的 IPC 模擬。
- 後端測試專案位於 `tests/backend/`，以 `unit/scanner.rs` 執行掃描與正則服務測試，以 `e2e/filesystem.rs` 使用暫存檔案系統執行檔案服務流程測試。
- `neo-fd-desktop/src-tauri/src/backend.rs` 是後端測試可呼叫的 Tauri 無關服務邊界；產品 Tauri 命令只負責橋接。
- `neo-fd-desktop/.husky/pre-commit` 執行 `.neo_harness/verify.sh lint`。
- `neo-fd-desktop/.husky/commit-msg` 依 `commitlint.config.js` 檢查提交訊息。
- 驗證工作流會在三種作業系統矩陣中執行產品與四套測試檢查；本機與工作流共用 `.neo_harness/verify.sh`。

## 已知缺口

- 沒有測試覆蓋率門檻、效能門檻或架構依賴自動檢查。
- 尚無涵蓋實際 Tauri 桌面執行期、權限錯誤或跨平台畫面的自動測試。
- 沒有由專案設定指定的定期清理或技術債責任人。
