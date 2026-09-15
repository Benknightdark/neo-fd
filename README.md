# Neo FD

Neo FD 是桌面檔案掃描工具，用來搜尋資料夾中的敏感內容。

## 開發與建置

從專案根目錄執行以下命令：

```text
npm --prefix neo-fd-desktop run tauri:dev
npm --prefix neo-fd-desktop run tauri:build
npm --prefix neo-fd-desktop run tauri:build:dev
```

`tauri:dev` 啟動帶有開發身份與圖示的桌面程式；`tauri:build` 建立正式安裝包；`tauri:build:dev` 建立可與正式版並存的開發安裝包。
