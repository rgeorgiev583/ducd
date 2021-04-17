mod cache;
mod com_github_rgeorgiev583_ducd;
mod du;
mod error;
mod varlink;

use crate::varlink::VarlinkServer;
use cache::Cache;
use du::space_usage;
use error::{Error, Result};
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use log::error;
#[cfg(not(windows))]
use signal_hook::{consts::signal::SIGUSR1, iterator::Signals};
use std::path::Path;
#[cfg(not(windows))]
use std::thread::spawn;

fn log_error<T, E: std::fmt::Display>(result: std::result::Result<T, E>) {
    if let Err(err) = result {
        error!("{}", err);
    }
}

fn main() -> Result<()> {
    let args = std::env::args();
    if args.len() == 0 {
        return Err(Error::DucdError("no paths to watch provided".to_owned()));
    }

    let cache = Cache::new();

    if !cfg!(windows) {
        let cache = cache.clone();
        spawn(move || -> Result<()> {
            let mut signals = Signals::new(&[SIGUSR1])?;
            for signal in &mut signals {
                if let SIGUSR1 = signal {
                    print!("{}", cache);
                }
            }
            Ok(())
        });
    }

    {
        let cache = cache.clone();
        spawn(move || -> Result<()> {
            let varlink_server = VarlinkServer::new(cache);
            varlink_server.start()?;
            Ok(())
        });
    }

    let mut watcher = Hotwatch::new()?;
    for path in args.skip(1) {
        let cache = cache.clone();
        let result = watcher.watch(Path::new(&path), move |event: Event| {
            let result = (|| match event {
                Event::NoticeWrite(file_path) => cache.update(&file_path, space_usage(&file_path)?),
                Event::NoticeRemove(file_path) => cache.remove(&file_path),
                Event::Rename(old_file_path, new_file_path) => {
                    let result = cache.remove(&old_file_path);
                    log_error(result);
                    cache.update(&new_file_path, space_usage(&new_file_path)?)
                }
                Event::Rescan => {
                    // TODO: implement invalidation only of entries with prefix "path"
                    cache.invalidate();
                    Ok(())
                }
                _ => Ok(()),
            })();
            log_error(result);
            Flow::Continue
        });
        log_error(result);
    }
    watcher.run();

    Ok(())
}
