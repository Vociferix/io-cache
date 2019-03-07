use super::*;

pub trait Set {
    type L: Lookup;
    type R: Replace;

    const STATIC_META_MEM: usize;
    const META_MEM_PER_BLOCK: usize;

    fn lookup(&self) -> &Self::L;
    fn lookup_mut(&mut self) -> &mut Self::L;
    fn replace(&self) -> &Self::R;
    fn replace_mut(&mut self) -> &mut Self::R;
    fn block(&self, idx: usize) -> &[u8];
    fn block_mut(&mut self, idx: usize) -> &mut [u8];
}

pub struct NWaySet<L: Lookup, R: Replace, Block: Array<u8>, Blocks: Array<Block>> {
    blocks: Blocks,
    lookup: L,
    replace: R,
    _marker: std::marker::PhantomData<Block>,
}

impl<L: Lookup, R: Replace, Block: Array<u8>, Blocks: Array<Block>> NWaySet<L, R, Block, Blocks> {
    fn new() -> Self {
        Self {
            blocks: Blocks::new(),
            lookup: L::new(0),
            replace: R::new(0),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<L: Lookup, R: Replace, Block: Array<u8>, Blocks: Array<Block>> Set
    for NWaySet<L, R, Block, Blocks>
{
    type L = L;
    type R = R;

    const STATIC_META_MEM: usize =
        std::mem::size_of::<Self>() - (Self::META_MEM_PER_BLOCK + Block::LEN);
    const META_MEM_PER_BLOCK: usize = L::META_MEM_PER_BLOCK + R::META_MEM_PER_BLOCK;

    fn lookup(&self) -> &Self::L {
        &self.lookup
    }

    fn lookup_mut(&mut self) -> &mut Self::L {
        &mut self.lookup
    }

    fn replace(&self) -> &Self::R {
        &self.replace
    }

    fn replace_mut(&mut self) -> &mut Self::R {
        &mut self.replace
    }

    fn block(&self, idx: usize) -> &[u8] {
        self.blocks.get_ref()[idx].get_ref()
    }

    fn block_mut(&mut self, idx: usize) -> &mut [u8] {
        self.blocks.get_mut()[idx].get_mut()
    }
}

pub struct DirectMappedSet<Block: Array<u8>> {
    block: Block,
    lookup: DMLookup,
    replace: DMReplace,
}

impl<Block: Array<u8>> DirectMappedSet<Block> {
    fn new() -> Self {
        Self {
            block: Block::new(),
            lookup: DMLookup::new(0),
            replace: DMReplace::new(0),
        }
    }
}

impl<Block: Array<u8>> Set for DirectMappedSet<Block> {
    type L = DMLookup;
    type R = DMReplace;

    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = 0;

    fn lookup(&self) -> &Self::L {
        &self.lookup
    }

    fn lookup_mut(&mut self) -> &mut Self::L {
        &mut self.lookup
    }

    fn replace(&self) -> &Self::R {
        &self.replace
    }

    fn replace_mut(&mut self) -> &mut Self::R {
        &mut self.replace
    }

    fn block(&self, _: usize) -> &[u8] {
        self.block.get_ref()
    }

    fn block_mut(&mut self, _: usize) -> &mut [u8] {
        self.block.get_mut()
    }
}

pub struct FullyAssociativeSet<L: Lookup, R: Replace, Block: Array<u8>> {
    blocks: Vec<Block>,
    lookup: L,
    replace: R,
}

impl<L: Lookup, R: Replace, Block: Array<u8>> FullyAssociativeSet<L, R, Block> {
    fn new(count: usize) -> Self {
        Self {
            blocks: vec![Block::new(); count],
            lookup: L::new(count),
            replace: R::new(count),
        }
    }
}

impl<L: Lookup, R: Replace, Block: Array<u8>> Set for FullyAssociativeSet<L, R, Block> {
    type L = L;
    type R = R;

    const STATIC_META_MEM: usize =
        std::mem::size_of::<Self>() - (Self::META_MEM_PER_BLOCK + Block::LEN);
    const META_MEM_PER_BLOCK: usize = L::META_MEM_PER_BLOCK + R::META_MEM_PER_BLOCK;

    fn lookup(&self) -> &Self::L {
        &self.lookup
    }

    fn lookup_mut(&mut self) -> &mut Self::L {
        &mut self.lookup
    }

    fn replace(&self) -> &Self::R {
        &self.replace
    }

    fn replace_mut(&mut self) -> &mut Self::R {
        &mut self.replace
    }

    fn block(&self, idx: usize) -> &[u8] {
        self.blocks[idx].get_ref()
    }

    fn block_mut(&mut self, idx: usize) -> &mut [u8] {
        self.blocks[idx].get_mut()
    }
}

pub trait Sets {
    type S: Set;
    type IMS: InnerMut<Self::S>;

    fn new(mem: usize) -> Self;
    fn new_strict(mem: usize) -> Self;
    fn count(&self) -> usize;
    fn set(&self, page: u64) -> &Self::IMS;
    fn data_mem(&self) -> usize;
    fn meta_mem(&self) -> usize;

    fn total_mem(&self) -> usize {
        self.data_mem() + self.meta_mem()
    }
}

pub struct NWaySets<
    L: Lookup,
    R: Replace,
    Block: Array<u8>,
    Blocks: Array<Block>,
    S: InnerMut<NWaySet<L, R, Block, Blocks>>,
> {
    sets: Vec<S>,
    _marker1: std::marker::PhantomData<L>,
    _marker2: std::marker::PhantomData<R>,
    _marker3: std::marker::PhantomData<Block>,
    _marker4: std::marker::PhantomData<Blocks>,
}

impl<
        L: Lookup,
        R: Replace,
        Block: Array<u8>,
        Blocks: Array<Block>,
        S: InnerMut<NWaySet<L, R, Block, Blocks>>,
    > NWaySets<L, R, Block, Blocks, S>
{
    const MEM_PER_SET: usize = (Block::LEN * Blocks::LEN)
        + NWaySet::<L, R, Block, Blocks>::STATIC_META_MEM
        + (NWaySet::<L, R, Block, Blocks>::META_MEM_PER_BLOCK * Blocks::LEN);
}

impl<
        L: Lookup,
        R: Replace,
        Block: Array<u8>,
        Blocks: Array<Block>,
        S: InnerMut<NWaySet<L, R, Block, Blocks>>,
    > Sets for NWaySets<L, R, Block, Blocks, S>
{
    type S = NWaySet<L, R, Block, Blocks>;
    type IMS = S;

    fn new(mem: usize) -> Self {
        let set_count = mem / (Blocks::LEN * Block::LEN);
        let mut sets: Vec<S> = Vec::new();
        sets.reserve(set_count);
        for _ in 0..set_count {
            sets.push(S::new(NWaySet::new()));
        }
        Self {
            sets,
            _marker1: std::marker::PhantomData,
            _marker2: std::marker::PhantomData,
            _marker3: std::marker::PhantomData,
            _marker4: std::marker::PhantomData,
        }
    }

    fn new_strict(mem: usize) -> Self {
        let set_count = (mem - std::mem::size_of::<Self>()) / Self::MEM_PER_SET;
        let mut sets: Vec<S> = Vec::new();
        sets.reserve(set_count);
        for _ in 0..set_count {
            sets.push(S::new(NWaySet::new()));
        }
        Self {
            sets,
            _marker1: std::marker::PhantomData,
            _marker2: std::marker::PhantomData,
            _marker3: std::marker::PhantomData,
            _marker4: std::marker::PhantomData,
        }
    }

    fn count(&self) -> usize {
        self.sets.len()
    }

    fn set(&self, page: u64) -> &Self::IMS {
        &self.sets[(page % self.sets.len() as u64) as usize]
    }

    fn data_mem(&self) -> usize {
        self.sets.len() * Blocks::LEN * Block::LEN
    }

    fn meta_mem(&self) -> usize {
        self.sets.len()
            * (NWaySet::<L, R, Block, Blocks>::STATIC_META_MEM
                + (NWaySet::<L, R, Block, Blocks>::META_MEM_PER_BLOCK * Blocks::LEN))
    }
}

pub struct DirectMappedSets<Block: Array<u8>, S: InnerMut<DirectMappedSet<Block>>> {
    sets: Vec<S>,
    _marker: std::marker::PhantomData<Block>,
}

impl<Block: Array<u8>, S: InnerMut<DirectMappedSet<Block>>> DirectMappedSets<Block, S> {
    const MEM_PER_SET: usize = Block::LEN
        + DirectMappedSet::<Block>::STATIC_META_MEM
        + DirectMappedSet::<Block>::META_MEM_PER_BLOCK;
}

impl<Block: Array<u8>, S: InnerMut<DirectMappedSet<Block>>> Sets for DirectMappedSets<Block, S> {
    type S = DirectMappedSet<Block>;
    type IMS = S;

    fn new(mem: usize) -> Self {
        let set_count = mem / Block::LEN;
        let mut sets: Vec<S> = Vec::new();
        sets.reserve(set_count);
        for _ in 0..set_count {
            sets.push(S::new(DirectMappedSet::new()));
        }
        Self {
            sets,
            _marker: std::marker::PhantomData,
        }
    }

    fn new_strict(mem: usize) -> Self {
        let set_count = (mem - std::mem::size_of::<Self>()) / Self::MEM_PER_SET;
        let mut sets: Vec<S> = Vec::new();
        sets.reserve(set_count);
        for _ in 0..set_count {
            sets.push(S::new(DirectMappedSet::new()));
        }
        Self {
            sets,
            _marker: std::marker::PhantomData,
        }
    }

    fn count(&self) -> usize {
        self.sets.len()
    }

    fn set(&self, page: u64) -> &Self::IMS {
        &self.sets[(page % self.sets.len() as u64) as usize]
    }

    fn data_mem(&self) -> usize {
        self.sets.len() * Block::LEN
    }

    fn meta_mem(&self) -> usize {
        self.sets.len()
            * (DirectMappedSet::<Block>::STATIC_META_MEM
                + DirectMappedSet::<Block>::META_MEM_PER_BLOCK)
    }
}

pub struct FullyAssociativeSets<
    L: Lookup,
    R: Replace,
    Block: Array<u8>,
    S: InnerMut<FullyAssociativeSet<L, R, Block>>,
> {
    set: S,
    _marker1: std::marker::PhantomData<L>,
    _marker2: std::marker::PhantomData<R>,
    _marker3: std::marker::PhantomData<Block>,
}

impl<L: Lookup, R: Replace, Block: Array<u8>, S: InnerMut<FullyAssociativeSet<L, R, Block>>>
    FullyAssociativeSets<L, R, Block, S>
{
    const MEM_PER_BLOCK: usize =
        Block::LEN + FullyAssociativeSet::<L, R, Block>::META_MEM_PER_BLOCK;
}

impl<L: Lookup, R: Replace, Block: Array<u8>, S: InnerMut<FullyAssociativeSet<L, R, Block>>> Sets
    for FullyAssociativeSets<L, R, Block, S>
{
    type S = FullyAssociativeSet<L, R, Block>;
    type IMS = S;

    fn new(mem: usize) -> Self {
        Self {
            set: S::new(FullyAssociativeSet::new(mem / Block::LEN)),
            _marker1: std::marker::PhantomData,
            _marker2: std::marker::PhantomData,
            _marker3: std::marker::PhantomData,
        }
    }

    fn new_strict(mem: usize) -> Self {
        Self {
            set: S::new(FullyAssociativeSet::new(mem / Self::MEM_PER_BLOCK)),
            _marker1: std::marker::PhantomData,
            _marker2: std::marker::PhantomData,
            _marker3: std::marker::PhantomData,
        }
    }

    fn count(&self) -> usize {
        1
    }

    fn set(&self, _page: u64) -> &Self::IMS {
        &self.set
    }

    fn data_mem(&self) -> usize {
        self.set.read(|s| s.blocks.len()) * Block::LEN
    }

    fn meta_mem(&self) -> usize {
        (self.set.read(|s| s.blocks.len()) * FullyAssociativeSet::<L, R, Block>::META_MEM_PER_BLOCK)
            + std::mem::size_of::<Self>()
    }
}
