use anyhow::Result;
use ignore::{WalkBuilder, WalkState};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

// 限制單行最大讀取為 64KB (65,536 位元組) 以防無換行符的大型檔案導致 OOM
const MAX_LINE_BYTES: usize = 65536;

#[derive(serde::Serialize, Clone)]
pub struct ScanResult {
    pub path: Arc<str>,
    pub line_num: usize,
    pub pattern_name: Arc<str>,
    pub matched_text: String,
}

pub struct Scanner<F>
where
    F: Fn(ScanResult) + Send + Sync + 'static,
{
    patterns: Vec<(Arc<str>, Regex)>,
    on_match: Arc<F>,
    aborted: Arc<AtomicBool>,
    match_count: Arc<AtomicUsize>,
}

impl<F> Scanner<F>
where
    F: Fn(ScanResult) + Send + Sync + 'static,
{
    pub fn new(patterns: Vec<(Arc<str>, Regex)>, on_match: F, aborted: Arc<AtomicBool>) -> Self {
        Self {
            patterns,
            on_match: Arc::new(on_match),
            aborted,
            match_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn scan_dir(&self, path: &Path) {
        let patterns = Arc::new(self.patterns.clone());
        let on_match = Arc::clone(&self.on_match);
        let aborted = Arc::clone(&self.aborted);
        let match_count = Arc::clone(&self.match_count);

        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .build_parallel();

        walker.run(|| {
            let patterns = Arc::clone(&patterns);
            let on_match = Arc::clone(&on_match);
            let aborted = Arc::clone(&aborted);
            let match_count = Arc::clone(&match_count);

            Box::new(move |entry| {
                // 如果已觸發中止，立即結束走訪執行緒
                if aborted.load(Ordering::Relaxed) {
                    return WalkState::Quit;
                }

                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return WalkState::Continue,
                };

                let path = entry.path();
                if path.is_file() {
                    let _ = scan_file(path, &patterns, &*on_match, &aborted, &match_count);
                }

                WalkState::Continue
            })
        });
    }
}

/// 偵測檔案是否為二進位檔案 (讀取前 1024 位元組檢查是否包含空字元 '\0')
fn is_binary_file(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buffer = [0u8; 1024];
    match file.read(&mut buffer) {
        Ok(n) => buffer[..n].contains(&0),
        Err(_) => false,
    }
}

fn scan_file<F>(
    path: &Path,
    patterns: &[(Arc<str>, Regex)],
    on_match: &F,
    aborted: &AtomicBool,
    match_count: &AtomicUsize,
) -> Result<()>
where
    F: Fn(ScanResult) + Send + Sync + 'static,
{
    // 1. 如果已中止，直接跳過
    if aborted.load(Ordering::Relaxed) {
        return Ok(());
    }

    // 2. 二進位偵測安全防護
    if is_binary_file(path) {
        return Ok(());
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Ok(()),
    };

    let mut reader = BufReader::new(file);
    let mut buf = Vec::with_capacity(1024);
    let mut line_num = 1;

    // 共享同一個 path 的 Arc<str> 避免在同一檔案有多筆匹配時重複複製
    let path_str = path.to_string_lossy();
    let path_arc: Arc<str> = Arc::from(path_str.as_ref());

    loop {
        // 定期檢查中止狀態
        if aborted.load(Ordering::Relaxed) {
            break;
        }

        buf.clear();
        // 限制單次最多讀取 MAX_LINE_BYTES 以防記憶體暴增 (OOM)
        match reader
            .by_ref()
            .take(MAX_LINE_BYTES as u64)
            .read_until(b'\n', &mut buf)
        {
            Ok(0) => break,
            Ok(_) => {
                // 使用 lossy 方式處理非 UTF-8 內容，避免因編碼錯誤中斷掃描
                let line_str = String::from_utf8_lossy(&buf);

                for (name, re) in patterns {
                    if aborted.load(Ordering::Relaxed) {
                        break;
                    }
                    for mat in re.find_iter(&line_str) {
                        // 增加匹配計數，若達到 10,000 筆限制則自動觸發中止
                        let count = match_count.fetch_add(1, Ordering::Relaxed);
                        if count >= 10000 {
                            aborted.store(true, Ordering::Relaxed);
                            break;
                        }

                        on_match(ScanResult {
                            path: Arc::clone(&path_arc),
                            line_num,
                            pattern_name: Arc::clone(name),
                            matched_text: mat.as_str().to_string(),
                        });
                    }
                }
            }
            Err(_) => break,
        }
        line_num += 1;
    }

    Ok(())
}
