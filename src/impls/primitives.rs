use traits::{ByteEncodable, ByteDecodable};
use errors::{ByteVecError, BVExpectedSize};
use std::mem::transmute;
use {BVEncodeResult, BVDecodeResult, BVSize};
use std::mem::size_of;

macro_rules! impl_integrals {
    {$($t:ty : $size:expr),*} => {
        $(
            impl ByteEncodable for $t {
                fn get_size<Size>(&self) -> Option<Size> where Size: BVSize + ByteEncodable {
                    Some(Size::from_usize($size))
                }

                fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>>
                    where Size: BVSize + ByteEncodable
                {
                    unsafe {
                        let bytes: [u8; $size] = transmute(self.to_le());
                        Ok(bytes.to_vec())
                    }
                }
            }

            impl ByteDecodable for $t {
                fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<$t>
                    where Size: BVSize + ByteDecodable
                {
                    if bytes.len() == $size {
                        let mut t_bytes = [0u8; $size];
                        for (b, s) in (&mut t_bytes).into_iter().zip(bytes) {
                            *b = *s;
                        }
                        unsafe { Ok(<$t>::from_le(transmute(t_bytes))) }
                    } else {
                        Err(ByteVecError::BadSizeDecodeError {
                            expected: BVExpectedSize::EqualTo($size as usize),
                            actual: bytes.len()
                        })
                    }
                }
            }
        )*
    }
}

impl_integrals! {
    u8: 1,
    u16: 2,
    u32: 4,
    u64: 8,
    i8: 1,
    i16: 2,
    i32: 4,
    i64: 8
}

macro_rules! as_unsized_impl {
    {$($t:ty : $unsizd:ty),*} => {
        $(
            impl ByteEncodable for $t {
                fn get_size<Size>(&self) -> Option<Size>
                    where Size: BVSize + ByteEncodable
                {
                    Some(Size::from_usize(size_of::<$t>()))
                }

                fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>>
                    where Size: BVSize + ByteEncodable
                {
                    unsafe {
                        let unsigned: $unsizd = transmute(*self);
                        unsigned.encode::<Size>()
                    }
                }
            }

            impl ByteDecodable for $t {
                fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<$t>
                    where Size: BVSize + ByteDecodable
                {
                    let unsigned = try!(<$unsizd>::decode::<Size>(bytes));
                    unsafe { Ok(transmute(unsigned)) }
                }
            }
        )*
    }
}

as_unsized_impl! {
    f32: u32,
    f64: u64,
    char: u32
}

impl ByteEncodable for usize {
    fn get_size<Size>(&self) -> Option<Size>
        where Size: BVSize + ByteEncodable
    {
        Some(Size::from_usize(size_of::<usize>()))
    }

    fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>>
        where Size: BVSize + ByteEncodable
    {
        match size_of::<usize>() {
            2 => (*self as u16).encode::<Size>(),
            4 => (*self as u32).encode::<Size>(),
            8 => (*self as u64).encode::<Size>(),
            _ => panic!("unknown size for usize"),
        }
    }
}

impl ByteDecodable for usize {
    fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<usize>
        where Size: BVSize + ByteDecodable
    {
        Ok(match size_of::<usize>() {
            2 => try!(u16::decode::<Size>(bytes)).as_usize(),
            4 => try!(u32::decode::<Size>(bytes)).as_usize(),
            8 => try!(u64::decode::<Size>(bytes)).as_usize(),
            _ => panic!("unknown size for usize"),
        })
    }
}
