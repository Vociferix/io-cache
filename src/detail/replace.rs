use super::*;

const NULL: usize = std::usize::MAX;

pub trait Replace {
    const STATIC_META_MEM: usize;
    const META_MEM_PER_BLOCK: usize;

    fn new(count: usize) -> Self;
    fn replace(&mut self) -> usize;
    fn record_access(&mut self, block_idx: usize);
}

pub struct DMReplace {}

impl Replace for DMReplace {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = 0;

    fn new(_: usize) -> Self {
        Self {}
    }
    fn replace(&mut self) -> usize {
        0
    }
    fn record_access(&mut self, _: usize) {}
}

pub struct Random<Size: ConstUsize> {
    seed: usize,
    _phantom: std::marker::PhantomData<Size>,
}

impl<Size: ConstUsize> Replace for Random<Size> {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = 0;

    fn new(_: usize) -> Self {
        Self {
            seed: 0x981234,
            _phantom: std::marker::PhantomData,
        }
    }

    fn replace(&mut self) -> usize {
        let s = self.seed;
        if std::mem::size_of::<usize>() == std::mem::size_of::<u32>() {
            let s = s ^ (s << 13);
            let s = s ^ (s >> 17);
            let s = s ^ (s << 5);
            self.seed = s;
            s & (Size::VALUE - 1)
        } else if std::mem::size_of::<usize>() == std::mem::size_of::<u64>() {
            let s = s ^ (s << 13);
            let s = s ^ (s >> 7);
            let s = s ^ (s << 17);
            self.seed = s;
            s & (Size::VALUE - 1)
        } else {
            panic!("io_cache random replacement not supported on this architecture");
        }
    }

    fn record_access(&mut self, _: usize) {}
}

pub struct LRU<Data: Array<LRUMeta>> {
    data: Data,
    front: usize,
    back: usize,
}

#[derive(Default, Clone, Copy)]
pub struct LRUMeta {
    next: usize,
    prev: usize,
}

impl<Data: Array<LRUMeta>> Replace for LRU<Data> {
    const STATIC_META_MEM: usize =
        std::mem::size_of::<Self>() - (Self::META_MEM_PER_BLOCK * Data::LEN);
    const META_MEM_PER_BLOCK: usize = std::mem::size_of::<LRUMeta>();

    fn new(_: usize) -> Self {
        let mut meta = Data::new();
        for (idx, block) in meta.get_mut().iter_mut().enumerate() {
            if idx == 0 {
                block.prev = NULL;
            } else {
                block.prev = idx - 1;
            }
            if idx == (Data::LEN - 1) {
                block.next = NULL;
            } else {
                block.next = idx + 1;
            }
        }
        Self {
            data: meta,
            front: 0,
            back: Data::LEN - 1,
        }
    }

    fn replace(&mut self) -> usize {
        let ret = self.front;
        let next = self.data.get_ref()[ret].next;
        if next != NULL {
            self.front = next;
            let back = self.back;
            self.back = ret;
            {
                let data = self.data.get_mut();
                data[back].next = ret;
                let r = &mut data[ret];
                r.prev = back;
                r.next = NULL;
            }
        }
        ret
    }

