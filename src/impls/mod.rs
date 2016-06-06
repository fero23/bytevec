mod collections;
mod primitives;

/// Represents the generic integral type of the structure size indicators
pub trait BVSize: Sized {
    /// Returns a `Self` value casted from an `usize` value
    fn from_usize(val: usize) -> Self;
    /// Returns an `usize` value casted from a `Self` value
    fn as_usize(&self) -> usize;
    /// Returns the max value for `Self`
    fn max_value() -> Self;
    /// Calls the `checked_add` method of `self` passing `rhs`
    fn checked_add(self, rhs: Self) -> Option<Self>;
    /// Returns the returned value of [`std::mem::size_of`][1] for `Self`
    /// [1]: http://doc.rust-lang.org/stable/std/mem/fn.size_of.html
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