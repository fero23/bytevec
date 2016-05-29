use traits::{ByteEncodable, ByteDecodable};
use errors::{ByteVecError, BVWantedSize};
use {BVEncodeResult, BVDecodeResult};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

macro_rules! validate_collection {
    ($byte_vec:ident, $index:ident, $len:ident, $size_vec:ident, $ret:expr) => {{
        if $byte_vec.len() >= 4 {
            $len = try!(u32::decode(&$byte_vec[..4])) as usize;
            $index = 4;
            let sizes_len = $len * 4;
            if $byte_vec[4..].len() >= sizes_len {
                $size_vec = Vec::new();
                for _ in 0..$len {
                    $size_vec.push(try!(u32::decode(&$byte_vec[$index..$index + 4])));
                    $index += 4;
                }
                let body_size = $size_vec.iter().fold(0, |acc, &size| acc + size);
                if body_size as usize == $byte_vec[4 + sizes_len..].len() {
                    $ret
                } else {
                    Err(ByteVecError::BadSizeDecodeError {
                        wanted: BVWantedSize::EqualTo(4 + sizes_len as u32 + body_size),
                        actual: $byte_vec.len() as u32
                    })
                }
            }
            else {
                Err(ByteVecError::BadSizeDecodeError {
                    wanted: BVWantedSize::MoreThan(4 + sizes_len as u32),
                    actual: $byte_vec.len() as u32
                })
            }
        } else {
            Err(ByteVecError::BadSizeDecodeError {
                wanted: BVWantedSize::MoreThan(4),
                actual: $byte_vec.len() as u32
            })
        }
    }};
}

impl ByteEncodable for str {
    fn get_size(&self) -> Option<u32> {
        if self.len() <= u32::max_value() as usize {
            Some(self.len() as u32)
        } else {
            None
        }
    }

    fn encode(&self) -> BVEncodeResult<Vec<u8>> {
        if self.get_size().is_some() {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&self.as_bytes().to_vec());
            Ok(bytes)
        } else {
            Err(ByteVecError::OverflowError)
        }
    }
}

impl<'a> ByteEncodable for &'a str {
    fn get_size(&self) -> Option<u32> {
        (**self).get_size()
    }

    fn encode(&self) -> BVEncodeResult<Vec<u8>> {
        (**self).encode()
    }
}

impl ByteEncodable for String {
    fn get_size(&self) -> Option<u32> {
        (**self).get_size()
    }

    fn encode(&self) -> BVEncodeResult<Vec<u8>> {
        (**self).encode()
    }
}

impl ByteDecodable for String {
    fn decode(bytes: &[u8]) -> BVDecodeResult<String> {
        Ok(try!(::std::str::from_utf8(bytes)).to_string())
    }
}

macro_rules! collection_encode_impl {
    () => {
        fn get_size(&self) -> Option<u32> {
            self.iter()
                .fold(Some(0), |acc, elem| {
                    acc.and_then(|acc: u32| {
                        (&elem).get_size()
                            .and_then(|size| {
                                acc.checked_add(size).and_then(|acc_size| acc_size.checked_add(4))
                            })
                    })
                })
                .and_then(|total: u32| total.checked_add(4))
        }

        fn encode(&self) -> BVEncodeResult<Vec<u8>> {
            if self.get_size().is_some() {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&try!((self.len() as u32).encode()));
                for elem in self {
                    bytes.extend_from_slice(&try!((&elem).get_size().unwrap().encode()));
                }
                for elem in self {
                    bytes.extend_from_slice(&try!((&elem).encode()));
                }
                Ok(bytes)
            } else {
                Err(ByteVecError::OverflowError)
            }
        }
    }
}

impl<T> ByteEncodable for Vec<T>
    where T: ByteEncodable
{
    collection_encode_impl!();
}

impl<T> ByteDecodable for Vec<T>
    where T: ByteDecodable
{
    fn decode(bytes: &[u8]) -> BVDecodeResult<Vec<T>> {
        let len;
        let mut index;
        let mut sizes;
        validate_collection!(bytes, index, len, sizes, {
            let mut vec = Vec::with_capacity(len);
            for size in sizes.into_iter() {
                vec.push(try!(T::decode(&bytes[index..index + size as usize])));
                index += size as usize;
            }
            Ok(vec)
        })
    }
}

