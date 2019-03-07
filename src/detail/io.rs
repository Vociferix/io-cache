use super::*;

use std::io::{Read, Write, Seek, SeekFrom};
use std::cell::RefCell;
use std::sync::{RwLock, Mutex, Condvar, Arc, atomic::AtomicBool, atomic::Ordering};
use std::thread::JoinHandle;

const NIL: u64 = std::u64::MAX;

pub trait Reader<Source: Read + Seek> where Self: std::marker::Sized {
    fn new(source: Source) -> std::io::Result<Self>;
    fn into_inner(self) -> Source;
    fn len(&self) -> u64;
    fn read(&self, page: u64, block: &mut [u8]) -> std::io::Result<()>;
}

pub trait Writer<Source: Read + Write + Seek>: Reader<Source> {
    fn write(&mut self, page: u64, block: &[u8]) -> std::io::Result<()>;
}

struct SrcInfo<Source: Read + Seek> {
    source: Source,
    len: u64,
}

impl<Source: Read + Seek> SrcInfo<Source> {
    fn new(mut source: Source) -> std::io::Result<Self> {
        let len = source.seek(SeekFrom::End(0))?;
        Ok(Self {
            source,
            len,
        })
    }
}

pub struct SyncIO<Source: Read + Seek, BlockSz: ConstUsize> {
    source: RefCell<SrcInfo<Source>>,
    _marker: std::marker::PhantomData<BlockSz>,
}

impl<Source: Read + Seek, BlockSz: ConstUsize> Reader<Source> for SyncIO<Source, BlockSz> {
    fn new(source: Source) -> std::io::Result<Self> {
        Ok(Self {
            source: RefCell::new(SrcInfo::new(source)?),
            _marker: std::marker::PhantomData,
        })
    }

    fn into_inner(self) -> Source {
        self.source.into_inner().source
    }

    fn len(&self) -> u64 {
        self.source.borrow().len
    }

    fn read(&self, page: u64, block: &mut [u8]) -> std::io::Result<()> {
        let mut s = self.source.borrow_mut();
        s.source.seek(SeekFrom::Start(page * BlockSz::VALUE as u64))?;
        s.source.read(block)?;
        Ok(())
    }
}

impl<Source: Read + Write + Seek, BlockSz: ConstUsize> Writer<Source> for SyncIO<Source, BlockSz> {
    fn write(&mut self, page: u64, block: &[u8]) -> std::io::Result<()> {
        let mut s = self.source.get_mut();
        let pos = s.source.seek(SeekFrom::Start(page * BlockSz::VALUE as u64))?;
        let pos = pos + s.source.write(block)? as u64;
        let len = s.len;
        if len < pos {
            s.len = pos;
        }
        Ok(())
    }
}

struct AsyncIOMeta<Block: Array<u8> + Send + Sync, Queue: Array<(u64, Block)> + Send + Sync, Table: Array<(u64, usize)> + Send + Sync> {
    queue: Queue,
    table: Table,
    front: usize,
    back: usize,
    end: bool,
    _marker: std::marker::PhantomData<Block>,
}

