mod cache;
mod com_github_rgeorgiev583_ducd;
mod du;
mod error;
mod log;
mod varlink;
mod watcher;

#[cfg(not(windows))]
use signal_hook::{consts::signal::SIGUSR1, iterator::Signals};
use std::env::args;
use std::path::Path;
#[cfg(not(windows))]
use std::thread::spawn;

use crate::{error::Result, log::log_error, varlink::VarlinkServer, watcher::Watcher};

fn main() -> Result<()> {
    let watcher = Watcher::new()?;

    if !cfg!(windows) {
        let cache = watcher.cache.clone();
        spawn(move || -> Result<()> {
            let mut signals = Signals::new(&[SIGUSR1])?;
            for signal in signals.forever() {
                if let SIGUSR1 = signal {
                    print!("{}", cache);
                }
            }
            Ok(())
        });
    }

    for path in args().skip(1) {
        let result = watcher.watch(Path::new(&path));
        log_error(result);
    }

    let varlink_server = VarlinkServer::new(watcher);
    varlink_server.start()?;
    Ok(())
}
