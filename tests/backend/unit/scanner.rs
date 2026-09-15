use anyhow::{anyhow, Result};
use neo_fd_desktop::backend::{compile_patterns, scan_directory, ScanResult};
use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

fn scan_directory_results(
    path: &Path,
    patterns: Vec<(String, String, bool)>,
    max_results: Option<usize>,
) -> Result<Vec<ScanResult>> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let captured_results = Arc::clone(&results);

    scan_directory(
        path,
        patterns,
        move |result| {
            if let Ok(mut results) = captured_results.lock() {
                results.push(result);
            }
        },
        Arc::new(AtomicBool::new(false)),
        max_results,
    )
    .map_err(anyhow::Error::msg)?;

    let results = Arc::try_unwrap(results)
        .map_err(|_| anyhow!("掃描結果仍被其他執行緒持有"))?
        .into_inner()
        .map_err(|_| anyhow!("掃描結果鎖定失敗"))?;

    Ok(results)
}

fn sort_results(results: &mut [ScanResult]) {
    results.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.line_num.cmp(&right.line_num))
            .then(pattern_order(&left.pattern_name).cmp(&pattern_order(&right.pattern_name)))
            .then(left.matched_text.cmp(&right.matched_text))
    });
}

fn pattern_order(name: &str) -> usize {
    match name {
        "身分證字號" => 0,
        "台灣十大姓氏" => 1,
        _ => 2,
    }
}

#[test]
fn 掃描未設定上限時回傳所有匹配() -> Result<()> {
    let directory = TempDir::new()?;
    fs::write(
        directory.path().join("unlimited.txt"),
        "secret\nsecret\nsecret\n",
    )?;

    let results = scan_directory_results(
        directory.path(),
        vec![("測試".to_string(), "secret".to_string(), false)],
        None,
    )?;

    assert_eq!(results.len(), 3);
    Ok(())
}

#[test]
fn 掃描設定上限時停止於指定數量() -> Result<()> {
    let directory = TempDir::new()?;
    fs::write(
        directory.path().join("limited.txt"),
        "secret\nsecret\nsecret\n",
    )?;

    let results = scan_directory_results(
        directory.path(),
        vec![("測試".to_string(), "secret".to_string(), false)],
        Some(2),
    )?;

    assert_eq!(results.len(), 2);
    Ok(())
}

#[test]
fn 掃描邊界規則時只回傳獨立匹配() -> Result<()> {
    let directory = TempDir::new()?;
    fs::write(
        directory.path().join("boundaries.txt"),
        "欄位：A123456789。 姓名「陳小明」\n\
A123456789,B223456789\n\
XA123456789Y 客王小明戶\n\
前 A123456789後 前陳小明後\n\
標記$A123456789$\n\
\u{3000}A123456789\t王小明\u{3000}\n\
A123456789",
    )?;

    let mut results = scan_directory_results(
        directory.path(),
        vec![
            (
                "身分證字號".to_string(),
                r"[A-Za-z][12]\d{8}".to_string(),
                true,
            ),
            (
                "台灣十大姓氏".to_string(),
                r"[陳林黃張李王吳劉蔡楊][\u{4e00}-\u{9fa5}]{2}".to_string(),
                true,
            ),
        ],
        None,
    )?;
    sort_results(&mut results);

    assert_eq!(results.len(), 7);
    assert_eq!(
        results
            .iter()
            .map(|result| result.matched_text.as_str())
            .collect::<Vec<_>>(),
        vec![
            "A123456789",
            "陳小明",
            "A123456789",
            "B223456789",
            "A123456789",
            "王小明",
            "A123456789",
        ]
    );
    Ok(())
}

#[test]
fn 自定義規則保留嵌入式匹配() -> Result<()> {
    let directory = TempDir::new()?;
    fs::write(
        directory.path().join("custom-pattern.txt"),
        "prefixsecretvalue",
    )?;

    let results = scan_directory_results(
        directory.path(),
        vec![("測試".to_string(), "secret".to_string(), false)],
        None,
    )?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].matched_text, "secret");
    Ok(())
}

#[test]
fn 遞迴掃描時略過二進位檔案() -> Result<()> {
    let directory = TempDir::new()?;
    let nested = directory.path().join("nested");
    fs::create_dir(&nested)?;
    fs::write(nested.join("plain.txt"), "secret")?;
    fs::write(
        nested.join("binary.bin"),
        [0, b's', b'e', b'c', b'r', b'e', b't'],
    )?;

    let results = scan_directory_results(
        directory.path(),
        vec![("測試".to_string(), "secret".to_string(), false)],
        None,
    )?;

    assert_eq!(results.len(), 1);
    assert!(results[0].path.ends_with("plain.txt"));
    Ok(())
}

#[test]
fn 初始中止狀態不會讀取檔案() -> Result<()> {
    let directory = TempDir::new()?;
    fs::write(directory.path().join("aborted.txt"), "secret")?;
    let results = Arc::new(Mutex::new(Vec::new()));
    let captured_results = Arc::clone(&results);
    let aborted = Arc::new(AtomicBool::new(true));

    scan_directory(
        directory.path(),
        vec![("測試".to_string(), "secret".to_string(), false)],
        move |result| {
            if let Ok(mut results) = captured_results.lock() {
                results.push(result);
            }
        },
        aborted,
        None,
    )
    .map_err(anyhow::Error::msg)?;

    assert!(results
        .lock()
        .map_err(|_| anyhow!("掃描結果鎖定失敗"))?
        .is_empty());
    Ok(())
}

#[test]
fn 無效正則表達式會回傳錯誤() {
    let error = compile_patterns(vec![("錯誤".to_string(), "[".to_string(), false)])
        .expect_err("無效正則表達式應該失敗");

    assert!(!error.is_empty());
}

#[test]
fn 編譯正則表達式時保留規則名稱與邊界設定() -> Result<()> {
    let patterns = compile_patterns(vec![("測試".to_string(), regex::escape("secret"), true)])
        .map_err(anyhow::Error::msg)?;

    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].0.as_ref(), "測試");
    assert!(patterns[0].2);
    Ok(())
}
