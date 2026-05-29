# Neo FD

基於 Rust 開發的高效能、多執行緒檔案掃描工具。支援快速遍歷目錄，並利用正規表達式 (Regular Expression) 比對檔案內容。內建「台灣身分證字號」與「台灣十大姓氏」掃描，且支援自定義正規表達式。

## 專案特色

*   **桌面應用支援**：提供基於 Vue 3 與 Tauri 2 的桌面圖形介面 (GUI)。
*   **極速平行掃描**：底層採用與 `ripgrep` 相同的 `ignore` crate，支援高效平行目錄走訪。
*   **零記憶體浪費**：引擎採用單一 Buffer 重複讀取，避免在大檔案掃描時頻繁分配記憶體。
*   **智慧過濾**：自動套用 `.gitignore` 排除規則，並跳過二進位與非 UTF-8 檔案，確保穩定執行。
*   **即時錯誤檢查**：桌面介面提供即時的正規表達式語法驗證。
*   **模組化設計**：核心引擎為獨立的 `scanner-core` Library，便於整合至其他 Rust 應用（如 Web 伺服器或桌面應用）。

## 目錄結構

採用 Cargo Workspace 架構，核心邏輯與介面層徹底解耦：

```text
neo-fd/
├── Cargo.toml               # Workspace 定義檔
├── Cargo.lock
├── scanner-core/            # [Library] 核心掃描引擎，處理多執行緒走訪與正則比對
└── scanner-desktop/         # [Tauri]   桌面圖形介面 (GUI) 實作，基於 Vue 3
```

## 安裝與執行

請確保系統已安裝 Rust、Cargo (Edition 2024) 與 Node.js。

### 1. 啟動桌面應用程式 (GUI)

```bash
cd scanner-desktop
npm install
npm run tauri dev
```

### 2. 編譯 Release 執行檔 (Desktop)

若要編譯桌面應用程式安裝檔：

```bash
cd scanner-desktop
npm run tauri build
```

安裝檔（.msi, .exe 等）將位於：
`target/release/bundle/`


## 整合核心引擎

在其他 Rust 專案的 `Cargo.toml` 中加入相依路徑：

```toml
[dependencies]
scanner-core = { path = "../neo-fd/scanner-core" }
```

接著在程式碼中呼叫：

```rust
use anyhow::Result;
use regex::Regex;
use scanner_core::Scanner;
use std::path::PathBuf;

fn main() -> Result<()> {
    let patterns = vec![
        ("自定義配對".to_string(), Regex::new(r"YOUR_REGEX_HERE")?)
    ];
    // 傳入回呼函數來處理掃描結果
    let scanner = Scanner::new(patterns, |res| {
        println!("[{}:{}] {}: {}", res.path, res.line_num, res.pattern_name, res.matched_text);
    });
    scanner.scan_dir(&PathBuf::from("/path/to/scan"));

    Ok(())
}
```

## 開發規範

*   **格式化**：提交前請執行 `cargo fmt`。
*   **靜態檢查**：確保 `cargo clippy` 無警告。
*   **核心原則**：桌面介面須妥善處理錯誤輸入；核心引擎保持零記憶體分配。
