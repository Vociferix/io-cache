// Settings:
//
// block_size: usize
// write_strategy: { WriteBack, WriteThrough }
// async_write: bool
// associativity: { DirectMapped, FullyAssociative, NWay(usize) }
// replacement: { Random, LRU, LFU, LRFU, FIFO }
// blocks_per_fetch: usize
// thread_safe: bool
// enable_stats: bool

use super::detail::*;

use std::io::{Read, Seek};

pub trait CacheConfig {
    type Source: Read + Seek;
    type BlockSize: ConstUsize;
    type Blocks: Array<u8>;
    type WriteThrough: Bool;
    type AsyncWrite: Bool;
    type Associativity: ConstUsize;
    type NWay: ConstUsize;
    type BlocksPerFetch: ConstUsize;
    type ThreadSafe: Bool;
    type EnableStats: Bool;
    type WrappedSource: InnerMut<Self::Source>;
    type S: Sets;
}

pub struct DefaultConfig<T> {
    _marker: std::marker::PhantomData<T>,
}

/*
impl<T> CacheConfig<T> for DefaultConfig<T> {
    type WrappedSource = Mutex<T>;
    type WrappedSet = RwLock<T>;
}
*/

/*
macro_rules! config_predef {
    (block_size: $size:expr) => {
        pub struct BlockData {
            data: [u8; ($size) as usize],
        }

        impl ::file_cache::BlockData for BlockData {
            fn new() -> Self {
                Self {
                    data: [0; ($size) as usize],
                }
            }

            fn get_ref(&self) -> &[u8] {
                &self.data
            }

            fn get_mut(&mut self) -> &mut [u8] {
                &mut self.data
            }
        }
    };
    (read_only: true) => {};
    (read_only: false) => {};
    (enable_stats: true) => {};
    (enable_stats: false) => {};
    (write_strategy: WriteBack) => {};
    (write_strategy: WriteThrough) => {};
    (associativity: DirectMapped) => {};
    (associativity: FullyAssociative) => {};
    (associativity: NWay($n:expr)) => {
        pub struct SetData {
            data: [BlockData; ($n) as usize],
            info: [::file_cache::BlockInfo; ($n) as usize],
        }

        impl ::file_cache::SetData<BlockData> for SetData {
            fn new() -> Self {
                Self {
                    data: [BlockData::new(); ($n) as usize],
                    info: [::file_cache::BlockInfo::new(), ($n) as usize],
                }
            }

            fn get_data_ref(&self) -> &[BlockData] {
                &self.data
            }

            fn get_data_mut(&mut self) -> &mut [BlockData] {
                &mut self.data
            }

            fn get_info_ref(&self) -> &[::file_cache::BlockInfo] {
                &self.info
            }

            fn get_info_mut(&mut self) -> &mut [::file_cache::BlockInfo] {
                &mut self.info
            }
        }
    };
    (replacement: Random) => {};
    (replacement: LRU) => {};
    (replacement: LFU) => {};
    (replacement: FIFO) => {};
    (way_prediction: true) => {};
    (way_prediction: false) => {};
    (blocks_per_fetch: $count:expr) => {};
    (async_io: true) => {};
    (async_io: false) => {};
    (thread_safe: true) => {};
    (thread_safe: false) => {};
}

macro_rules! config_entry {
    ($name:ident | block_size: $size:expr) => {
        const BLOCK_SIZE: usize = ($size) as usize;
        type BlockData = self:: $name ::file_cache_detail::BlockData;
    };
    ($name:ident | read_only: true) => { type ReadOnly = ::file_cache::True; };
    ($name:ident | read_only: false) => { type ReadOnly = ::file_cache::False; };
    ($name:ident | enable_stats: true) => { type Stats = ::file_cache::True; };
    ($name:ident | enable_stats: false) => { type Stats = ::file_cache::False; };
    ($name:ident | write_strategy: WriteBack) => { type WriteStrat = ::file_cache::WriteBack; };
    ($name:ident | write_strategy: WriteThrough) => { type WriteStrat = ::file_cache::WriteThrough; };
    ($name:ident | associativity: DirectMapped) => {
        type Assoc = ::file_cache::DirectMapped;
        const NWAY: usize = 0;
        type SetData = ::file_cache::SetDataNull;
    };
    ($name:ident | associativity: FullyAssociative) => {
        type Assoc = ::file_cache::FullyAssociative;
        const NWAY: usize = 0;
        type SetData = ::file_cache::SetDataNull;
    };
    ($name:ident | associativity: NWay($n:expr)) => {
        type Assoc = ::file_cache::NWay;
        const NWAY: usize = ($n) as usize;
        type SetData = self:: $name ::file_cache_detail::SetData;
    };
    ($name:ident | replacement: Random) => {
        type Replace = ::file_cache::Random;
        type
    };
    ($name:ident | replacement: LRU) => {};
    ($name:ident | replacement: LFU) => {};
    ($name:ident | replacement: FIFO) => {};
    ($name:ident | way_prediction: true) => {};
    ($name:ident | way_prediction: false) => {};
    ($name:ident | blocks_per_fetch: $count:expr) => {};
    ($name:ident | thread_safe: true) => {};
    ($name:ident | thread_safe: false) => {};
}

macro_rules! cache_config {
    ( config $name:ident { $( $field:ident : $value:tt ),* } ) => {
        mod $name {
            mod file_cache_detail {
                $( config_predef!($field : $value) )*
            }
        }
        struct $name ;
        impl Trait for $name {
            $( config_entry!($name | $field : $value); )*
        }
    };
    ( config $name:ident { $( $field:ident : $value:tt ),* , } ) => {
        mod $name {
            mod file_cache_detail {
                $( config_predef!($field : $value) )*
            }
        }
        struct $name ;
        impl Trait for $name {
            $( config_entry!($name | $field : $value); )*
        }
    };
}
*/