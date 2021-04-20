use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{du::space_usage, error::Result};

#[derive(Debug)]
struct CacheMap {
    inner: HashMap<PathBuf, i64>,
}

impl CacheMap {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn get(&self, path: &Path) -> Result<i64> {
        match self.inner.get(path) {
            Some(size) => Ok(*size),
            None => Ok(space_usage(&path)?),
        }
    }

    fn set(&mut self, path: &Path, size: i64) {
        let size_diff: i64 = match self.inner.insert(path.to_owned(), size) {
            Some(old_size) => size - old_size,
            None => size,
        };
        let mut path = path.to_owned();
        loop {
            path.pop();
            if let Some(size) = self.inner.get_mut(&path) {
                *size += size_diff;
            }
            if path.parent().is_none() {
                break;
            }
        }
    }

    fn update(&mut self, path: &Path) -> Result<i64> {
        let size = self.get(path)?;
        self.set(path, size);
        Ok(size)
    }

    fn remove(&mut self, path: &Path) {
        self.set(path, 0);
        self.inner.remove(path);
    }

    fn invalidate(&mut self) {
        self.inner.clear();
    }

    fn to_string_map(&self) -> HashMap<String, i64> {
        self.inner
            .iter()
            .map(|(path, size)| (path.to_string_lossy().to_string(), *size))
            .collect()
    }
}

impl Display for CacheMap {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for (path, size) in self.inner.iter() {
            writeln!(f, "{} {}", size, path.to_string_lossy())?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Cache {
    inner: Arc<Mutex<CacheMap>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(CacheMap::new())),
        }
    }

    pub fn get(&self, path: &Path) -> Result<i64> {
        let inner = self.inner.lock().unwrap();
        inner.get(path)
    }

    pub fn set(&self, path: &Path, size: i64) {
        let mut inner = self.inner.lock().unwrap();
        inner.set(path, size)
    }

    pub fn update(&self, path: &Path) -> Result<i64> {
        let mut inner = self.inner.lock().unwrap();
        inner.update(path)
    }

    pub fn remove(&self, path: &Path) {
        let mut inner = self.inner.lock().unwrap();
        inner.remove(path)
    }

    pub fn invalidate(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.invalidate()
    }

    pub fn to_string_map(&self) -> HashMap<String, i64> {
        let inner = self.inner.lock().unwrap();
        inner.to_string_map()
    }
}

impl Display for Cache {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let inner = self.inner.lock().unwrap();
        inner.fmt(f)
    }
}
