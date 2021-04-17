use crate::error::Result;
use std::fs::metadata;
use std::path::Path;
use walkdir::WalkDir;

pub fn space_usage(path: &Path) -> Result<i64> {
    let metadata = metadata(path)?;
    let file_type = metadata.file_type();
    if file_type.is_dir() {
        let mut size = 0;
        for entry in WalkDir::new(path) {
            size += entry?.metadata()?.len();
        }
        Ok(size as i64)
    } else {
        Ok(metadata.len() as i64)
    }
}
