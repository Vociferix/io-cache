mod consts;
mod inner_mut;
mod lookup;
mod replace;
mod set;
mod io;

pub use consts::*;
pub use inner_mut::*;
pub use lookup::*;
pub use replace::*;
pub use set::*;
pub use io::*;

/*
const fn hash32(x: u32) -> u32 {
    let x = x + 1;
    let x = ((x >> 16) ^ x).wrapping_mul(0x45D9F3B);
    let x = ((x >> 16) ^ x).wrapping_mul(0x45D9F3B);
    (x >> 16) ^ x
}
*/

const fn hash64(x: u64) -> u64 {
    let x = x + 1;
    let x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    let x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}
