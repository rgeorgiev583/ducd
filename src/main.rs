mod cache;
mod error;

use cache::{get_file_size, Cache};
use error::{Error, Result};
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use log::error;
use std::path::Path;

fn main() -> Result<()> {
    let args = std::env::args();
    if args.len() == 0 {
        return Err(Error::DucdError("no paths to watch provided".to_owned()));
    }

    let cache = Cache::new();
    let mut watcher = Hotwatch::new().expect("path watcher failed to initialize");
    for path in args.skip(1) {
        let cache = cache.clone();
        let result = watcher.watch(Path::new(&path), move |event: Event| {
            let result = (|| match event {
                Event::NoticeWrite(file_path) => {
                    cache.update(&file_path, get_file_size(&file_path)?)
                }
                Event::NoticeRemove(file_path) => cache.remove(&file_path),
                Event::Rename(old_file_path, new_file_path) => {
                    let mut result = cache.remove(&old_file_path);
                    if let Err(err) = cache.update(&new_file_path, get_file_size(&new_file_path)?) {
                        result = Err(err);
                    }
                    result
                }
                Event::Rescan => {
                    // TODO: implement invalidation only of entries with prefix "path"
                    cache.invalidate();
                    Ok(())
                }
                _ => Ok(()),
            })();
            if let Err(err) = result {
                error!("{}", err);
            }
            Flow::Continue
        });
        if let Err(err) = result {
            error!("{}", err);
        }
    }
    watcher.run();
    Ok(())
}
