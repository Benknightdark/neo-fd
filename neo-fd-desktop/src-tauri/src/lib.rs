use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
pub mod backend;
mod file_ops;
pub mod scanner;
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
    patterns: Vec<(String, String, bool)>,
    max_results: Option<usize>,
) -> Result<(), String> {
    let compiled_patterns = backend::compile_patterns(patterns)?;

    // 重設中止狀態
    state.aborted.store(false, Ordering::Relaxed);

    let aborted_scanner = Arc::clone(&state.aborted);
    let (tx, rx) = std::sync::mpsc::sync_channel::<backend::ScanResult>(RESULT_CHANNEL_BUFFER);

    let scan_path = std::path::PathBuf::from(path);
    let app_clone = app.clone();

    // 啟動背景工作執行緒群組
    std::thread::spawn(move || {
        // 啟動掃描執行緒
        let scan_handle = std::thread::spawn(move || {
            let scanner = backend::Scanner::new(
                compiled_patterns,
                move |result| {
                    let _ = tx.send(result);
                },
                aborted_scanner,
                max_results,
            );
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
    backend::read_file_content(std::path::Path::new(&path))
}

#[tauri::command]
async fn write_file_content(path: String, content: String) -> Result<(), String> {
    backend::write_file_content(std::path::Path::new(&path), &content)
}

#[tauri::command]
async fn delete_file(path: String) -> Result<(), String> {
    backend::delete_file(std::path::Path::new(&path))
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
