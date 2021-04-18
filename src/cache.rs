use crate::{du::space_usage, error::Result};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

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

    pub fn update(&self, path: &Path, size: i64) {
        let mut inner = self.inner.lock().unwrap();
        let size_diff: i64 = match inner.insert(path.to_owned(), size) {
            Some(old_size) => size - old_size,
            None => size,
        };
        let mut path = path.to_owned();
        loop {
            path.pop();
            if let Some(size) = inner.get_mut(&path) {
                *size += size_diff;
            }
            if path.parent().is_none() {
                break;
            }
        }
    }

    pub fn remove(&self, path: &Path) {
        self.update(path, 0);
        let mut inner = self.inner.lock().unwrap();
        inner.remove(path);
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
