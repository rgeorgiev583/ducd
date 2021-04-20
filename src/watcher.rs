use hotwatch::{Event, Hotwatch};
use log::error;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{cache::Cache, du::space_usage, error::Result, log::log_error};

#[derive(Clone)]
pub struct Watcher {
    watcher: Arc<Mutex<Hotwatch>>,
    watched_paths: Arc<Mutex<HashSet<PathBuf>>>,
    pub cache: Cache,
}

impl Watcher {
    pub fn new() -> Result<Self> {
        Ok(Self {
            watcher: Arc::new(Mutex::new(Hotwatch::new()?)),
            watched_paths: Arc::new(Mutex::new(HashSet::new())),
            cache: Cache::new(),
        })
    }

    pub fn watch(&self, path: &Path) -> Result<()> {
        let cache = self.cache.clone();
        let handler = move |event: Event| {
            let result: Result<_> = (|| {
                match event {
                    Event::NoticeWrite(file_path) => {
                        cache.set(&file_path, space_usage(&file_path)?)
                    }
                    Event::NoticeRemove(file_path) => cache.remove(&file_path),
                    Event::Create(file_path) => cache.set(&file_path, space_usage(&file_path)?),
                    Event::Write(file_path) => cache.set(&file_path, space_usage(&file_path)?),
                    Event::Chmod(_) => {}
                    Event::Remove(file_path) => cache.remove(&file_path),
                    Event::Rename(old_file_path, new_file_path) => {
                        cache.remove(&old_file_path);
                        cache.set(&new_file_path, space_usage(&new_file_path)?)
                    }
                    Event::Rescan => {
                        // TODO: implement invalidation only of entries with prefix "path"
                        cache.invalidate()
                    }
                    Event::Error(err, _) => error!("{}", err),
                };
                Ok(())
            })();
            log_error(result);
        };
        {
            let mut watcher = self.watcher.lock().unwrap();
            watcher.watch(path, handler)?;
        }
        {
            let mut watched_paths = self.watched_paths.lock().unwrap();
            watched_paths.insert(path.to_owned());
        }
        Ok(())
    }

    pub fn unwatch(&self, path: &Path) -> Result<()> {
        {
            let mut watcher = self.watcher.lock().unwrap();
            watcher.unwatch(path)?;
        }
        {
            let mut watched_paths = self.watched_paths.lock().unwrap();
            watched_paths.remove(path);
        }
        Ok(())
    }

    pub fn is_watched(&self, path: &Path) -> bool {
        let watched_paths = self.watched_paths.lock().unwrap();
        let mut path = path.to_owned();
        loop {
            if watched_paths.contains(&path) {
                return true;
            }

            path.pop();
            if path.parent().is_none() {
                break;
            }
        }
        false
    }
}
