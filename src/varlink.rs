use crate::cache::Cache;
use crate::com_github_rgeorgiev583_ducd::{
    self, Call_GetSpaceUsage, Call_InvalidateCache, VarlinkInterface,
};
use crate::error::Error;
use std::path::Path;

#[derive(Clone)]
pub struct VarlinkServer {
    cache: Cache,
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
        path: String,
    ) -> varlink::Result<()> {
        let path = Path::new(&path);
        let size = self.cache.get(path)?;
        self.cache.update(path, size);
        call.reply(size)
    }

    fn invalidate_cache(&self, call: &mut dyn Call_InvalidateCache) -> varlink::Result<()> {
        self.cache.invalidate();
        call.reply()
    }
}

impl VarlinkServer {
    pub fn new(cache: Cache) -> Self {
        Self { cache: cache }
    }

    pub fn start(&self) -> varlink::Result<()> {
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
            "ducd",
            "0.1.0",
            "https://github.com/rgeorgiev583/ducd",
            vec![Box::new(ducd_interface)],
        );
        varlink::listen(
            service,
            VARLINK_ADDRESS,
            &varlink::ListenConfig {
                ..Default::default()
            },
        )
    }
}
