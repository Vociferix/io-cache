use super::*;

const NULL: usize = std::usize::MAX;
const NIL: u64 = std::u64::MAX;
const DEL: u64 = NIL - 1;

pub trait Lookup {
    const STATIC_META_MEM: usize;
    const META_MEM_PER_BLOCK: usize;

    fn new(count: usize) -> Self;
    fn find(&self, page: u64) -> usize;
    fn insert(&mut self, page: u64, frame: usize);
    fn remove(&mut self, page: u64, frame_hint: usize);
}

pub struct DMLookup {}

impl Lookup for DMLookup {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = 0;

    fn new(_: usize) -> Self {
        Self {}
    }
    fn find(&self, _: u64) -> usize {
        0
    }
    fn insert(&mut self, _: u64, _: usize) {}
    fn remove(&mut self, _: u64, _: usize) {}
}

pub struct Table<Tbl: Array<(u64, usize)>> {
    table: Tbl,
}

impl<Tbl: Array<(u64, usize)>> Lookup for Table<Tbl> {
    const STATIC_META_MEM: usize =
        std::mem::size_of::<Self>() - (Self::META_MEM_PER_BLOCK * Tbl::LEN / 2);
    const META_MEM_PER_BLOCK: usize = std::mem::size_of::<(u64, usize)>() * 2;

    fn new(_: usize) -> Self {
        Self {
            table: Tbl::new_with((NIL, NULL)),
        }
    }

    fn find(&self, page: u64) -> usize {
        let idx_init = (hash64(page) & (Tbl::LEN as u64 - 1)) as usize;
        let mut idx = idx_init;
        let tbl = self.table.get_ref();
        {
            let entry = &tbl[idx];
            if entry.0 == NIL {
                return NULL;
            }
            if entry.0 == page {
                return entry.1;
            }
            idx = (idx + 1) & (Tbl::LEN - 1);
        }
        while idx != idx_init {
            let entry = &tbl[idx];
            if entry.0 == NIL {
                return NULL;
            }
            if entry.0 == page {
                return entry.1;
            }
            idx = (idx + 1) & (Tbl::LEN - 1);
        }
        return NULL;
    }

    fn insert(&mut self, page: u64, frame: usize) {
        let mut idx = (hash64(page) & (Tbl::LEN as u64 - 1)) as usize;
        let tbl = self.table.get_mut();
        loop {
            let entry = &mut tbl[idx];
            if entry.0 == DEL || entry.0 == NIL {
                entry.0 = page;
                entry.1 = frame;
                return;
            }
            idx = (idx + 1) & (Tbl::LEN - 1);
        }
    }

    fn remove(&mut self, page: u64, _: usize) {
        let mut idx = (hash64(page) & (Tbl::LEN as u64 - 1)) as usize;
        let tbl = self.table.get_mut();
        loop {
            let entry = &mut tbl[idx];
            if entry.0 == page {
                entry.0 = DEL;
                return;
            }
            idx = (idx + 1) & (Tbl::LEN - 1);
        }
    }
}

pub struct Scan<Blocks: Array<u64>> {
    blocks: Blocks,
}

impl<Blocks: Array<u64>> Lookup for Scan<Blocks> {
    const STATIC_META_MEM: usize =
        std::mem::size_of::<Self>() - (Self::META_MEM_PER_BLOCK * Blocks::LEN);
    const META_MEM_PER_BLOCK: usize = std::mem::size_of::<u64>();

    fn new(_: usize) -> Self {
        Self {
            blocks: Blocks::new_with(NIL),
        }
    }

    fn find(&self, page: u64) -> usize {
        for (f, p) in self.blocks.get_ref().iter().enumerate() {
            if *p == page {
                return f;
            }
        }
        return NULL;
    }

    fn insert(&mut self, page: u64, frame: usize) {
        self.blocks.get_mut()[frame] = page;
    }

    fn remove(&mut self, page: u64, frame_hint: usize) {
        if frame_hint < Blocks::LEN {
            self.blocks.get_mut()[frame_hint] = NIL;
        } else {
            let frame = self.find(page);
            self.blocks.get_mut()[frame] = NIL;
        }
    }
}

pub struct FATable {
    table: Vec<(u64, usize)>,
}

impl Lookup for FATable {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = std::mem::size_of::<(u64, usize)>() * 2;

    fn new(count: usize) -> Self {
        Self {
            table: vec![(NIL, NULL); (count * 3) / 2],
        }
    }

    fn find(&self, page: u64) -> usize {
        let idx_init = (hash64(page) % self.table.len() as u64) as usize;
        let mut idx = idx_init;
        {
            let entry = self.table[idx];
            if entry.0 == NIL {
                return NULL;
            }
            if entry.0 == page {
                return entry.1;
            }
            idx = (idx + 1) % self.table.len();
        }
        while idx != idx_init {
            let entry = self.table[idx];
            if entry.0 == NIL {
                return NULL;
            }
            if entry.0 == page {
                return entry.1;
            }
            idx = (idx + 1) % self.table.len();
        }
        return NULL;
    }

    fn insert(&mut self, page: u64, frame: usize) {
        let mut idx = (hash64(page) % self.table.len() as u64) as usize;
        loop {
            let entry = &mut self.table[idx];
            if entry.0 == DEL || entry.0 == NIL {
                entry.0 = page;
                entry.1 = frame;
                return;
            }
            idx = (idx + 1) % self.table.len();
        }
    }

    fn remove(&mut self, page: u64, _: usize) {
        let mut idx = (hash64(page) % self.table.len() as u64) as usize;
        loop {
            let entry = &mut self.table[idx];
            if entry.0 == page {
                entry.0 = DEL;
                return;
            }
            idx = (idx + 1) % self.table.len();
        }
    }
}

pub struct FAScan {
    blocks: Vec<u64>,
}

impl Lookup for FAScan {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = std::mem::size_of::<u64>();

    fn new(count: usize) -> Self {
        Self {
            blocks: vec![NIL; count],
        }
    }

    fn find(&self, page: u64) -> usize {
        for (f, p) in self.blocks.iter().enumerate() {
            if *p == page {
                return f;
            }
        }
        return NULL;
    }

    fn insert(&mut self, page: u64, frame: usize) {
        self.blocks[frame] = page;
    }

    fn remove(&mut self, page: u64, frame_hint: usize) {
        if frame_hint < self.blocks.len() {
            self.blocks[frame_hint] = NIL;
        } else {
            let frame = self.find(page);
            self.blocks[frame] = NIL;
        }
    }
}
