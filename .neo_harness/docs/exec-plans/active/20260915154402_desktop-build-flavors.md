# Neo FD 開發版與正式版分離

## 目的與可觀察成果

讓 Neo FD 的開發執行與測試安裝包使用獨立的桌面應用程式身份，能與正式安裝版並存，並以不同圖示清楚辨識開發版。

完成後應可觀察到：

- 正式版維持 `neo-fd-desktop` 與 `com.ben.neo-fd-desktop`。
- 開發版使用 `neo-fd-desktop-dev` 與 `com.ben.neo-fd-desktop.dev`。
- 開發視窗標題為 `neo-fd-desktop 開發版`。
- 開發版使用帶有 `DEV` 標記的品牌圖示。
- 開發版與正式版的應用程式資料及 WebView 資料隔離。
- 開發伺服器與開發安裝包均顯示「開發」，正式建置顯示「正式」。

## 專案背景與重要路徑

- Tauri 基礎設定：`neo-fd-desktop/src-tauri/tauri.conf.json`
- 開發 flavor 設定：`neo-fd-desktop/src-tauri/tauri.dev.conf.json`
- npm 與 Vite 建置入口：`neo-fd-desktop/package.json`
- 前端環境標籤：`neo-fd-desktop/src/utils/appMeta.ts`
- 正式圖示：`neo-fd-desktop/src-tauri/icons/`
- 開發圖示：`neo-fd-desktop/src-tauri/icons/dev/`
- 前端設定契約測試：`tests/frontend/unit/desktopFlavor.spec.ts`

## 目前與期望行為

目前只有一份 `tauri.conf.json`；`tauri dev`、`tauri build` 與所有圖示共用正式身份。前端以 `import.meta.env.DEV` 判斷環境，因此可安裝建置使用正式 Vite 模式時會顯示「正式」。

期望以 Tauri `--config` 合併機制提供開發 flavor：

- `tauri:dev` 使用開發設定。
- `tauri:build` 使用基礎正式設定。
- `tauri:build:dev` 使用開發設定並以 `development` mode 建置前端。
- Rust IPC、掃描器、檔案服務與權限契約不變。

## 架構與其他限制

- 開發設定只覆寫桌面 metadata、視窗標題、開發建置命令及圖示，不修改 Rust 執行邏輯。
- `app.windows` 與 `bundle.icon` 是陣列；開發設定必須完整列出需要保留的項目。
- 正式發布工作流不帶開發設定，版本同步只修改基礎 `tauri.conf.json`。
- 開發圖示以目前品牌圖形為基礎，在右下角加入高對比 `DEV` 標記；正式圖示不得覆寫。
- 開發與正式版本使用不同 identifier，以隔離系統應用程式身份與 WebView 資料。

## 里程碑及進度

- [x] 新增開發圖示來源與跨平台圖示產物。
- [x] 新增 Tauri flavor 設定及 npm 建置入口。
- [x] 修正 Vite 環境標籤並補上設定契約測試。
- [x] 同步 README、Harness、執行環境與 CI 文件。
- [x] 執行自動驗證、桌面建置與啟動檢查並記錄結果。
- [ ] 在具備 macOS 輔助取用權限的桌面環境完成互動式視窗標題與資料隔離檢查。

## 發現與決策紀錄

- 2026-09-15：基準前端驗證成功：19 個單元測試與 2 個端對端測試通過；建置存在既有 `@vueuse/core` PURE 註解警告。
- 2026-09-15：基準 Rust 驗證成功，產品與後端測試均通過。
- 2026-09-15：採用 `neo-fd-desktop-dev` 與 `com.ben.neo-fd-desktop.dev`，不新增測試或預發布 flavor。
- 2026-09-15：採用不同 identifier 以隔離應用程式資料，不進行正式資料匯入。
- 2026-09-15：採用既有品牌圖形加 `DEV` 標記的可重現圖示，不重新生成整個品牌標誌。
- 2026-09-15：Tauri `--no-bundle` 正式與開發建置均成功。
- 2026-09-15：macOS 正式與開發 `.app` 均成功產生；兩者的 bundle name、identifier 與 `icon.icns` 已完成對照。
- 2026-09-15：`tauri:dev` 實際啟動 Vite 與 Rust 桌面程序後可正常停止；系統輔助取用權限不足，未能以腳本讀取視窗標題。
- 2026-09-15：完成工作樹程式碼審查，未發現可操作的重大或一般問題。

## 具體命令與預期結果

圖示產生：

```text
npm --prefix neo-fd-desktop run tauri -- icon src-tauri/icons/dev-icon.svg --output src-tauri/icons/dev
```

預期 `neo-fd-desktop/src-tauri/icons/dev/` 產生有效的 PNG、ICNS、ICO 與其他 Tauri 標準圖示檔案。

建置入口：

```text
npm --prefix neo-fd-desktop run tauri:dev
npm --prefix neo-fd-desktop run tauri:build
npm --prefix neo-fd-desktop run tauri:build:dev
```

預期第一個命令啟動開發身份，第二個命令建立正式身份，第三個命令建立可與正式版並存的開發安裝包。

## 自動與手動驗證

自動驗證：

```text
npm --prefix neo-fd-desktop run lint
npm --prefix neo-fd-desktop run build
npm --prefix neo-fd-desktop run build:dev
npm --prefix tests/frontend run lint
npm --prefix tests/frontend run typecheck
npm --prefix tests/frontend run test:unit
npm --prefix tests/frontend run test:e2e
npm --prefix neo-fd-desktop run tauri:build -- --no-bundle
npm --prefix neo-fd-desktop run tauri:build:dev -- --no-bundle
sh .neo_harness/verify.sh
```

上述命令均已成功；前端建置保留既有 `@vueuse/core` PURE 註解警告。

手動驗證：

- 同時啟動正式安裝版與 `tauri:dev`，確認視窗標題、Dock 或工作列圖示不同。
- 安裝開發安裝包，確認安裝器、應用程式名稱與圖示均帶有開發辨識。
- 在兩個版本分別變更可保存的前端狀態，確認資料不互相影響。
- 確認正式發布命令仍使用正式名稱、identifier 與圖示。

已完成 `.app` metadata 與圖示資源對照，以及 `tauri:dev` 啟動檢查；視窗標題與兩個已安裝版本的互動式資料隔離仍需在具備 macOS 輔助取用權限的桌面環境確認。

## 冪等、失敗復原與回復方式

- 圖示產生只寫入 `icons/dev/`，可使用相同來源重複產生，不觸碰正式資產。
- 若開發圖示產生失敗，不將不完整檔案接入設定；修正來源後重新產生。
- 若開發 flavor 驗證失敗，正式 `tauri build` 與正式發布設定仍可獨立使用。
- 回復時移除開發設定、開發圖示、開發 npm 入口、環境標籤變更及相關文件；正式設定保持原值。

## 完成結果與剩餘工作

核心實作、自動驗證、桌面建置與啟動檢查已完成；計畫仍保留在 `active/`，因目前 macOS 輔助取用權限不足，尚未能直接完成視窗標題與兩個已安裝版本的互動式資料隔離檢查。
