# 系統架構

## 系統概觀

`neo-fd-desktop` 是桌面檔案掃描應用程式。前端在 `neo-fd-desktop/src/` 負責設定輸入、掃描狀態、結果呈現與檔案內容操作；原生層在 `neo-fd-desktop/src-tauri/src/` 提供命令、背景工作與檔案掃描。兩側以命令呼叫及掃描事件交換資料。

目前的主要入口是 `neo-fd-desktop/index.html` 載入 `neo-fd-desktop/src/main.ts`，`neo-fd-desktop/src/main.ts` 建立應用程式與全域狀態後掛載 `neo-fd-desktop/src/App.vue`。原生執行入口是 `neo-fd-desktop/src-tauri/src/main.rs`，委派至 `neo-fd-desktop/src-tauri/src/lib.rs` 的 `run`。

## 主要元件

| 元件 | 責任 | 允許依賴 | 禁止依賴 | 證據 |
| :--- | :--- | :--- | :--- | :--- |
| `neo-fd-desktop/src/main.ts` | 建立前端應用程式、註冊全域狀態並掛載根元件 | `neo-fd-desktop/src/App.vue`、全域樣式、狀態註冊 | 掃描實作與檔案直接操作 | `neo-fd-desktop/index.html`、`neo-fd-desktop/src/main.ts` |
| `neo-fd-desktop/src/App.vue` | 組合主要畫面，管理掃描事件監聽器的初始化與清理 | 主要畫面元件、掃描 Store | 原生檔案操作實作 | `neo-fd-desktop/src/App.vue` |
| `neo-fd-desktop/src/stores/scan.ts` | 保存掃描設定與結果，接收批次事件，處理取消及結果索引 | `api/ipc.ts`、通知 Store、事件監聽 | 直接讀寫檔案、掃描演算法 | `neo-fd-desktop/src/stores/scan.ts` |
| `neo-fd-desktop/src/api/ipc.ts` | 定義命令契約、封裝命令呼叫與錯誤通知 | 命令呼叫介面、系統開啟介面、通知 Store | 目錄走訪與正則比對 | `neo-fd-desktop/src/api/ipc.ts` |
| `neo-fd-desktop/src/components/*.vue` | 提供路徑與規則設定、結果列表或樹狀檢視、內容抽屜及通知畫面 | Store、IPC 封裝、結果樹工具 | 原生命令的實作細節 | `neo-fd-desktop/src/components/ScanSidebar.vue`、`neo-fd-desktop/src/components/ScanResultsTable.vue`、`neo-fd-desktop/src/components/CodeViewerDrawer.vue`、`neo-fd-desktop/src/components/ToastNotifications.vue` |
| `neo-fd-desktop/src/utils/resultTree.ts` | 將掃描結果轉為可折疊的目錄、檔案及匹配節點 | `ScanResultItem` 型別 | UI、IPC、檔案系統 | `neo-fd-desktop/src/utils/resultTree.ts` |
| `neo-fd-desktop/src-tauri/src/lib.rs` | 將 IPC 參數轉換為後端服務呼叫，建立掃描背景工作並轉送批次事件 | `backend.rs`、執行緒與事件介面 | 前端元件與瀏覽器 DOM、掃描業務實作 | `neo-fd-desktop/src-tauri/src/lib.rs` |
| `neo-fd-desktop/src-tauri/src/backend.rs` | 提供不依賴 Tauri 執行期的掃描、正則編譯與檔案服務公開邊界 | `scanner.rs`、`file_ops.rs`、標準路徑與同步原語 | 前端元件、Tauri 視窗與 IPC 事件 | `neo-fd-desktop/src-tauri/src/backend.rs` |
| `neo-fd-desktop/src-tauri/src/file_ops.rs` | 實作檔案讀取、寫入、刪除及大小與路徑檢查 | 標準檔案介面 | 前端元件、Tauri 執行期 | `neo-fd-desktop/src-tauri/src/file_ops.rs` |
| `neo-fd-desktop/src-tauri/src/scanner.rs` | 平行走訪目錄、逐行比對檔案、依規則旗標檢查左右邊界、處理取消與結果上限 | 標準檔案介面、目錄走訪、正則比對 | 使用者介面、視窗與事件 API | `neo-fd-desktop/src-tauri/src/scanner.rs` |
| `neo-fd-desktop/src-tauri/src/main.rs` | 原生二進位檔入口 | `lib.rs::run` | 掃描與檔案業務邏輯 | `neo-fd-desktop/src-tauri/src/main.rs` |
| `tests/frontend/` | 以獨立 npm 專案執行前端單元測試與瀏覽器端對端測試 | 前端產品原始碼、Vue 測試工具、Playwright | Rust 實作與實際桌面執行期 | `tests/frontend/package.json`、`tests/frontend/playwright.config.ts` |
| `tests/backend/` | 以獨立 Cargo 專案執行後端單元測試與實際檔案系統端對端測試 | `backend` 公開服務、暫存檔案系統 | Vue、Tauri 視窗與 IPC 執行期 | `tests/backend/Cargo.toml`、`tests/backend/e2e/filesystem.rs` |

## 資料與控制流程

