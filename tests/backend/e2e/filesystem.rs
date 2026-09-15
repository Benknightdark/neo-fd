use anyhow::{anyhow, Result};
use neo_fd_desktop::backend::{delete_file, read_file_content, scan_directory, write_file_content};
use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

fn scan_results(
    path: &Path,
    results: Arc<Mutex<Vec<neo_fd_desktop::backend::ScanResult>>>,
) -> Result<()> {
    let captured_results = Arc::clone(&results);
    scan_directory(
        path,
        vec![("自定義".to_string(), "secret".to_string(), false)],
        move |result| {
            if let Ok(mut results) = captured_results.lock() {
                results.push(result);
            }
        },
        Arc::new(AtomicBool::new(false)),
        None,
    )
    .map_err(anyhow::Error::msg)
}

#[test]
fn 實際檔案流程可掃描讀取修改並刪除() -> Result<()> {
    let directory = TempDir::new()?;
    let nested = directory.path().join("documents");
    fs::create_dir(&nested)?;
    let path = nested.join("report.txt");
    fs::write(&path, "第一行\nsecret\n第三行")?;

    let results = Arc::new(Mutex::new(Vec::new()));
    scan_results(directory.path(), Arc::clone(&results))?;
    let results = results.lock().map_err(|_| anyhow!("掃描結果鎖定失敗"))?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].line_num, 2);
    assert_eq!(results[0].matched_text, "secret");
    drop(results);

    assert_eq!(
        read_file_content(&path).map_err(anyhow::Error::msg)?,
        "第一行\nsecret\n第三行"
    );
    write_file_content(&path, "第一行\n已修正\n第三行").map_err(anyhow::Error::msg)?;
    assert_eq!(
        read_file_content(&path).map_err(anyhow::Error::msg)?,
        "第一行\n已修正\n第三行"
    );

    delete_file(&path).map_err(anyhow::Error::msg)?;
    assert!(!path.exists());
    assert!(delete_file(&path).is_err());
    Ok(())
}

#[test]
fn 檔案服務維持大小與路徑限制() -> Result<()> {
    let directory = TempDir::new()?;
    let oversized = directory.path().join("oversized.txt");
    fs::write(&oversized, vec![b'a'; 5 * 1024 * 1024 + 1])?;
    assert_eq!(
        read_file_content(&oversized),
        Err("檔案過大（超過 5MB），基於效能安全考量不予載入。".to_string())
    );

    assert_eq!(
        read_file_content(directory.path()),
        Err("所選路徑不是有效的檔案。".to_string())
    );
    assert_eq!(
        write_file_content(&directory.path().join("missing.txt"), "內容"),
        Err("所選路徑的檔案不存在。".to_string())
    );
    assert_eq!(
        delete_file(directory.path()),
        Err("所選路徑不是有效的檔案。".to_string())
    );
    Ok(())
}
