mod collections;
mod primitives;

pub trait BVSize: Sized {
    fn from_usize(val: usize) -> Self;
    fn as_usize(&self) -> usize;
    fn max_value() -> Self;
    fn checked_add(self, rhs: Self) -> Option<Self>;
    fn get_size_of() -> Self;
}

macro_rules! def_BVSize {
    ($($t:ty),*) => {
        $(
            impl BVSize for $t {
                fn from_usize(val: usize) -> $t {
                    val as $t
                }
                
                fn as_usize(&self) -> usize {
                    *self as usize
                }
                
                fn max_value() -> Self {
                    <$t>::max_value()
                }
                
                fn checked_add(self, rhs: Self) -> Option<$t> {
                    self.checked_add(rhs)
                }
                
                fn get_size_of() -> Self {
                    <$t>::from_usize(::std::mem::size_of::<$t>())
                }
            }
        )*
    }
}

def_BVSize!(u8, u16, u32, u64);