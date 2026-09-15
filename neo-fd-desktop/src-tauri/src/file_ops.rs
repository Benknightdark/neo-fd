use std::path::Path;

const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

pub fn read_file_content(path: &Path) -> Result<String, String> {
    if !path.is_file() {
        return Err("所選路徑不是有效的檔案。".to_string());
    }

    let metadata = std::fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err("檔案過大（超過 5MB），基於效能安全考量不予載入。".to_string());
    }

    std::fs::read_to_string(path).map_err(|error| format!("無法讀取檔案內容: {error}"))
}

pub fn write_file_content(path: &Path, content: &str) -> Result<(), String> {
    if !path.exists() {
        return Err("所選路徑的檔案不存在。".to_string());
    }
    if !path.is_file() {
        return Err("所選路徑不是有效的檔案。".to_string());
    }

    std::fs::write(path, content).map_err(|error| format!("無法寫入檔案內容: {error}"))
}

pub fn delete_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("檔案不存在或已被刪除。".to_string());
    }
    if !path.is_file() {
        return Err("所選路徑不是有效的檔案。".to_string());
    }

    std::fs::remove_file(path).map_err(|error| format!("無法刪除檔案: {error}"))
}
