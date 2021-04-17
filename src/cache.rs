use crate::du::space_usage;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Cache {
    inner: Arc<Mutex<HashMap<PathBuf, i64>>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, path: &Path) -> Result<i64> {
        let mut inner = self.inner.lock().unwrap();
        match (*inner).get_mut(path) {
            Some(size) => Ok(*size),
            None => Ok(space_usage(&path)?),
        }
    }

    pub fn update(&self, path: &Path, size: i64) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
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
                    if let Ok(mut file_size) = space_usage(&path) {
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
        let mut inner = self.inner.lock().unwrap();
        inner.remove(path);
        result
    }

    pub fn invalidate(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.clear();
    }
}

impl Display for Cache {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let inner = self.inner.lock().unwrap();
        for (path, size) in inner.iter() {
            writeln!(f, "{} {}", size, path.to_string_lossy())?;
        }
        Ok(())
    }
}
