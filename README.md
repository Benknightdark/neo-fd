# Neo FD

基於 Rust 開發的高效能、多執行緒檔案掃描工具。支援快速遍歷目錄，並利用正規表達式 (Regular Expression) 比對檔案內容。內建「台灣身分證字號」與「台灣十大姓氏」掃描，且支援自定義正規表達式。

## 專案特色

*   **桌面應用支援**：提供基於 Vue 3 與 Tauri 2 的桌面圖形介面 (GUI)。
*   **極速平行掃描**：底層採用與 `ripgrep` 相同的 `ignore` crate，支援高效平行目錄走訪。
*   **零記憶體浪費**：引擎採用單一 Buffer 重複讀取，避免在大檔案掃描時頻繁分配記憶體。
*   **智慧過濾**：自動套用 `.gitignore` 排除規則，並跳過二進位與非 UTF-8 檔案，確保穩定執行。
*   **即時錯誤檢查**：桌面介面提供即時的正規表達式語法驗證。
*   **模組化設計**：核心引擎設計為 `neo-fd-desktop/src-tauri` 中的獨立 `scanner` 模組，專注於平行目錄遍歷與個資掃描，便於內部維護與 Tauri 指令綁定。

## 目錄結構

合流重構與架構精簡後，後端完全收攏為獨立 Crate，專案結構如下：

```text
neo-fd/
├── package.json             # 根 npm 腳本定義
└── neo-fd-desktop/          # 前端與 Tauri 桌面端專案
    ├── src/                 # Vue 3 前端單頁應用 (GUI)
    └── src-tauri/           # Tauri 2 / Rust 後端核心
        ├── Cargo.toml       # 整合 anyhow / ignore 依賴之核心配置
        └── src/
            ├── lib.rs       # Tauri 指令及事件生命週期管理
            ├── main.rs      # 二進位進入點
            └── scanner.rs   # 核心掃描模組，處理多執行緒走訪與正則匹配
```

## 安裝與執行

本專案已將所有前端、Tauri 後端與相關相依工具鏈（Husky、Commitlint、Linter）全數收攏至 `neo-fd-desktop/` 目錄中，根目錄保持 100% 純淨。**請確保所有的 npm 相依安裝、開發生產指令，均於 `neo-fd-desktop/` 目錄內執行。**

請確保系統已安裝 Rust、Cargo 與 Node.js。

### 1. 啟動桌面應用程式 (GUI)

```bash
cd neo-fd-desktop
npm install
npm run tauri dev
```

### 2. 編譯 Release 執行檔 (Desktop)

若要編譯桌面應用程式安裝檔：

```bash
cd neo-fd-desktop
npm run tauri build
```

安裝檔（.msi, .exe 等）將位於：
`target/release/bundle/`


## 開發與品質規範

本專案所有的品質管控指令皆已整合並收攏至 `neo-fd-desktop` 中：
*   **全域靜態檢查**：提交前請在 `neo-fd-desktop` 目錄下執行 `npm run lint:all`（進行前端 Biome 與後端 Rust fmt/clippy 檢查）。
*   **單元測試驗證**：請在 `neo-fd-desktop` 目錄下執行 `npm run test:all` 執行所有前端 Vitest 與後端 Cargo 測試。
*   **核心原則**：桌面介面與 IPC 層須妥善處理錯誤輸入，絕不 Panic；後端引擎保持零記憶體分配。

