use super::detail::*;
use super::*;

use std::io::{Read, Seek};

pub struct CacheImpl<Source: Read + Seek, S: Sets> {
    #[allow(dead_code)]
    source: Source,
    #[allow(dead_code)]
    sets: S,
}

impl<Source: Read + Seek, S: Sets> CacheImpl<Source, S> {
    // Panic if mem isn't enough to hold one set.
    // Panic if CacheConfig is an invalid configuration.
    pub fn new(source: Source, mem: usize) -> Self {
        Self {
            source,
            sets: S::new(mem),
        }
    }

    // Panic if mem isn't enough to hold one set and meta data.
    // Panic if CacheConfig is an invalid configuration.
    pub fn new_strict(source: Source, mem: usize) -> Self {
        Self {
            source,
            sets: S::new_strict(mem),
        }
    }

    pub fn into_inner(self) -> Source {
        unimplemented!()
    }

    pub fn read_chunks<R: RangeBounds<u64>, F: FnMut(&[u8])>(&self, _range: R, _f: F) {
        unimplemented!()
    }

    pub fn read<R: RangeBounds<u64>>(&self, _range: R, _buf: &mut [u8]) -> usize {
        unimplemented!()
    }

    pub fn write(&mut self, _offset: u64, _buf: &[u8]) -> usize {
        unimplemented!()
    }
}
