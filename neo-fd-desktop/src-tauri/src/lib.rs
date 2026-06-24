use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
mod scanner;
use scanner::Scanner;
use tauri::Emitter;

const RESULT_CHANNEL_BUFFER: usize = 2_000;
const RESULT_BATCH_SIZE: usize = 500;
const RESULT_BATCH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

#[derive(Default)]
struct AppScanState {
    aborted: Arc<AtomicBool>,
}

#[tauri::command]
async fn scan_directory(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppScanState>,
    path: String,
    patterns: Vec<(String, String)>,
    max_results: Option<usize>,
) -> Result<(), String> {
    let mut regex_patterns = Vec::new();
    for (name, p) in patterns {
        let re = Regex::new(&p).map_err(|e| e.to_string())?;
        let name_arc: Arc<str> = Arc::from(name.as_str());
        regex_patterns.push((name_arc, re));
    }

    // 重設中止狀態
    state.aborted.store(false, Ordering::Relaxed);

    let aborted_scanner = Arc::clone(&state.aborted);
    let (tx, rx) = std::sync::mpsc::sync_channel::<scanner::ScanResult>(RESULT_CHANNEL_BUFFER);

    let scanner = Scanner::new(
        regex_patterns,
        move |res| {
            let _ = tx.send(res);
        },
        aborted_scanner,
        max_results,
    );

    let scan_path = std::path::PathBuf::from(path);
    let app_clone = app.clone();

    // 啟動背景工作執行緒群組
    std::thread::spawn(move || {
        // 啟動掃描執行緒
        let scan_handle = std::thread::spawn(move || {
            scanner.scan_dir(&scan_path);
        });

        // 收集執行緒：從 rx 中讀取結果，並進行固定間隔與數量的批次發送
        let mut batch = Vec::new();
        let mut last_emit = std::time::Instant::now();

        loop {
            let timeout = RESULT_BATCH_INTERVAL
                .checked_sub(last_emit.elapsed())
                .unwrap_or(std::time::Duration::ZERO);

            match rx.recv_timeout(timeout) {
                Ok(res) => {
                    batch.push(res);
                    if batch.len() >= RESULT_BATCH_SIZE
                        || last_emit.elapsed() >= RESULT_BATCH_INTERVAL
                    {
                        let _ = app_clone.emit("scan-result-batch", &batch);
                        batch.clear();
                        last_emit = std::time::Instant::now();
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if !batch.is_empty() {
                        let _ = app_clone.emit("scan-result-batch", &batch);
                        batch.clear();
                    }
                    last_emit = std::time::Instant::now();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        // 發送剩餘的最後一批結果
        if !batch.is_empty() {
            let _ = app_clone.emit("scan-result-batch", &batch);
        }

        // 等待掃描執行緒結束 (通常此時已結束)
        let _ = scan_handle.join();

        // 觸發掃描結束事件
        let _ = app_clone.emit("scan-finished", ());
    });

    Ok(())
}

#[tauri::command]
async fn cancel_scan(state: tauri::State<'_, AppScanState>) -> Result<(), String> {
    state.aborted.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn read_file_content(path: String) -> Result<String, String> {
    let file_path = std::path::Path::new(&path);
    if !file_path.is_file() {
        return Err("所選路徑不是有效的檔案。".to_string());
    }

    // 防禦性檢查：限制最大讀取檔案大小為 5MB 避免記憶體崩潰與效能瓶頸
    let metadata = std::fs::metadata(file_path).map_err(|e| e.to_string())?;
    if metadata.len() > 5 * 1024 * 1024 {
        return Err("檔案過大（超過 5MB），基於效能安全考量不予載入。".to_string());
    }

    // 讀取檔案內容，優雅處理 I/O 錯誤與非 UTF-8 格式
    let content =
        std::fs::read_to_string(file_path).map_err(|e| format!("無法讀取檔案內容: {}", e))?;

    Ok(content)
}

#[tauri::command]
async fn write_file_content(path: String, content: String) -> Result<(), String> {
    let file_path = std::path::Path::new(&path);
    if !file_path.exists() {
        return Err("所選路徑的檔案不存在。".to_string());
    }
    if !file_path.is_file() {
        return Err("所選路徑不是有效的檔案。".to_string());
    }

    std::fs::write(file_path, content).map_err(|e| format!("無法寫入檔案內容: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn delete_file(path: String) -> Result<(), String> {
    let file_path = std::path::Path::new(&path);
    if !file_path.exists() {
        return Err("檔案不存在或已被刪除。".to_string());
    }
    if !file_path.is_file() {
        return Err("所選路徑不是有效的檔案。".to_string());
    }

    std::fs::remove_file(file_path).map_err(|e| format!("無法刪除檔案: {}", e))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppScanState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            cancel_scan,
            read_file_content,
            write_file_content,
            delete_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