impl<Block: Array<u8> + Send + Sync, Queue: Array<(u64, Block)> + Send + Sync, Table: Array<(u64, usize)> + Send + Sync> AsyncIOMeta<Block, Queue, Table> {
    fn new() -> Self {
        Self {
            queue: Queue::new(),
            table: Table::new_with((NIL, 0)),
            front: 0,
            back: 0,
            end: false,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct AsyncIO<Source: Read + Write + Seek + Send + Sync, Block: Array<u8> + Send + Sync, Queue: Array<(u64, Block)> + Send + Sync, Table: Array<(u64, usize)> + Send + Sync> {
    inner: Arc<AsyncIOImpl<Source, Block, Queue, Table>>,
    worker: Mutex<Option<JoinHandle<std::io::Result<()>>>>,
}

pub struct AsyncIOImpl<Source: Read + Write + Seek + Send + Sync, Block: Array<u8> + Send + Sync, Queue: Array<(u64, Block)> + Send + Sync, Table: Array<(u64, usize)> + Send + Sync> {
    source: RwLock<SrcInfo<Source>>,
    meta: Mutex<AsyncIOMeta<Block, Queue, Table>>,
    condvar: Condvar,
    error_flag: AtomicBool,
}

fn async_io_worker<Source, Block, Queue, Table>(data: Arc<AsyncIOImpl<Source, Block, Queue, Table>>) -> std::io::Result<()>
    where Source: Read + Write + Seek + Send + Sync, Block: Array<u8> + Send + Sync, Queue: Array<(u64, Block)> + Send + Sync, Table: Array<(u64, usize)> + Send + Sync
{
    let mut block = Block::new();
    let mut page: u64 = 0;
    loop {
        {
            let mut lock = data.meta.lock().unwrap();
            let end = loop {
                lock = data.condvar.wait(lock).unwrap();

                if lock.end {
                    break true;
                }

                if lock.front != lock.back {
                    page = lock.queue.get_ref()[lock.front].0;
                    block.get_mut().write(lock.queue.get_ref()[lock.front].1.get_ref())?;
                    break false;
                }
            };

            if end { return Ok(()); }
        }

        {
            let mut lock = data.source.write().unwrap();
            lock.source.seek(SeekFrom::Start(page * Block::LEN as u64))?;
            lock.source.write(block.get_ref())?;
        }

        {
            let mut lock = data.meta.lock().unwrap();
            lock.front = (lock.front + 1) % Queue::LEN;
            let mut idx = (hash64(page) % Table::LEN as u64) as usize;
            {
                let table = lock.table.get_mut();
                while table[idx].0 != page {
                    idx = (idx + 1) % Table::LEN;
                }
                table[idx].0 = NIL;
            }
        }
    }
}

impl<Source, Block, Queue, Table> AsyncIO<Source, Block, Queue, Table>
    where Source: Read + Write + Seek + Send + Sync + 'static, Block: Array<u8> + Send + Sync + 'static, Queue: Array<(u64, Block)> + Send + Sync + 'static, Table: Array<(u64, usize)> + Send + Sync + 'static
{
    fn check_error(&self) -> std::io::Result<()> {
        if self.inner.error_flag.swap(false, Ordering::SeqCst) {
            let mut lock = self.worker.lock().unwrap();
            let ret = std::mem::replace(&mut *lock, None).unwrap().join().unwrap();
            let t = self.inner.clone();
            *lock = Some(std::thread::spawn(move || { async_io_worker(t) }));
            ret
        } else {
            Ok(())
        }
    }
}

impl<Source, Block, Queue, Table> Reader<Source> for AsyncIO<Source, Block, Queue, Table>
    where Source: Read + Write + Seek + Send + Sync + 'static, Block: Array<u8> + Send + Sync + 'static, Queue: Array<(u64, Block)> + Send + Sync + 'static, Table: Array<(u64, usize)> + Send + Sync + 'static
{
    fn new(source: Source) -> std::io::Result<Self> {
        let inner = Arc::new(AsyncIOImpl {
            source:RwLock::new(SrcInfo::new(source)?),
            meta: Mutex::new(AsyncIOMeta::new()),
            condvar: Condvar::new(),
            error_flag: AtomicBool::new(false),
        });
        let t = inner.clone();
        Ok(Self {
            inner,
            worker: Mutex::new(Some(std::thread::spawn(move || { async_io_worker(t) }))),
        })
    }

    fn into_inner(self) -> Source {
        self.inner.meta.lock().unwrap().end = true;
        self.inner.condvar.notify_one();
        let _ = self.worker.into_inner().unwrap().unwrap().join();
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => inner,
            _ => panic!("Failed to unwrap Arc"),
        }.source.into_inner().unwrap().source
    }

    fn len(&self) -> u64 {
        self.inner.source.read().unwrap().len
    }

    fn read(&self, page: u64, block: &mut [u8]) -> std::io::Result<()> {
        self.check_error()?;
        let s = &mut self.inner.source.write().unwrap().source;
        s.seek(SeekFrom::Start(page * Block::LEN as u64))?;
        s.read(block)?;
        Ok(())
    }
}

impl<Source, Block, Queue, Table> Writer<Source> for AsyncIO<Source, Block, Queue, Table>
    where Source: Read + Write + Seek + Send + Sync + 'static, Block: Array<u8> + Send + Sync + 'static, Queue: Array<(u64, Block)> + Send + Sync + 'static, Table: Array<(u64, usize)> + Send + Sync + 'static
{
    fn write(&mut self, _page: u64, _block: &[u8]) -> std::io::Result<()> {
        self.check_error()?;
        unimplemented!()
    }
}