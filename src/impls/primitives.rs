use traits::{ByteEncodable, ByteDecodable};
use errors::{ByteVecError, BVWantedSize};
use std::mem::transmute;
use {BVEncodeResult, BVDecodeResult};
use std::mem::size_of;

macro_rules! impl_integrals {
    {$($t:ty : $size:expr),*} => {
        $(
            impl ByteEncodable for $t {
                fn get_size(&self) -> Option<u32> {
                    Some($size)
                }

                fn encode(&self) -> BVEncodeResult<Vec<u8>> {
                    unsafe {
                        let bytes: [u8; $size] = transmute(self.to_be());
                        Ok(bytes.to_vec())
                    }
                }
            }

            impl ByteDecodable for $t {
                fn decode(bytes: &[u8]) -> BVDecodeResult<$t> {
                    if bytes.len() == $size {
                        let mut t_bytes = [0u8; $size];
                        for (b, s) in (&mut t_bytes).into_iter().zip(bytes) {
                            *b = *s;
                        }
                        unsafe { Ok(<$t>::from_be(transmute(t_bytes))) }
                    } else {
                        Err(ByteVecError::BadSizeDecodeError {
                            wanted: BVWantedSize::EqualTo($size),
                            actual: bytes.len() as u32
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
                fn get_size(&self) -> Option<u32> {
                    Some(size_of::<$t>() as u32)
                }

                fn encode(&self) -> BVEncodeResult<Vec<u8>> {
                    unsafe {
                        let unsigned: $unsizd = transmute(*self);
                        unsigned.encode()
                    }
                }
            }

            impl ByteDecodable for $t {
                fn decode(bytes: &[u8]) -> BVDecodeResult<$t> {
                    let unsigned = try!(<$unsizd>::decode(bytes));
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
    fn get_size(&self) -> Option<u32> {
        Some(size_of::<usize>() as u32)
    }

    fn encode(&self) -> BVEncodeResult<Vec<u8>> {
        match size_of::<usize>() {
            2 => (*self as u16).encode(),
            4 => (*self as u32).encode(),
            8 => (*self as u64).encode(),
            _ => panic!("unknown size for usize"),
        }
    }
}

impl ByteDecodable for usize {
    fn decode(bytes: &[u8]) -> BVDecodeResult<usize> {
        Ok(match size_of::<usize>() {
            2 => try!(u16::decode(bytes)) as usize,
            4 => try!(u32::decode(bytes)) as usize,
            8 => try!(u64::decode(bytes)) as usize,
            _ => panic!("unknown size for usize"),
        })
    }
}