impl<T> ByteEncodable for [T]
    where T: ByteEncodable
{
    collection_encode_impl!();
}

impl<T> ByteEncodable for HashSet<T>
    where T: ByteEncodable + Eq + Hash
{
    collection_encode_impl!();
}

impl<T> ByteDecodable for HashSet<T>
    where T: ByteDecodable + Eq + Hash
{
    fn decode(bytes: &[u8]) -> BVDecodeResult<HashSet<T>> {
        let len;
        let mut index;
        let mut sizes;
        validate_collection!(bytes, index, len, sizes, {
            let mut set = HashSet::with_capacity(len);
            for size in sizes.into_iter() {
                set.insert(try!(T::decode(&bytes[index..index + size as usize])));
                index += size as usize;
            }
            Ok(set)
        })
    }
}

impl<K, V> ByteEncodable for HashMap<K, V>
    where K: ByteEncodable + Hash + Eq,
          V: ByteEncodable
{
    collection_encode_impl!();
}

impl<K, V> ByteDecodable for HashMap<K, V>
    where K: ByteDecodable + Hash + Eq,
          V: ByteDecodable
{
    fn decode(bytes: &[u8]) -> BVDecodeResult<HashMap<K, V>> {
        let len;
        let mut index;
        let mut sizes;
        validate_collection!(bytes, index, len, sizes, {
            let mut map = HashMap::with_capacity(len);
            for size in sizes.into_iter() {
                let (key, value) = try!(<(K, V)>::decode(&bytes[index..index + size as usize]));
                map.insert(key, value);
                index += size as usize;
            }
            Ok(map)
        })
    }
}

