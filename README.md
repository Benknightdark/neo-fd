# Neo FD

Neo FD 是桌面檔案掃描工具，用來搜尋資料夾中的敏感內容。

## 開發與建置

從專案根目錄執行以下命令：

```text
bun --cwd=neo-fd-desktop run tauri:dev
bun --cwd=neo-fd-desktop run tauri:build
bun --cwd=neo-fd-desktop run tauri:build:dev
```

兩個 `package.json` 都宣告 Bun `1.4.3`，CI 依此設定 Bun 版本。產品前端 `neo-fd-desktop/` 與前端測試 `tests/frontend/` 是各自獨立的套件專案，分別維護 `bun.lock`。

`tauri:dev` 啟動帶有開發身份與圖示的桌面程式；`tauri:build` 建立正式安裝包；`tauri:build:dev` 建立可與正式版並存的開發安裝包。
