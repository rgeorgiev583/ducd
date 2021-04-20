use crate::{
    com_github_rgeorgiev583_ducd::{
        self, Call_GetSpaceUsage, Call_InvalidateCache, Call_ListCacheEntries, Call_StartWatching,
        Call_StopWatching, VarlinkInterface,
    },
    error::Error,
    watcher::Watcher,
};
use std::path::Path;

#[derive(Clone)]
pub struct VarlinkServer {
    watcher: Watcher,
}

impl From<Error> for varlink::error::Error {
    fn from(err: Error) -> Self {
        varlink::error::Error(
            varlink::error::ErrorKind::Generic,
            Some(Box::new(err)),
            None,
        )
    }
}

impl VarlinkInterface for VarlinkServer {
    fn get_space_usage(
        &self,
        call: &mut dyn Call_GetSpaceUsage,
        r#path: String,
    ) -> varlink::Result<()> {
        let path = Path::new(&path);
        if !self.watcher.is_watched(path) {
            self.watcher.watch(path)?;
        }
        let size = self.watcher.cache.update(path)?;
        call.reply(size)
    }

    fn start_watching(
        &self,
        call: &mut dyn Call_StartWatching,
        r#path: String,
    ) -> varlink::Result<()> {
        let path = Path::new(&path);
        if self.watcher.is_watched(path) {
            return Err(Error::DucdError(format!(
                "{} is already being watched",
                path.to_string_lossy()
            ))
            .into());
        }

        self.watcher.watch(path)?;
        call.reply()
    }

    fn stop_watching(
        &self,
        call: &mut dyn Call_StopWatching,
        r#path: String,
    ) -> varlink::Result<()> {
        let path = Path::new(&path);
        if !self.watcher.is_watched(path) {
            return Err(Error::DucdError(format!(
                "{} is not being watched",
                path.to_string_lossy()
            ))
            .into());
        }

        self.watcher.unwatch(path)?;
        call.reply()
    }

    fn list_cache_entries(&self, call: &mut dyn Call_ListCacheEntries) -> varlink::Result<()> {
        call.reply(self.watcher.cache.to_string_map())
    }

    fn invalidate_cache(&self, call: &mut dyn Call_InvalidateCache) -> varlink::Result<()> {
        self.watcher.cache.invalidate();
        call.reply()
    }
}

impl VarlinkServer {
    pub fn new(watcher: Watcher) -> Self {
        Self { watcher }
    }

    pub fn start(&self, address: Option<&str>) -> varlink::Result<()> {
        const VARLINK_ADDRESS: &str = if cfg!(target_os = "windows") {
            "tcp:127.0.0.1:42069"
        } else if cfg!(target_os = "linux") {
            "unix:@com.github.rgeorgiev583.ducd"
        } else {
            "unix:/run/com.github.rgeorgiev583.ducd"
        };
        let ducd_interface = com_github_rgeorgiev583_ducd::new(Box::new(self.clone()));
        let service = varlink::VarlinkService::new(
            "com.github.rgeorgiev583",
            crate_name!(),
            crate_version!(),
            env!("CARGO_PKG_HOMEPAGE"),
            vec![Box::new(ducd_interface)],
        );
        varlink::listen(
            service,
            match address {
                Some(address) => address,
                None => VARLINK_ADDRESS,
            },
            &varlink::ListenConfig {
                ..Default::default()
            },
        )
    }
}
