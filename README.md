# Rust Scanner

這是一個基於 Rust 開發的高效能、多執行緒檔案掃描工具。它能夠快速遍歷本機目錄，並利用正規表達式 (Regular Expression) 找出檔案中匹配的字串。專案內建了「台灣身分證字號」與「台灣十大姓氏」的掃描模式，並支援使用者輸入自定義的正規表達式。

## 專案特色

*   **高效能多執行緒**：底層採用 `ignore` crate（與 `ripgrep` 相同底層），支援極速的平行目錄走訪。
*   **記憶體零浪費**：核心引擎採用「單一 Buffer」重複讀取策略，在掃描超大檔案時能大幅降低記憶體分配與 GC 開銷。
*   **智慧過濾**：自動尊重並套用 `.gitignore` 的排除規則，同時靜默跳過二進位或非 UTF-8 編碼的檔案，確保掃描過程穩定不中斷。
*   **互動式介面 (TUI)**：提供直覺的終端機互動介面，包含即時的正規表達式語法錯誤檢查。
*   **模組化架構**：核心掃描引擎被獨立抽離為 `scanner-core` Library，可輕鬆被其他 Rust 專案（如 Web 伺服器或 GUI 應用）整合。

## 目錄結構

本專案採用 Cargo Workspace 架構，將核心邏輯與命令列介面徹底解耦：

```text
rust-scanner-workspace/
├── Cargo.toml               # Workspace 定義檔
├── Cargo.lock
├── scanner-core/            # [Library] 核心掃描引擎，處理多執行緒走訪與正則比對
└── rust-scanner-cli/        # [Binary]  終端機互動介面 (TUI) 實作
```

## 安裝與執行

請確保您的系統已安裝 Rust 與 Cargo (Edition 2024)。

### 1. 啟動互動式 CLI (TUI)

```bash
cd rust-scanner-workspace
cargo run --bin rust-scanner-cli
```

啟動後，您可以使用鍵盤：
*   **上下方向鍵 (`↑` / `↓`)**：選擇內建的掃描模式（身分證字號或十大姓氏）。
*   **直接輸入**：在下方的輸入框打字，可自定義正規表達式。若語法錯誤，介面會以紅色文字即時提示。
*   **Enter**：確認模式後，進入路徑輸入畫面。可輸入特定路徑（如 `tests/data`），若直接按 Enter 則預設掃描全機。
*   **Esc**：返回上一步或離開程式。

### 2. 編譯 Release 版本

若要獲得最佳的掃描效能，強烈建議編譯 Release 版本：

```bash
cd rust-scanner-workspace
cargo build --release
```
編譯完成後，執行檔將位於 `target/release/rust-scanner-cli`。

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
    let scanner = Scanner::new(patterns);
    scanner.scan_dir(&PathBuf::from("/path/to/scan"));
}
```

## 開發與貢獻

*   **格式化**：提交程式碼前請執行 `cargo fmt`。
*   **靜態檢查**：請確保 `cargo clippy` 沒有出現警告。
*   **核心原則**：TUI 介面不應發生 Panic，須優雅處理使用者的錯誤輸入；核心引擎需保持記憶體操作的極簡化。
