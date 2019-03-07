pub trait InnerMut<T> {
    type InnerType;

    fn new(inner: T) -> Self;
    fn into_inner(self) -> T;
    fn read<Ret, F: FnOnce(&T) -> Ret>(&self, f: F) -> Ret;
    fn write<Ret, F: FnOnce(&mut T) -> Ret>(&self, f: F) -> Ret;
}

pub struct Mutex<T> {
    mutex: std::sync::Mutex<T>,
}

impl<T> InnerMut<T> for Mutex<T> {
    type InnerType = T;

    fn new(inner: T) -> Self {
        Self {
            mutex: std::sync::Mutex::new(inner),
        }
    }

    fn into_inner(self) -> T {
        match self.mutex.into_inner() {
            Ok(inner) => inner,
            Err(e) => e.into_inner(),
        }
    }

    fn read<Ret, F: FnOnce(&T) -> Ret>(&self, f: F) -> Ret {
        f(&*self.mutex.lock().unwrap())
    }

    fn write<Ret, F: FnOnce(&mut T) -> Ret>(&self, f: F) -> Ret {
        f(&mut *self.mutex.lock().unwrap())
    }
}

pub struct RwLock<T> {
    rwlock: std::sync::RwLock<T>,
}

impl<T> InnerMut<T> for RwLock<T> {
    type InnerType = T;

    fn new(inner: T) -> Self {
        Self {
            rwlock: std::sync::RwLock::new(inner),
        }
    }

    fn into_inner(self) -> T {
        match self.rwlock.into_inner() {
            Ok(inner) => inner,
            Err(e) => e.into_inner(),
        }
    }

    fn read<Ret, F: FnOnce(&T) -> Ret>(&self, f: F) -> Ret {
        f(&*self.rwlock.read().unwrap())
    }

    fn write<Ret, F: FnOnce(&mut T) -> Ret>(&self, f: F) -> Ret {
        f(&mut *self.rwlock.write().unwrap())
    }
}

pub struct RefCell<T> {
    cell: std::cell::RefCell<T>,
}

impl<T> InnerMut<T> for RefCell<T> {
    type InnerType = T;

    fn new(inner: T) -> Self {
        Self {
            cell: std::cell::RefCell::new(inner),
        }
    }

    fn into_inner(self) -> T {
        self.cell.into_inner()
    }

    fn read<Ret, F: FnOnce(&T) -> Ret>(&self, f: F) -> Ret {
        f(&*self.cell.borrow())
    }

    fn write<Ret, F: FnOnce(&mut T) -> Ret>(&self, f: F) -> Ret {
        f(&mut *self.cell.borrow_mut())
    }
}
