mod cache;
mod com_github_rgeorgiev583_ducd;
mod du;
mod error;
mod log;
mod varlink;
mod watcher;

#[macro_use]
extern crate clap;

use std::env::args;
use std::path::Path;

use crate::{error::Result, log::log_error, varlink::VarlinkServer, watcher::Watcher};

fn main() -> Result<()> {
    let options = app_from_crate!()
        .args_from_usage(
            "-a, --address=[ADDRESS] 'Sets a custom address for the Varlink server'
             [PATH]...               'Sets paths to watch'",
        )
        .get_matches();

    let watcher = Watcher::new()?;

    for path in args().skip(1) {
        let result = watcher.watch(Path::new(&path));
        log_error(result);
    }

    let varlink_server = VarlinkServer::new(watcher);
    varlink_server.start(options.value_of("address"))?;
    Ok(())
}