    fn record_access(&mut self, block_idx: usize) {
        let back = self.back;
        if back != block_idx {
            let front = self.front;
            if front == block_idx {
                let next = self.data.get_ref()[front].next;
                self.front = next;
                self.back = front;
                {
                    let data = self.data.get_mut();
                    data[next].prev = NULL;
                    data[back].next = front;
                    let b = &mut data[front];
                    b.prev = back;
                    b.next = NULL;
                }
            } else {
                self.back = block_idx;
                let (next, prev) = {
                    let data = &self.data.get_ref()[block_idx];
                    (data.next, data.prev)
                };
                {
                    let data = self.data.get_mut();
                    data[next].prev = prev;
                    data[prev].next = next;
                    data[block_idx].prev = back;
                    data[block_idx].next = NULL;
                    data[back].next = block_idx;
                }
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct HeapElem<T> {
    data: T,
    pos: usize,
}

struct Heap<T: Clone + Default, Data: Array<HeapElem<T>>, Queue: Array<usize>> {
    data: Data,
    queue: Queue,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Default, Data: Array<HeapElem<T>>, Queue: Array<usize>> Heap<T, Data, Queue> {
    /*
    const STATIC_META_MEM: usize =
        std::mem::size_of::<Self>() - (Self::META_MEM_PER_BLOCK * Data::LEN);
    */
    const META_MEM_PER_BLOCK: usize =
        std::mem::size_of::<HeapElem<T>>() + std::mem::size_of::<usize>();

    fn new<F: FnMut(usize, &mut T)>(mut f: F) -> Self {
        let mut data = Data::new();
        for (idx, elem) in data.get_mut().iter_mut().enumerate() {
            f(idx, &mut elem.data);
        }
        let mut queue = Queue::new();
        for (idx, elem) in queue.get_mut().iter_mut().enumerate() {
            *elem = idx;
        }
        Self {
            data,
            queue,
            _phantom: std::marker::PhantomData,
        }
    }

    /*
    fn get_ref(&self, idx: usize) -> &T {
        &self.data.get_ref()[idx].data
    }
    */

    /*
    fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self.data.get_mut()[idx].data
    }
    */

    fn top(&self) -> usize {
        self.queue.get_ref()[0]
    }

    fn update<F: FnOnce(&mut T), Cmp: Fn(&T, &T) -> i8>(&mut self, idx: usize, f: F, cmp: Cmp) {
        let pos = {
            let b = &mut self.data.get_mut()[idx];
            f(&mut b.data);
            b.pos
        };
        let left = ((pos + 1) * 2) - 1;
        let right = (pos + 1) * 2;
        if left < Queue::LEN {
            if right < Queue::LEN {
                let (left_idx, right_idx) = {
                    let queue = self.queue.get_ref();
                    (queue[left], queue[right])
                };
                self.send_down(idx, pos, left_idx, left, right_idx, right, cmp);
            } else {
                let left_idx = self.queue.get_ref()[left];
                let flag = {
                    let data = self.data.get_ref();
                    cmp(&data[idx].data, &data[left_idx].data) >= 0
                };
                if flag {
                    let queue = self.queue.get_mut();
                    queue[pos] = queue[left];
                    queue[left] = idx;
                }
            }
        }
    }

    fn send_down<Cmp: Fn(&T, &T) -> i8>(
        &mut self,
        idx: usize,
        mut pos: usize,
        left_idx: usize,
        mut left_pos: usize,
        right_idx: usize,
        mut right_pos: usize,
        cmp: Cmp,
    ) {
        loop {
            let flag = {
                let data = self.data.get_ref();
                if cmp(&data[idx].data, &data[left_idx].data) >= 0 {
                    -1
                } else if cmp(&data[idx].data, &data[right_idx].data) >= 0 {
                    1
                } else {
                    0
                }
            };

            if flag < 0 {
                let queue = self.queue.get_mut();
                queue[pos] = queue[left_pos];
                queue[left_pos] = idx;
                pos = left_pos;
            } else if flag > 0 {
                let queue = self.queue.get_mut();
                queue[pos] = queue[right_pos];
                queue[right_pos] = idx;
                pos = right_pos;
            } else {
                return;
            }

            left_pos = ((pos + 1) * 2) - 1;
            right_pos = (pos + 1) * 2;

            if left_pos >= Queue::LEN {
                return;
            } else if right_pos >= Queue::LEN {
                let flag = {
                    let data = self.data.get_ref();
                    cmp(&data[idx].data, &data[left_idx].data) >= 0
                };
                if flag {
                    let queue = self.queue.get_mut();
                    queue[pos] = queue[left_pos];
                    queue[left_pos] = idx;
                }
                return;
            }
        }
    }
}

pub struct LFU<Data: Array<HeapElem<u64>>, Queue: Array<usize>> {
    heap: Heap<u64, Data, Queue>,
}

fn lfu_cmp(l: &u64, r: &u64) -> i8 {
    if *l < *r {
        -1
    } else {
        (*l > *r) as i8
    }
}

impl<Data: Array<HeapElem<u64>>, Queue: Array<usize>> Replace for LFU<Data, Queue> {
    const STATIC_META_MEM: usize =
        std::mem::size_of::<Self>() - (Self::META_MEM_PER_BLOCK * Data::LEN);
    const META_MEM_PER_BLOCK: usize = Heap::<u64, Data, Queue>::META_MEM_PER_BLOCK;

    fn new(_: usize) -> Self {
        Self {
            heap: Heap::new(|_, _| {}),
        }
    }

    fn replace(&mut self) -> usize {
        let ret = self.heap.top();
        self.heap.update(ret, |t| *t = 0, lfu_cmp);
        ret
    }

    fn record_access(&mut self, idx: usize) {
        self.heap.update(idx, |t| *t += 1, lfu_cmp);
    }
}

pub struct LRFU<Data: Array<HeapElem<LRFUMeta>>, Queue: Array<usize>, Rate: ConstF32> {
    heap: Heap<LRFUMeta, Data, Queue>,
    now: u64,
    _phantom: std::marker::PhantomData<Rate>,
}

#[derive(Default, Clone, Copy)]
pub struct LRFUMeta {
    crf: f32,
    time: u64,
}

fn crf_calc<Rate: ConstF32>(b: &LRFUMeta, now: u64) -> f32 {
    Rate::VALUE.powi((now - b.time) as i32) * b.crf
}

impl<Data: Array<HeapElem<LRFUMeta>>, Queue: Array<usize>, Rate: ConstF32> Replace
    for LRFU<Data, Queue, Rate>
{
    const STATIC_META_MEM: usize =
        std::mem::size_of::<Self>() - (Self::META_MEM_PER_BLOCK * Data::LEN);
    const META_MEM_PER_BLOCK: usize = Heap::<LRFUMeta, Data, Queue>::META_MEM_PER_BLOCK;

    fn new(_: usize) -> Self {
        Self {
            heap: Heap::new(|_, b: &mut LRFUMeta| {
                b.crf = 1.0;
            }),
            now: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn replace(&mut self) -> usize {
        let ret = self.heap.top();
        let now = self.now;
        self.heap.update(
            ret,
            |b| {
                b.crf = 1.0;
                b.time = 0;
            },
            |_, r| {
                let crf = crf_calc::<Rate>(r, now);
                if crf < 1.0 {
                    1
                } else if crf > 1.0 {
                    -1
                } else {
                    0
                }
            },
        );
        ret
    }

    fn record_access(&mut self, idx: usize) {
        self.now += 1;
        let now = self.now;
        self.heap.update(
            idx,
            |b| {
                b.crf = crf_calc::<Rate>(b, now) + 1.0;
                b.time = now;
            },
            |l, r| {
                let r_crf = crf_calc::<Rate>(r, now);
                if r_crf < l.crf {
                    1
                } else if r_crf > l.crf {
                    -1
                } else {
                    0
                }
            },
        );
    }
}

pub struct FIFO<Size: ConstUsize> {
    curr: usize,
    _phantom: std::marker::PhantomData<Size>,
}

impl<Size: ConstUsize> Replace for FIFO<Size> {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = 0;

    fn new(_: usize) -> Self {
        Self {
            curr: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn replace(&mut self) -> usize {
        let ret = self.curr;
        self.curr = (self.curr + 1) & (Size::VALUE - 1);
        ret
    }

    fn record_access(&mut self, _: usize) {}
}

pub struct FARandom {
    seed: usize,
    count: usize,
}

impl Replace for FARandom {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = 0;

    fn new(count: usize) -> Self {
        Self {
            seed: 0x981234,
            count,
        }
    }

    fn replace(&mut self) -> usize {
        let s = self.seed;
        if std::mem::size_of::<usize>() == std::mem::size_of::<u32>() {
            let s = s ^ (s << 13);
            let s = s ^ (s >> 17);
            let s = s ^ (s << 5);
            self.seed = s;
            s % self.count
        } else if std::mem::size_of::<usize>() == std::mem::size_of::<u64>() {
            let s = s ^ (s << 13);
            let s = s ^ (s >> 7);
            let s = s ^ (s << 17);
            self.seed = s;
            s % self.count
        } else {
            panic!("io_cache random replacement not supported on this architecture");
        }
    }

    fn record_access(&mut self, _: usize) {}
}

pub struct FALRU {
    list: Vec<LRUMeta>,
    front: usize,
    back: usize,
}

impl Replace for FALRU {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = std::mem::size_of::<LRUMeta>();

    fn new(count: usize) -> Self {
        let cm1 = count - 1;
        let mut meta = vec![LRUMeta::default(); count];
        for (idx, block) in meta.iter_mut().enumerate() {
            if idx == 0 {
                block.prev = NULL;
            } else {
                block.prev = idx - 1;
            }
            if idx == cm1 {
                block.next = NULL;
            } else {
                block.next = idx + 1;
            }
        }
        Self {
            list: meta,
            front: 0,
            back: cm1,
        }
    }

    fn replace(&mut self) -> usize {
        let ret = self.front;
        let next = self.list[ret].next;
        if next != NULL {
            self.front = next;
            let back = self.back;
            self.back = ret;
            {
                self.list[back].next = ret;
                let r = &mut self.list[ret];
                r.prev = back;
                r.next = NULL;
            }
        }
        ret
    }

    fn record_access(&mut self, block_idx: usize) {
        let back = self.back;
        if back != block_idx {
            let front = self.front;
            if front == block_idx {
                let next = self.list[front].next;
                self.front = next;
                self.back = front;
                {
                    self.list[next].prev = NULL;
                    self.list[back].next = front;
                    let b = &mut self.list[front];
                    b.prev = back;
                    b.next = NULL;
                }
            } else {
                self.back = block_idx;
                let (next, prev) = {
                    let data = &self.list[block_idx];
                    (data.next, data.prev)
                };
                {
                    self.list[next].prev = prev;
                    self.list[prev].next = next;
                    self.list[block_idx].prev = back;
                    self.list[block_idx].next = NULL;
                    self.list[back].next = block_idx;
                }
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
struct FAHeapElem<T> {
    data: T,
    pos: usize,
}

struct FAHeap<T: Default + Clone> {
    data: Vec<FAHeapElem<T>>,
    queue: Vec<usize>,
}

impl<T: Default + Clone> FAHeap<T> {
    // const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize =
        std::mem::size_of::<HeapElem<T>>() + std::mem::size_of::<usize>();

    fn new<F: FnMut(usize, &mut T)>(count: usize, mut f: F) -> Self {
        let mut data: Vec<FAHeapElem<T>> = vec![FAHeapElem::default(); count];
        for (idx, elem) in data.iter_mut().enumerate() {
            f(idx, &mut elem.data);
        }
        let mut queue = vec![0; count];
        for (idx, elem) in queue.iter_mut().enumerate() {
            *elem = idx;
        }
        Self { data, queue }
    }

    /*
    fn get_ref(&self, idx: usize) -> &T {
        &self.data[idx].data
    }
    */

    /*
    fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self.data[idx].data
    }
    */

    fn top(&self) -> usize {
        self.queue[0]
    }

    fn update<F: FnOnce(&mut T), Cmp: Fn(&T, &T) -> i8>(&mut self, idx: usize, f: F, cmp: Cmp) {
        let pos = {
            let b = &mut self.data[idx];
            f(&mut b.data);
            b.pos
        };
        let left = ((pos + 1) * 2) - 1;
        let right = (pos + 1) * 2;
        if left < self.queue.len() {
            if right < self.queue.len() {
                let (left_idx, right_idx) = { (self.queue[left], self.queue[right]) };
                self.send_down(idx, pos, left_idx, left, right_idx, right, cmp);
            } else {
                let left_idx = self.queue[left];
                let flag = { cmp(&self.data[idx].data, &self.data[left_idx].data) >= 0 };
                if flag {
                    self.queue[pos] = self.queue[left];
                    self.queue[left] = idx;
                }
            }
        }
    }

    fn send_down<Cmp: Fn(&T, &T) -> i8>(
        &mut self,
        idx: usize,
        mut pos: usize,
        left_idx: usize,
        mut left_pos: usize,
        right_idx: usize,
        mut right_pos: usize,
        cmp: Cmp,
    ) {
        loop {
            let flag = {
                if cmp(&self.data[idx].data, &self.data[left_idx].data) >= 0 {
                    -1
                } else if cmp(&self.data[idx].data, &self.data[right_idx].data) >= 0 {
                    1
                } else {
                    0
                }
            };

            if flag < 0 {
                self.queue[pos] = self.queue[left_pos];
                self.queue[left_pos] = idx;
                pos = left_pos;
            } else if flag > 0 {
                self.queue[pos] = self.queue[right_pos];
                self.queue[right_pos] = idx;
                pos = right_pos;
            } else {
                return;
            }

            left_pos = ((pos + 1) * 2) - 1;
            right_pos = (pos + 1) * 2;

            if left_pos >= self.queue.len() {
                return;
            } else if right_pos >= self.queue.len() {
                let flag = { cmp(&self.data[idx].data, &self.data[left_idx].data) >= 0 };
                if flag {
                    self.queue[pos] = self.queue[left_pos];
                    self.queue[left_pos] = idx;
                }
                return;
            }
        }
    }
}

pub struct FALFU {
    heap: FAHeap<u64>,
}

impl Replace for FALFU {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = FAHeap::<u64>::META_MEM_PER_BLOCK;

    fn new(count: usize) -> Self {
        Self {
            heap: FAHeap::new(count, |_, _| {}),
        }
    }

    fn replace(&mut self) -> usize {
        let ret = self.heap.top();
        self.heap.update(ret, |t| *t = 0, lfu_cmp);
        ret
    }

    fn record_access(&mut self, idx: usize) {
        self.heap.update(idx, |t| *t += 1, lfu_cmp);
    }
}

pub struct FALRFU<Rate: ConstF32> {
    heap: FAHeap<LRFUMeta>,
    now: u64,
    _marker: std::marker::PhantomData<Rate>,
}

impl<Rate: ConstF32> Replace for FALRFU<Rate> {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = FAHeap::<u64>::META_MEM_PER_BLOCK;

    fn new(count: usize) -> Self {
        Self {
            heap: FAHeap::new(count, |_, b: &mut LRFUMeta| {
                b.crf = 1.0;
            }),
            now: 0,
            _marker: std::marker::PhantomData,
        }
    }

    fn replace(&mut self) -> usize {
        let ret = self.heap.top();
        let now = self.now;
        self.heap.update(
            ret,
            |b| {
                b.crf = 1.0;
                b.time = 0;
            },
            |_, r| {
                let crf = crf_calc::<Rate>(r, now);
                if crf < 1.0 {
                    1
                } else if crf > 1.0 {
                    -1
                } else {
                    0
                }
            },
        );
        ret
    }

    fn record_access(&mut self, idx: usize) {
        self.now += 1;
        let now = self.now;
        self.heap.update(
            idx,
            |b| {
                b.crf = crf_calc::<Rate>(b, now) + 1.0;
                b.time = now;
            },
            |l, r| {
                let r_crf = crf_calc::<Rate>(r, now);
                if r_crf < l.crf {
                    1
                } else if r_crf > l.crf {
                    -1
                } else {
                    0
                }
            },
        );
    }
}

pub struct FAFIFO {
    curr: usize,
    count: usize,
}

impl Replace for FAFIFO {
    const STATIC_META_MEM: usize = std::mem::size_of::<Self>();
    const META_MEM_PER_BLOCK: usize = 0;

    fn new(count: usize) -> Self {
        Self { curr: 0, count }
    }

    fn replace(&mut self) -> usize {
        let ret = self.curr;
        self.curr = (self.curr + 1) % self.count;
        ret
    }

    fn record_access(&mut self, _: usize) {}
}
