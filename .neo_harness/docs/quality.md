# 品質規則

## 靜態檢查與建置

- `neo-fd-desktop/tsconfig.json` 啟用 strict、未使用區域與參數檢查，以及未處理的 `switch` 分支檢查。
- `neo-fd-desktop/biome.json` 啟用格式化與推薦 lint 規則；`npm run lint` 檢查 `src/`。
- `npm run build` 先執行 `vue-tsc --noEmit`，再執行前端建置。
- Rust 由 `cargo fmt --check` 與 `cargo clippy -- -D warnings` 檢查；警告視為失敗。
- 本次前端建置成功，但顯示依賴套件的 `/* #__PURE__ */` 註解位置警告；目前未將此警告視為建置失敗。

## 測試與提交檢查

- 前端測試位於 `neo-fd-desktop/src/App.test.ts`、`neo-fd-desktop/src/components/ScanSidebar.test.ts`、`neo-fd-desktop/src/stores/scan.test.ts` 與 `neo-fd-desktop/src/utils/resultTree.test.ts`，由 `neo-fd-desktop/vitest.config.ts` 收集 `*.spec.ts` 與 `*.test.ts`。
- Rust 單元測試位於 `neo-fd-desktop/src-tauri/src/scanner.rs`。
- `neo-fd-desktop/.husky/pre-commit` 執行前端與 Rust 靜態檢查。
- `neo-fd-desktop/.husky/commit-msg` 依 `commitlint.config.js` 檢查提交訊息。
- 驗證工作流會在三種作業系統矩陣中執行 Rust 與前端檢查；本機與工作流共用 `.neo_harness/verify.sh`。

## 已知缺口

- 沒有測試覆蓋率門檻、效能門檻或架構依賴自動檢查。
- 沒有 `e2e/` 測試檔案，雖然 `package.json` 定義了端對端測試命令。
- 沒有針對檔案讀寫刪除、權限錯誤、原生事件或跨平台畫面的自動測試。
- 沒有由專案設定指定的定期清理或技術債責任人。
