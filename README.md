# Rust Scanner

這是一個基於 Rust 開發的高效能、多執行緒檔案掃描工具。它能夠快速遍歷本機目錄，並利用正規表達式 (Regular Expression) 找出檔案中匹配的字串。專案內建了「台灣身分證字號」與「台灣十大姓氏」的掃描模式，並支援使用者輸入自定義的正規表達式。

## 專案特色

*   **多平台支援**：提供終端機介面 (TUI) 與現代化的桌面介面 (GUI/Tauri)。
*   **高效能多執行緒**：底層採用 `ignore` crate（與 `ripgrep` 相同底層），支援極速的平行目錄走訪。
*   **記憶體零浪費**：核心引擎採用「單一 Buffer」重複讀取策略，在掃描超大檔案時能大幅降低記憶體分配與 GC 開銷。
*   **智慧過濾**：自動尊重並套用 `.gitignore` 的排除規則，同時靜默跳過二進位或非 UTF-8 編碼的檔案，確保掃描過程穩定不中斷。
*   **互動式介面**：無論是 TUI 還是 GUI，皆提供直覺的互動體驗，包含即時的正規表達式語法錯誤檢查。
*   **模組化架構**：核心掃描引擎被獨立抽離為 `scanner-core` Library，可輕鬆被其他 Rust 專案（如 Web 伺服器、CLI 或 GUI 應用）整合。

## 目錄結構

本專案採用 Cargo Workspace 架構，將核心邏輯與命令列介面徹底解耦：

```text
rust-scanner-workspace/
├── Cargo.toml               # Workspace 定義檔
├── Cargo.lock
├── scanner-core/            # [Library] 核心掃描引擎，處理多執行緒走訪與正則比對
├── rust-scanner-cli/        # [Binary]  終端機互動介面 (TUI) 實作
└── scanner-desktop/         # [Tauri]   桌面圖形介面 (GUI) 實作，基於 Vue 3
```

## 安裝與執行

請確保您的系統已安裝 Rust、Cargo (Edition 2024) 與 Node.js。

### 1. 啟動互動式 CLI (TUI)

```bash
cd rust-scanner-workspace
cargo run --bin rust-scanner-cli
```

### 2. 啟動桌面應用程式 (GUI)

```bash
cd rust-scanner-workspace/scanner-desktop
npm install
npm run tauri dev
```

### 3. 編譯 Release 執行檔 (CLI)

若要獲得最佳的掃描效能並產生獨立的執行檔，請執行：

```bash
cd rust-scanner-workspace
cargo build --release --bin rust-scanner-cli
```

編譯完成後，執行檔將位於：
`rust-scanner-workspace/target/release/rust-scanner-cli`

## 如何將核心引擎整合至其他專案？

如果您有其他 Rust 專案需要使用這個高效能的掃描引擎，只需在該專案的 `Cargo.toml` 中加入本地相依路徑：

```toml
[dependencies]
scanner-core = { path = "../rust-scanner-workspace/scanner-core" }
```

接著在您的程式碼中呼叫：

```rust
use scanner_core::Scanner;
use regex::Regex;
use std::path::PathBuf;

fn main() {
    let patterns = vec![
        ("自定義配對".to_string(), Regex::new(r"YOUR_REGEX_HERE").unwrap())
    ];
    // 傳入回呼函數來處理掃描結果
    let scanner = Scanner::new(patterns, |res| {
        println!("[{}:{}] {}: {}", res.path, res.line_num, res.pattern_name, res.matched_text);
    });
    scanner.scan_dir(&PathBuf::from("/path/to/scan"));
}
```

## 開發與貢獻

*   **格式化**：提交程式碼前請執行 `cargo fmt`。
*   **靜態檢查**：請確保 `cargo clippy` 沒有出現警告。
*   **核心原則**：TUI 介面不應發生 Panic，須優雅處理使用者的錯誤輸入；核心引擎需保持記憶體操作的極簡化。
