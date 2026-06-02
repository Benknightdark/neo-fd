# Neo FD

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

## macOS 安裝疑難排解 (M系列晶片 / Intel 晶片)

由於本專案發布的安裝檔尚未配置付費 Apple 開發者憑證，當您從 GitHub 下載安裝檔並打開時，macOS 系統可能會跳出 **「已損壞，無法開啟。您應該將它移至垃圾桶」** 或 **「無法確認開發者」** 的 Gatekeeper 警告。

這並非檔案實際損壞，請依照下列步驟快速解除 macOS 系統的下載隔離限制：

1. 將解壓後的 `neo-fd-desktop.app` 或由 DMG 拖出的應用程式放入 **「應用程式」 (Applications)** 資料夾中。
2. 打開 Mac 的 **「終端機」 (Terminal)**。
3. 執行以下指令以清除下載隔離屬性：
   ```bash
   sudo xattr -cr /Applications/neo-fd-desktop.app
   ```
4. 重新雙擊點擊應用程式即可正常開啟。


## 開發重點

- 前端 UI 位於 `neo-fd-desktop/src/`。
- Tauri 指令與事件橋接位於 `neo-fd-desktop/src-tauri/src/lib.rs`。
- 核心掃描邏輯位於 `neo-fd-desktop/src-tauri/src/scanner.rs`。
- 修改掃描規則時，請同步確認前端設定、Regex 驗證與後端測試。
- 測試資料只能使用虛擬資料，不要提交真實個資。

更多協作與提交規範請參考 `AGENTS.md`。
