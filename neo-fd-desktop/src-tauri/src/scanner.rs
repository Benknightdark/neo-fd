use anyhow::Result;
use ignore::{WalkBuilder, WalkState};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

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
    patterns: Vec<(Arc<str>, Regex, bool)>,
    on_match: Arc<F>,
    aborted: Arc<AtomicBool>,
    match_count: Arc<AtomicUsize>,
    max_results: Option<usize>,
}

impl<F> Scanner<F>
where
    F: Fn(ScanResult) + Send + Sync + 'static,
{
    pub fn new(
        patterns: Vec<(Arc<str>, Regex, bool)>,
        on_match: F,
        aborted: Arc<AtomicBool>,
        max_results: Option<usize>,
    ) -> Self {
        Self {
            patterns,
            on_match: Arc::new(on_match),
            aborted,
            match_count: Arc::new(AtomicUsize::new(0)),
            max_results,
        }
    }

    pub fn scan_dir(&self, path: &Path) {
        let patterns = Arc::new(self.patterns.clone());
        let on_match = Arc::clone(&self.on_match);
        let aborted = Arc::clone(&self.aborted);
        let match_count = Arc::clone(&self.match_count);
        let max_results = self.max_results;

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
                    let _ = scan_file(
                        path,
                        &patterns,
                        &*on_match,
                        &aborted,
                        &match_count,
                        max_results,
                    );
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

/// 判斷字元是否為可接受的左右邊界：Unicode 空白或 Unicode 標點
fn is_boundary_char(character: char) -> bool {
    static BOUNDARY_CHAR_RE: OnceLock<Regex> = OnceLock::new();
    let mut encoded = [0u8; 4];

    BOUNDARY_CHAR_RE
        .get_or_init(|| Regex::new(r"[\s\p{P}]").expect("固定 Unicode 邊界 Regex 必須有效"))
        .is_match(character.encode_utf8(&mut encoded))
}

/// 驗證匹配內容左右兩側均為有效邊界；行首與行尾視為有效邊界
fn has_valid_boundaries(line: &str, start: usize, end: usize) -> bool {
    let before_is_valid = line[..start]
        .chars()
        .next_back()
        .map(is_boundary_char)
        .unwrap_or(true);
    let after_is_valid = line[end..]
        .chars()
        .next()
        .map(is_boundary_char)
        .unwrap_or(true);

    before_is_valid && after_is_valid
}

fn scan_file<F>(
    path: &Path,
    patterns: &[(Arc<str>, Regex, bool)],
    on_match: &F,
    aborted: &AtomicBool,
    match_count: &AtomicUsize,
    max_results: Option<usize>,
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

                for (name, re, requires_boundary) in patterns {
                    if aborted.load(Ordering::Relaxed) {
                        break;
                    }
                    for mat in re.find_iter(&line_str) {
                        if *requires_boundary
                            && !has_valid_boundaries(&line_str, mat.start(), mat.end())
                        {
                            continue;
                        }

                        if let Some(max_results) = max_results {
                            // 若使用者設定最大匹配筆數，達到限制後自動觸發中止
                            let count = match_count.fetch_add(1, Ordering::Relaxed);
                            if count >= max_results {
                                aborted.store(true, Ordering::Relaxed);
                                break;
                            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(name: &str) -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        std::env::temp_dir().join(format!(
            "neo_fd_scanner_{name}_{}_{}",
            std::process::id(),
            timestamp
        ))
    }

    fn scan_temp_file(path: &Path, max_results: Option<usize>) -> Result<Vec<ScanResult>> {
        let patterns = vec![(Arc::<str>::from("測試"), Regex::new("secret")?, false)];
        scan_temp_file_with_patterns(path, &patterns, max_results)
    }

    fn scan_temp_file_with_patterns(
        path: &Path,
        patterns: &[(Arc<str>, Regex, bool)],
        max_results: Option<usize>,
    ) -> Result<Vec<ScanResult>> {
        let aborted = AtomicBool::new(false);
        let match_count = AtomicUsize::new(0);
        let results = Arc::new(Mutex::new(Vec::new()));
        let captured_results = Arc::clone(&results);

        scan_file(
            path,
            patterns,
            &move |result| {
                if let Ok(mut results) = captured_results.lock() {
                    results.push(result);
                }
            },
            &aborted,
            &match_count,
            max_results,
        )?;

        let results = results
            .lock()
            .map(|results| results.clone())
            .unwrap_or_default();

        Ok(results)
    }

    #[test]
    fn scan_file_without_max_results_returns_all_matches() -> Result<()> {
        let path = temp_file_path("unlimited.txt");
        fs::write(&path, "secret\nsecret\nsecret\n")?;

        let results = scan_temp_file(&path, None)?;
        let _ = fs::remove_file(&path);

        assert_eq!(results.len(), 3);

        Ok(())
    }

    #[test]
    fn scan_file_with_max_results_stops_after_limit() -> Result<()> {
        let path = temp_file_path("limited.txt");
        fs::write(&path, "secret\nsecret\nsecret\n")?;

        let results = scan_temp_file(&path, Some(2))?;
        let _ = fs::remove_file(&path);

        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[test]
    fn scan_file_with_boundary_requirement_returns_only_standalone_matches() -> Result<()> {
        let path = temp_file_path("boundaries.txt");
        fs::write(
            &path,
            "欄位：A123456789。 姓名「陳小明」\n\
A123456789,B223456789\n\
XA123456789Y 客王小明戶\n\
前 A123456789後 前陳小明後\n\
標記$A123456789$\n\
\u{3000}A123456789\t王小明\u{3000}\n\
A123456789",
        )?;

        let patterns = vec![
            (
                Arc::<str>::from("身分證字號"),
                Regex::new(r"[A-Za-z][12]\d{8}")?,
                true,
            ),
            (
                Arc::<str>::from("台灣十大姓氏"),
                Regex::new(r"[陳林黃張李王吳劉蔡楊][\u{4e00}-\u{9fa5}]{2}")?,
                true,
            ),
        ];

        let results = scan_temp_file_with_patterns(&path, &patterns, None)?;
        let _ = fs::remove_file(&path);

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
        assert_eq!(
            results
                .iter()
                .map(|result| result.pattern_name.as_ref())
                .collect::<Vec<_>>(),
            vec![
                "身分證字號",
                "台灣十大姓氏",
                "身分證字號",
                "身分證字號",
                "身分證字號",
                "台灣十大姓氏",
                "身分證字號",
            ]
        );

        Ok(())
    }

    #[test]
    fn scan_file_without_boundary_requirement_preserves_embedded_matches() -> Result<()> {
        let path = temp_file_path("custom-pattern.txt");
        fs::write(&path, "prefixsecretvalue")?;

        let results = scan_temp_file(&path, None)?;
        let _ = fs::remove_file(&path);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matched_text, "secret");

        Ok(())
    }
}
