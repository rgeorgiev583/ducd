use hotwatch::{Event, Hotwatch};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{cache::Cache, du::space_usage, error::Result, log::log_error};

#[derive(Clone)]
pub struct Watcher {
    watcher: Arc<Mutex<Hotwatch>>,
    pub cache: Cache,
}

impl Watcher {
    pub fn new() -> Result<Self> {
        Ok(Self {
            watcher: Arc::new(Mutex::new(Hotwatch::new()?)),
            cache: Cache::new(),
        })
    }

    pub fn watch(&self, path: &Path) -> Result<()> {
        let cache = self.cache.clone();
        let handler = move |event: Event| {
            let result: Result<_> = (|| {
                match event {
                    Event::NoticeWrite(file_path) => {
                        cache.update(&file_path, space_usage(&file_path)?)
                    }
                    Event::NoticeRemove(file_path) => cache.remove(&file_path),
                    Event::Rename(old_file_path, new_file_path) => {
                        cache.remove(&old_file_path);
                        cache.update(&new_file_path, space_usage(&new_file_path)?)
                    }
                    Event::Rescan => {
                        // TODO: implement invalidation only of entries with prefix "path"
                        cache.invalidate()
                    }
                    _ => {}
                };
                Ok(())
            })();
            log_error(result);
        };
        let mut watcher = self.watcher.lock().unwrap();
        watcher.watch(path, handler)?;
        Ok(())
    }

    pub fn unwatch(&self, path: &Path) -> Result<()> {
        let mut watcher = self.watcher.lock().unwrap();
        watcher.unwatch(path)?;
        Ok(())
    }
}
