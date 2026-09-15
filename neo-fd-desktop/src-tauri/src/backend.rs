use regex::Regex;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub use crate::file_ops::{delete_file, read_file_content, write_file_content};
pub use crate::scanner::{ScanResult, Scanner};

pub type ScanPattern = (String, String, bool);
pub type CompiledScanPattern = (Arc<str>, Regex, bool);

pub fn compile_patterns(patterns: Vec<ScanPattern>) -> Result<Vec<CompiledScanPattern>, String> {
    patterns
        .into_iter()
        .map(|(name, pattern, requires_boundary)| {
            Regex::new(&pattern)
                .map(|regex| (Arc::<str>::from(name), regex, requires_boundary))
                .map_err(|error| error.to_string())
        })
        .collect()
}

pub fn scan_directory<F>(
    path: &Path,
    patterns: Vec<ScanPattern>,
    on_match: F,
    aborted: Arc<AtomicBool>,
    max_results: Option<usize>,
) -> Result<(), String>
where
    F: Fn(ScanResult) + Send + Sync + 'static,
{
    let scanner = Scanner::new(compile_patterns(patterns)?, on_match, aborted, max_results);
    scanner.scan_dir(path);
    Ok(())
}
