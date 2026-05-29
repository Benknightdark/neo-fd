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

請確保系統已安裝 Rust、Cargo (Edition 2024) 與 Node.js。

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


## 開發規範

*   **格式化**：提交前請執行 `cargo fmt`。
*   **靜態檢查**：確保 `cargo clippy` 無警告。
*   **核心原則**：桌面介面與 IPC 層須妥善處理錯誤輸入，絕不 Panic；核心引擎保持零記憶體分配。
