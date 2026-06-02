# Neo FD 專案開發指引

本文件供 Neo FD 的開發者與程式助理使用，包含 CLI、IDE 外掛、本機工具與雲端代理。內容不綁定特定工具；以專案檔案與使用者最新需求為準。

## 1. 協作原則

- 所有的回答和commit msg都必須是正體中文
- 先理解需求與目前狀態，再修改檔案。
- 使用者是產品決策者；遇到影響範圍大的決策、需求衝突或技術限制時，先說明選項與取捨。
- 小步修改、清楚驗證，避免一次混入多個不相關變更。
- 用白話說明技術判斷，不用工具或框架名詞掩蓋風險。
- 不覆蓋未確認的既有修改；若工作樹已有變更，先分辨是否與本次任務相關。

## 2. 開始工作前

建議先完成以下檢查：

```bash
git status --short --branch
```

依任務閱讀相關文件：

- `README.md`
- `AGENTS.md`
- `neo-fd-desktop/package.json`
- `neo-fd-desktop/src-tauri/Cargo.toml`

如果任務會改動架構、掃描規則、CI/CD 或發布流程，先提出簡短計畫並取得確認。不綁定特定指令、CLI 或計畫檔格式。

## 3. 專案結構

```text
neo-fd/
├── AGENTS.md
├── README.md
└── neo-fd-desktop/
    ├── package.json
    ├── src/                 # Vue 3 前端
    └── src-tauri/           # Tauri 2 / Rust 後端
        ├── Cargo.toml
        └── src/
            ├── lib.rs       # Tauri 指令與事件橋接
            ├── main.rs
            └── scanner.rs   # 核心掃描邏輯
```

所有 npm 指令都在 `neo-fd-desktop/` 內執行。

## 4. 核心架構規範

- 前端 UI 放在 `neo-fd-desktop/src/`，處理互動、狀態與顯示。
- Tauri IPC 層放在 `neo-fd-desktop/src-tauri/src/lib.rs`，把前端請求轉成後端呼叫。
- 核心掃描邏輯放在 `neo-fd-desktop/src-tauri/src/scanner.rs`，保持獨立，不引入 UI、視窗或 Tauri API。
- 掃描器逐行讀取檔案時，不要在迴圈內建立新的 `String`；使用迴圈外的 `line_buf`，每次讀取前呼叫 `clear()`。
- 使用者輸入的 Regex 必須安全處理 `Result`，不得用 `unwrap()` 或 `expect()` 讓應用程式閃退。
- 目錄走訪與檔案讀取要處理權限不足、損壞檔案、非 UTF-8 與二進位內容；單一檔案失敗不得中斷整體掃描。

## 5. 常用指令

安裝相依套件：

```bash
cd neo-fd-desktop
npm install
```

啟動桌面開發環境：

```bash
cd neo-fd-desktop
npm run tauri dev
```

執行品質檢查：

```bash
cd neo-fd-desktop
npm run lint:all
```

執行測試：

```bash
cd neo-fd-desktop
npm run test:all
```

編譯桌面安裝檔：

```bash
cd neo-fd-desktop
npm run tauri build
```


## 6. 完成前檢查

- 確認修改範圍符合使用者需求。
- 文件或程式碼要符合實際專案結構。
- 能執行的檢查盡量執行；未執行時說明原因。
- 回報修改重點、驗證結果與剩餘風險。
