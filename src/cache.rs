use crate::error::{Error, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use walkdir::WalkDir;

pub fn get_file_size(path: &Path) -> Result<i64> {
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

#[derive(Clone, Debug)]
pub struct Cache {
    inner: Rc<RefCell<HashMap<PathBuf, i64>>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, path: &Path) -> Result<i64> {
        let mut inner = self.inner.borrow_mut();
        match inner.get_mut(path) {
            Some(size) => Ok(*size),
            None => Ok(get_file_size(&path)?),
        }
    }

    pub fn update(&self, path: &Path, size: i64) -> Result<()> {
        let mut inner = self.inner.borrow_mut();
        let size_diff: i64 = match inner.insert(path.to_owned(), size) {
            Some(old_size) => size - old_size,
            None => 0,
        };
        let mut path = path.to_owned();
        let mut result = Ok(());
        loop {
            path.pop();
            match inner.get_mut(&path) {
                Some(size) => *size += size_diff,
                None => {
                    if let Ok(mut file_size) = get_file_size(&path) {
                        file_size += size_diff;
                        inner.insert(path.to_path_buf(), file_size);
                    } else {
                        result = Err(Error::DucdError(
                            format!("could not determine size of {}", path.to_string_lossy())
                                .to_owned(),
                        ));
                    }
                }
            }
            if path.parent().is_none() {
                break;
            }
        }
        result
    }

    pub fn remove(&self, path: &Path) -> Result<()> {
        let result = self.update(path, 0);
        let mut inner = self.inner.borrow_mut();
        inner.remove(path);
        result
    }

    pub fn invalidate(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.clear();
    }
}
