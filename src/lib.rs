use std::ops::RangeBounds;

pub mod config;
pub mod detail;

mod cache_impl;
use cache_impl::CacheImpl;

pub struct IOCache<Config: config::CacheConfig> {
    cache: CacheImpl<Config::Source, Config::S>,
}

impl<Config: config::CacheConfig> IOCache<Config> {
    // Panic if mem isn't enough to hold one set.
    // Panic if CacheConfig is an invalid configuration.
    pub fn new(source: Config::Source, mem: usize) -> Self {
        Self {
            cache: CacheImpl::new(source, mem),
        }
    }

    // Panic if mem isn't enough to hold one set and meta data.
    // Panic if CacheConfig is an invalid configuration.
    pub fn new_strict(source: Config::Source, mem: usize) -> Self {
        Self {
            cache: CacheImpl::new_strict(source, mem),
        }
    }

    pub fn into_source(self) -> Config::Source {
        self.cache.into_inner()
    }

    pub fn read_chunks<R: RangeBounds<u64>, F: FnMut(&[u8])>(&self, range: R, f: F) {
        self.cache.read_chunks(range, f)
    }

    pub fn read<R: RangeBounds<u64>>(&self, range: R, buf: &mut [u8]) -> usize {
        self.cache.read(range, buf)
    }

    pub fn write(&mut self, offset: u64, buf: &[u8]) -> usize {
        self.cache.write(offset, buf)
    }
}

/*
cache_config! {
    config MyConfig {
        block_size: 128,
        read_only: true,
    }
}
*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
