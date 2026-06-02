use regex::Regex;
mod scanner;
use scanner::Scanner;
use tauri::Emitter;

#[tauri::command]
async fn scan_directory(
    app: tauri::AppHandle,
    path: String,
    patterns: Vec<(String, String)>,
) -> Result<(), String> {
    let mut regex_patterns = Vec::new();
    for (name, p) in patterns {
        let re = Regex::new(&p).map_err(|e| e.to_string())?;
        regex_patterns.push((name, re));
    }

    let app_clone = app.clone();
    let scanner = Scanner::new(regex_patterns, move |res| {
        let _ = app_clone.emit("scan-result", res);
    });

    let scan_path = std::path::PathBuf::from(path);
    std::thread::spawn(move || {
        scanner.scan_dir(&scan_path);
        let _ = app.emit("scan-finished", ());
    });

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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            read_file_content,
            write_file_content,
            delete_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
