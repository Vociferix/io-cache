pub trait Bool {
    const VALUE: bool;
}

pub struct True;
impl Bool for True {
    const VALUE: bool = true;
}

pub struct False;
impl Bool for False {
    const VALUE: bool = false;
}

pub trait ConstUsize {
    const VALUE: usize;
}

pub trait ConstF32 {
    const VALUE: f32;
}

pub trait Array<T: Sized + Default + Clone>: Clone + Default {
    const LEN: usize;
    fn new() -> Self;
    fn new_with(val: T) -> Self;
    fn get_ref(&self) -> &[T];
    fn get_mut(&mut self) -> &mut [T];
}
