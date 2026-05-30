# Neo FD1

Neo FD 是桌面檔案掃描工具，用來搜尋資料夾中的敏感內容。Rust 負責檔案走訪與內容比對，Vue 3 與 Tauri 2 提供桌面介面。

目前內建台灣身分證字號與常見姓氏相關掃描規則，也支援自訂正規表達式。

## 重點

- **桌面介面**：使用 Vue 3 與 Tauri 2 建立跨平台 GUI。
- **平行掃描**：後端使用 `ignore` crate 走訪目錄。
- **智慧過濾**：支援 `.gitignore` 規則，並跳過二進位與非 UTF-8 檔案。
- **自訂規則**：可新增正規表達式，並在介面中檢查語法。
- **錯誤隔離**：讀檔錯誤或 Regex 錯誤不會中斷整體掃描。

## 目錄結構

```text
neo-fd/
├── AGENTS.md
├── README.md
└── neo-fd-desktop/
    ├── package.json         # npm scripts 與前端相依套件
    ├── src/                 # Vue 3 前端
    └── src-tauri/           # Tauri 2 / Rust 後端
        ├── Cargo.toml
        └── src/
            ├── lib.rs       # Tauri 指令與事件橋接
            ├── main.rs
            └── scanner.rs   # 核心掃描模組
```

所有 npm 指令都在 `neo-fd-desktop/` 內執行。

## 環境需求

- Node.js
- npm
- Rust 與 Cargo
- Tauri 2 系統相依套件

## 安裝與啟動

```bash
cd neo-fd-desktop
npm install
npm run tauri dev
```

## 開發指令

執行前端與後端檢查：

```bash
cd neo-fd-desktop
npm run lint:all
```

執行前端與後端測試：

```bash
cd neo-fd-desktop
npm run test:all
```

編譯桌面安裝檔：

```bash
cd neo-fd-desktop
npm run tauri build
```

打包輸出位於 Tauri 的 `target/release/bundle/`。

## 開發重點

- 前端 UI 位於 `neo-fd-desktop/src/`。
- Tauri 指令與事件橋接位於 `neo-fd-desktop/src-tauri/src/lib.rs`。
- 核心掃描邏輯位於 `neo-fd-desktop/src-tauri/src/scanner.rs`。
- 修改掃描規則時，請同步確認前端設定、Regex 驗證與後端測試。
- 測試資料只能使用虛擬資料，不要提交真實個資。

更多協作與提交規範請參考 `AGENTS.md`。