macro_rules! tuple_impls {
    ($t:ident: $elem:ident) => {
        impl<$t> ByteEncodable for ($t,)
            where $t: ByteEncodable
        {
            fn get_size(&self) -> Option<u32> {
                (&(&self.0)).get_size()
            }

            fn encode(&self) -> BVEncodeResult<Vec<u8>> {
                (&(&self.0)).encode()
            }
        }

        impl<'a, $t> ByteEncodable for &'a (&'a $t,)
            where $t: ByteEncodable
        {
            fn get_size(&self) -> Option<u32> {
                self.0.get_size().and_then(|elem_size| elem_size.checked_add(4))
            }

            fn encode(&self) -> BVEncodeResult<Vec<u8>> {
                if self.get_size().is_some() {
                    let mut bytes = Vec::new();
                    bytes.extend_from_slice(&try!(self.0.get_size().unwrap().encode()));
                    bytes.extend_from_slice(&try!(self.0.encode()));
                    Ok(bytes)
                } else {
                    Err(ByteVecError::OverflowError)
                }
            }
        }

        impl<$t> ByteDecodable for ($t,)
            where $t: ByteDecodable
        {
            fn decode(bytes: &[u8]) -> BVDecodeResult<($t,)> {
                let size;

                if bytes.len() >= 4 {
                    size = try!(u32::decode(&bytes[..4]));
                }
                else {
                    return Err(ByteVecError::BadSizeDecodeError {
                        wanted: BVWantedSize::MoreThan(4),
                        actual: bytes.len() as u32
                    });
                }
                if size as usize == bytes[4..].len() {
                    Ok((try!($t::decode(&bytes[4..])),))
                } else {
                    Err(ByteVecError::BadSizeDecodeError {
                        wanted: BVWantedSize::EqualTo(4 + size),
                        actual: bytes.len() as u32
                    })
                }
            }
        }
    };

    // Lots of doubled code to implement recursion by dropping the first element each iteration,
    // so the first operation has to be done outside the macro loop, repeating the code
    ($t:ident: $elem:ident, $($_t:ident: $_elem:ident),*) => {
        impl<$t, $($_t),*> ByteEncodable for ($t, $($_t),*)
            where $t: ByteEncodable, $($_t: ByteEncodable),*
        {
            fn get_size(&self) -> Option<u32> {
                let &(ref $elem, $(ref $_elem),*) = self;
                (&($elem, $($_elem),*)).get_size()
            }

            fn encode(&self) -> BVEncodeResult<Vec<u8>> {
                let &(ref $elem, $(ref $_elem),*) = self;
                (&($elem, $($_elem),*)).encode()
            }
        }

        impl<'a, $t, $($_t),*> ByteEncodable for &'a (&'a $t, $(&'a $_t),*)
            where $t: ByteEncodable, $($_t: ByteEncodable),*
        {
            fn get_size(&self) -> Option<u32> {
                let &&($elem, $($_elem),*) = self;
                let mut size = Some(0);

                size = size.and_then(|size: u32|
                    $elem.get_size().and_then(|elem_size|
                        size.checked_add(elem_size).and_then(
                            |acc_size| acc_size.checked_add(4)
                        )
                    )
                );
                $(
                    size = size.and_then(|size: u32|
                        $_elem.get_size().and_then(|elem_size|
                            size.checked_add(elem_size).and_then(
                                |acc_size| acc_size.checked_add(4)
                            )
                        )
                    );
                )*
                size
            }

            fn encode(&self) -> BVEncodeResult<Vec<u8>> {
                if self.get_size().is_some() {
                    let &&($elem, $($_elem),*) = self;
                    let mut bytes = Vec::new();
                    bytes.extend_from_slice(&try!($elem.get_size().unwrap().encode()));
                    $(
                        bytes.extend_from_slice(&try!($_elem.get_size().unwrap().encode()));
                    )*
                    bytes.extend_from_slice(&try!($elem.encode()));
                    $(
                        bytes.extend_from_slice(&try!($_elem.encode()));
                    )*
                    Ok(bytes)
                } else {
                    Err(ByteVecError::OverflowError)
                }
            }
        }

        #[allow(unused_assignments)]
        impl<$t, $($_t),*> ByteDecodable for ($t, $($_t),*)
            where $t: ByteDecodable, $($_t: ByteDecodable),*
        {
            fn decode(bytes: &[u8]) -> BVDecodeResult<($t, $($_t),*)> {
                let mut index = 0;
                let mut sizes = ::std::collections::HashMap::new();

                if bytes.len() >= 4 {
                    sizes.insert(stringify!($elem),
                        try!(u32::decode(&bytes[..4])));
                    index += 4;
                }
                else {
                    return Err(ByteVecError::BadSizeDecodeError {
                        wanted: BVWantedSize::MoreThan(4 + index as u32),
                        actual: bytes.len() as u32
                    });
                }
                $(
                    if bytes[index..].len() >= 4 {
                        sizes.insert(stringify!($_elem),
                            try!(u32::decode(&bytes[index..index + 4])));
                        index += 4;
                    }
                    else {
                        return Err(ByteVecError::BadSizeDecodeError {
                            wanted: BVWantedSize::MoreThan(4 + index as u32),
                            actual: bytes.len() as u32
                        });
                    }
                )*

                let body_size = sizes.values().fold(0, |acc, &size| acc + size);
                if body_size as usize == bytes[index..].len() {
                    Ok((
                        {
                            let elem = try!($t::decode(
                                &bytes[index..index + sizes[stringify!($elem)] as usize]));
                            index += sizes[stringify!($elem)] as usize;
                            elem
                        },
                        $({
                            let elem = try!($_t::decode(
                                &bytes[index..index + sizes[stringify!($_elem)] as usize]));
                            index += sizes[stringify!($_elem)] as usize;
                            elem
                        }),*
                    ))
                } else {
                    Err(ByteVecError::BadSizeDecodeError {
                        wanted: BVWantedSize::EqualTo(
                            4 * sizes.len() as u32 + body_size),
                        actual: bytes.len() as u32
                    })
                }
            }
        }

        tuple_impls!($($_t: $_elem),*);
    }
}

tuple_impls! {
    A: a,
    B: b,
    C: c,
    D: d,
    E: e,
    F: f,
    G: g,
    H: h,
    I: i,
    J: j,
    K: k,
    L: l
}

impl ByteEncodable for () {
    fn get_size(&self) -> Option<u32> {
        Some(0)
    }

    fn encode(&self) -> BVEncodeResult<Vec<u8>> {
        // Send only size of 0
        0u32.encode()
    }
}

impl ByteDecodable for () {
    fn decode(_: &[u8]) -> BVDecodeResult<()> {
        Ok(())
    }
}