1. 使用者在 `neo-fd-desktop/src/components/ScanSidebar.vue` 輸入路徑、啟用內建規則或輸入自定義規則，並可設定正整數結果上限。
2. `neo-fd-desktop/src/stores/scan.ts` 組合啟用規則與邊界旗標並呼叫 `scannerApi.startScan`；`neo-fd-desktop/src/api/ipc.ts` 將呼叫轉為 `scan_directory` 命令。
3. `neo-fd-desktop/src-tauri/src/lib.rs` 透過 `neo-fd-desktop/src-tauri/src/backend.rs` 編譯正則規則，建立有界結果通道與背景執行緒，讓 `neo-fd-desktop/src-tauri/src/scanner.rs` 走訪目錄並產生匹配結果。
4. 原生層以 `scan-result-batch` 事件傳送結果，以 `scan-finished` 事件表示工作結束；Store 以前端批次方式更新結果與檔案路徑索引。
5. `neo-fd-desktop/src/components/ScanResultsTable.vue` 以列表或目錄樹呈現結果。使用者點擊匹配項目後，`neo-fd-desktop/src/App.vue` 開啟 `neo-fd-desktop/src/components/CodeViewerDrawer.vue`。
6. 內容抽屜透過 `read_file_content` 載入檔案，可透過 `write_file_content` 儲存修改、透過 `delete_file` 刪除檔案，完成刪除後通知 Store 移除該檔案的結果。
7. `tests/frontend/` 在瀏覽器初始化前建立 Tauri API 與事件的模擬，使用產品前端組合驗證掃描及檔案操作流程；`tests/backend/` 直接呼叫公開後端服務並使用暫存目錄驗證實際檔案行為。

## 執行邊界

- 前端只透過 `neo-fd-desktop/src/api/ipc.ts` 呼叫原生命令；命令契約包含 `scan_directory`、`cancel_scan`、`read_file_content`、`write_file_content` 與 `delete_file`。
- `neo-fd-desktop/src-tauri/src/lib.rs` 是前端命令與原生工作之間的邊界，掃描結果以事件而非同步命令返回值傳送。
- `neo-fd-desktop/src-tauri/src/backend.rs` 是 Tauri 命令與後端業務之間的公開服務邊界；它不依賴 Tauri 執行期，因此可由 `tests/backend/` 直接驗證。
- `tests/frontend/` 的端對端測試驗證瀏覽器中的產品組合與 IPC 參數，不宣稱涵蓋實際 Tauri 桌面執行期。
- `neo-fd-desktop/src-tauri/src/scanner.rs` 不匯入前端或視窗執行期；單一檔案開啟、讀取或目錄走訪錯誤目前會被略過，不中斷整體掃描。
- 掃描器將目錄走訪設為不略過隱藏項目但遵循忽略規則；二進位檔案會略過，單行讀取上限為 65,536 位元組。
- 需要邊界的規則只有在匹配內容左右兩側均為 Unicode 空白或 Unicode 標點時才回報；行首與行尾視為有效邊界，自定義規則不啟用此檢查。
- `read_file_content` 只接受檔案，且檔案大小不得超過 5 MB；寫入與刪除命令只檢查目標存在且為檔案。
- 空白掃描路徑會由 Store 轉為 `/`；這是目前實作行為，不是安全範圍限制。

## 不變量與機械驗證

| 不變量 | 狀態 | 驗證方式 |
| :--- | :--- | :--- |
| IPC 命令名稱、掃描規則 tuple 與結果資料欄位保持一致 | 文件規則；由前端單元與瀏覽器端對端測試觀察 | `neo-fd-desktop/src/api/ipc.ts`、`neo-fd-desktop/src-tauri/src/lib.rs`、`tests/frontend/unit/scan.spec.ts`、`tests/frontend/e2e/scan-flow.spec.ts` |
| 最大匹配筆數必須是空值或大於 0 的整數 | 可執行規則 | `npm --prefix tests/frontend run test:unit` |
| 批次結果建立穩定 ID 並維護檔案路徑索引 | 可執行規則 | `npm --prefix tests/frontend run test:unit` |
| 目錄樹依折疊狀態輸出節點 | 可執行規則 | `npm --prefix tests/frontend run test:unit` |
| Rust 格式、靜態檢查與測試不得失敗 | 可執行規則 | `sh .neo_harness/verify.sh rust` |
| 檔案掃描、讀取、寫入與刪除服務維持限制與結果 | 可執行規則 | `cargo test --manifest-path tests/backend/Cargo.toml` |
| 前端掃描流程傳送 IPC 並呈現結果 | 可執行規則 | `npm --prefix tests/frontend run test:e2e` |
| `scanner.rs` 不直接依賴前端或視窗層 | 文件規則 | 檢視 `neo-fd-desktop/src-tauri/src/scanner.rs` 的匯入與模組邊界 |

## 未確認事項

- 尚無專門的架構依賴檢查；模組邊界目前主要以文件與程式碼審查維護。
- 尚無涵蓋實際 Tauri 桌面命令、跨執行緒取消及實際檔案權限的整合測試；瀏覽器端對端測試使用 IPC 模擬，後端端對端測試使用公開服務。
- 尚無結構化日誌、指標或追蹤查詢入口；目前可見的執行期證據主要是畫面通知與錯誤輸出。
- 不同作業系統的桌面建置前置條件及簽署設定未由本專案完整定義。
