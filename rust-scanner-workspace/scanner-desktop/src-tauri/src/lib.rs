use regex::Regex;
use scanner_core::Scanner;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![scan_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
