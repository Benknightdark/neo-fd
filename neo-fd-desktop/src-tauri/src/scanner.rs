use anyhow::Result;
use ignore::{WalkBuilder, WalkState};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;

#[derive(serde::Serialize, Clone)]
pub struct ScanResult {
    pub path: String,
    pub line_num: usize,
    pub pattern_name: String,
    pub matched_text: String,
}

pub struct Scanner<F>
where
    F: Fn(ScanResult) + Send + Sync + 'static,
{
    patterns: Vec<(String, Regex)>,
    on_match: Arc<F>,
}

impl<F> Scanner<F>
where
    F: Fn(ScanResult) + Send + Sync + 'static,
{
    pub fn new(patterns: Vec<(String, Regex)>, on_match: F) -> Self {
        Self {
            patterns,
            on_match: Arc::new(on_match),
        }
    }

    pub fn scan_dir(&self, path: &Path) {
        let patterns = Arc::new(self.patterns.clone());
        let on_match = Arc::clone(&self.on_match);

        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .build_parallel();

        walker.run(|| {
            let patterns = Arc::clone(&patterns);
            let on_match = Arc::clone(&on_match);

            Box::new(move |entry| {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return WalkState::Continue,
                };

                let path = entry.path();
                if path.is_file() {
                    let _ = scan_file(path, &patterns, &*on_match);
                }

                WalkState::Continue
            })
        });
    }
}

fn scan_file<F>(path: &Path, patterns: &[(String, Regex)], on_match: &F) -> Result<()>
where
    F: Fn(ScanResult) + Send + Sync + 'static,
{
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Ok(()),
    };
    let mut reader = BufReader::new(file);
    let mut line_buf = String::new();
    let mut line_num = 1;

    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) => break,
            Ok(_) => {
                for (name, re) in patterns {
                    for mat in re.find_iter(&line_buf) {
                        on_match(ScanResult {
                            path: path.to_string_lossy().into_owned(),
                            line_num,
                            pattern_name: name.clone(),
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
