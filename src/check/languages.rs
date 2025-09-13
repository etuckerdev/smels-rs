use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct LanguageDetector;

impl LanguageDetector {
    pub fn detect_from_extension(path: &Path) -> Option<&'static str> {
        match path.extension()?.to_str()? {
            "rs" => Some("rust"),
            "js" | "jsx" | "ts" | "tsx" => Some("javascript"),
            "py" | "pyw" => Some("python"),
            "java" => Some("java"),
            "go" => Some("go"),
            "c" | "h" => Some("c"),
            "cpp" | "cc" | "cxx" | "hpp" => Some("cpp"),
            "cs" => Some("csharp"),
            _ => None,
        }
    }

    pub fn find_source_files(
        path: &Path,
        filter: Option<&str>,
    ) -> Result<Vec<(PathBuf, &'static str)>, std::io::Error> {
        let mut files = Vec::new();

        if path.is_file() {
            if let Some(lang) = Self::detect_from_extension(path) {
                if filter.is_none_or(|f| f.split(',').any(|ext| lang.contains(ext))) {
                    files.push((path.to_path_buf(), lang));
                }
            }
        } else {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(lang) = Self::detect_from_extension(path) {
                    if filter.is_none_or(|f| f.split(',').any(|ext| lang.contains(ext))) {
                        files.push((path.to_path_buf(), lang));
                    }
                }
            }
        }

        Ok(files)
    }
}
