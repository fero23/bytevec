use traits::{ByteEncodable, ByteDecodable};
use errors::{ByteVecError, BVExpectedSize};
use {BVEncodeResult, BVDecodeResult, BVSize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

macro_rules! validate_collection {
    ($byte_vec:ident, $index:ident, $len:ident, $size_vec:ident, $ret:expr) => {{
        if $byte_vec.len() >= Size::get_size_of().as_usize() {
            $len = try!(Size::decode::<Size>(
                &$byte_vec[..Size::get_size_of().as_usize()])).as_usize();
            $index = Size::get_size_of().as_usize();
            let sizes_len = $len * Size::get_size_of().as_usize();
            if $byte_vec[Size::get_size_of().as_usize()..].len() >= sizes_len {
                $size_vec = Vec::new();
                for _ in 0..$len {
                    $size_vec.push(try!(Size::decode::<Size>(
                        &$byte_vec[$index..$index + Size::get_size_of().as_usize()])));
                    $index += Size::get_size_of().as_usize();
                }
                let body_size = $size_vec.iter().fold(0, |acc, ref size| acc + size.as_usize());
                if body_size == $byte_vec[Size::get_size_of().as_usize() + sizes_len..].len() {
                    $ret
                } else {
                    Err(ByteVecError::BadSizeDecodeError {
                        expected: BVExpectedSize::EqualTo(
                            Size::get_size_of().as_usize() + sizes_len + body_size),
                        actual: $byte_vec.len()
                    })
                }
            }
            else {
                Err(ByteVecError::BadSizeDecodeError {
                    expected: BVExpectedSize::MoreThan(Size::get_size_of().as_usize() + sizes_len),
                    actual: $byte_vec.len()
                })
            }
        } else {
            Err(ByteVecError::BadSizeDecodeError {
                expected: BVExpectedSize::MoreThan(Size::get_size_of().as_usize()),
                actual: $byte_vec.len()
            })
        }
    }};
}

impl ByteEncodable for str {
    fn get_size<Size>(&self) -> Option<Size>
        where Size: BVSize + ByteEncodable
    {
        if self.len() <= Size::max_value().as_usize() {
            Some(Size::from_usize(self.len()))
        } else {
            None
        }
    }

    fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>>
        where Size: BVSize + ByteEncodable
    {
        if self.get_size::<Size>().is_some() {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&self.as_bytes().to_vec());
            Ok(bytes)
        } else {
            Err(ByteVecError::OverflowError)
        }
    }
}

impl<'a> ByteEncodable for &'a str {
    fn get_size<Size>(&self) -> Option<Size>
        where Size: BVSize + ByteEncodable
    {
        (**self).get_size::<Size>()
    }

    fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>>
        where Size: BVSize + ByteEncodable
    {
        (**self).encode::<Size>()
    }
}

impl ByteEncodable for String {
    fn get_size<Size>(&self) -> Option<Size>
        where Size: BVSize + ByteEncodable
    {
        (**self).get_size::<Size>()
    }

    fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>>
        where Size: BVSize + ByteEncodable
    {
        (**self).encode::<Size>()
    }
}

impl ByteDecodable for String {
    fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<String>
        where Size: BVSize + ByteDecodable
    {
        Ok(try!(::std::str::from_utf8(bytes)).to_string())
    }
}

macro_rules! collection_encode_impl {
    () => {
        fn get_size<Size>(&self) -> Option<Size> where Size: BVSize + ByteEncodable {
            self.iter()
                .fold(Some(Size::from_usize(0)), |acc, elem| {
                    acc.and_then(|acc: Size| {
                        (&elem)
                            .get_size::<Size>()
                            .and_then(|size| {
                                acc.checked_add(size).and_then(|acc_size|
                                    acc_size.checked_add(Size::get_size_of())
                                )
                            })
                    })
                })
                .and_then(|total: Size| total.checked_add(Size::get_size_of()))
        }

        fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>> where Size: BVSize + ByteEncodable {
            if self.get_size::<Size>().is_some() {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&try!((Size::from_usize(self.len())).encode::<Size>()));
                for elem in self {
                    bytes.extend_from_slice(&try!(
                        (&elem).get_size::<Size>().unwrap().encode::<Size>()));
                }
                for elem in self {
                    bytes.extend_from_slice(&try!((&elem).encode::<Size>()));
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
    fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<Vec<T>>
        where Size: BVSize + ByteDecodable
    {
        let len;
        let mut index;
        let mut sizes;
        validate_collection!(bytes, index, len, sizes, {
            let mut vec = Vec::with_capacity(len);
            for size in sizes.into_iter() {
                vec.push(try!(T::decode::<Size>(&bytes[index..index + size.as_usize()])));
                index += size.as_usize();
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
    fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<HashSet<T>>
        where Size: BVSize + ByteDecodable
    {
        let len;
        let mut index;
        let mut sizes;
        validate_collection!(bytes, index, len, sizes, {
            let mut set = HashSet::with_capacity(len);
            for size in sizes.into_iter() {
                set.insert(try!(T::decode::<Size>(&bytes[index..index + size.as_usize()])));
                index += size.as_usize();
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
    fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<HashMap<K, V>>
        where Size: BVSize + ByteDecodable
    {
        let len;
        let mut index;
        let mut sizes;
        validate_collection!(bytes, index, len, sizes, {
            let mut map = HashMap::with_capacity(len);
            for size in sizes.into_iter() {
                let (key, value) = try!(<(K, V)>::decode::<Size>(&bytes[index..index +
                                                                               size.as_usize()]));
                map.insert(key, value);
                index += size.as_usize();
            }
            Ok(map)
        })
    }
}

macro_rules! tuple_impls {
    ($t:ident: $elem:ident) => {
        impl<$t,> ByteEncodable for ($t,)
            where $t: ByteEncodable
        {
            fn get_size<Size>(&self) -> Option<Size> where Size: BVSize + ByteEncodable {
                (&(&self.0)).get_size::<Size>()
            }

            fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>> where Size: BVSize + ByteEncodable {
                (&(&self.0)).encode::<Size>()
            }
        }

        impl<'a, $t,> ByteEncodable for &'a (&'a $t,)
            where $t: ByteEncodable
        {
            fn get_size<Size>(&self) -> Option<Size> where Size: BVSize + ByteEncodable {
                self.0.get_size::<Size>().and_then(|elem_size|
                    elem_size.checked_add(Size::get_size_of()))
            }

            fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>> where Size: BVSize + ByteEncodable {
                if self.get_size::<Size>().is_some() {
                    let mut bytes = Vec::new();
                    bytes.extend_from_slice(&try!(
                        self.0.get_size::<Size>().unwrap().encode::<Size>()));
                    bytes.extend_from_slice(&try!(self.0.encode::<Size>()));
                    Ok(bytes)
                } else {
                    Err(ByteVecError::OverflowError)
                }
            }
        }

        impl<$t,> ByteDecodable for ($t,)
            where $t: ByteDecodable
        {
            fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<($t,)>
                where Size: BVSize + ByteDecodable
            {
                let size;

                if bytes.len() >= Size::get_size_of().as_usize() {
                    size = try!(Size::decode::<Size>(&bytes[..Size::get_size_of().as_usize()]));
                }
                else {
                    return Err(ByteVecError::BadSizeDecodeError {
                        expected: BVExpectedSize::MoreThan(Size::get_size_of().as_usize()),
                        actual: bytes.len()
                    });
                }
                if size.as_usize() == bytes[Size::get_size_of().as_usize()..].len() {
                    Ok((try!($t::decode::<Size>(&bytes[Size::get_size_of().as_usize()..])),))
                } else {
                    Err(ByteVecError::BadSizeDecodeError {
                        expected: BVExpectedSize::EqualTo(
                            Size::get_size_of().as_usize() + size.as_usize()),
                        actual: bytes.len()
                    })
                }
            }
        }
    };

    // Lots of doubled code to implement recursion by dropping the first element each iteration,
    // so the first operation has to be done outside the macro loop, repeating the code
    ($t:ident: $elem:ident, $($_t:ident: $_elem:ident),*) => {
        impl<$t, $($_t,)*> ByteEncodable for ($t, $($_t),*)
            where $t: ByteEncodable, $($_t: ByteEncodable),*
        {
            fn get_size<Size>(&self) -> Option<Size> where Size: BVSize + ByteEncodable {
                let &(ref $elem, $(ref $_elem),*) = self;
                (&($elem, $($_elem),*)).get_size::<Size>()
            }

            fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>> where Size: BVSize + ByteEncodable {
                let &(ref $elem, $(ref $_elem),*) = self;
                (&($elem, $($_elem),*)).encode::<Size>()
            }
        }

        impl<'a, $t, $($_t,)*> ByteEncodable for &'a (&'a $t, $(&'a $_t),*)
            where $t: ByteEncodable, $($_t: ByteEncodable),*
        {
            fn get_size<Size>(&self) -> Option<Size> where Size: BVSize + ByteEncodable {
                let &&($elem, $($_elem),*) = self;
                let mut size = Some(Size::from_usize(0));

                size = size.and_then(|size: Size|
                    $elem.get_size::<Size>().and_then(|elem_size|
                        size.checked_add(elem_size).and_then(
                            |acc_size| acc_size.checked_add(Size::get_size_of())
                        )
                    )
                );
                $(
                    size = size.and_then(|size: Size|
                        $_elem.get_size::<Size>().and_then(|elem_size|
                            size.checked_add(elem_size).and_then(
                                |acc_size| acc_size.checked_add(Size::get_size_of())
                            )
                        )
                    );
                )*
                size
            }

            fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>> where Size: BVSize + ByteEncodable {
                if self.get_size::<Size>().is_some() {
                    let &&($elem, $($_elem),*) = self;
                    let mut bytes = Vec::new();
                    bytes.extend_from_slice(&try!(
                        $elem.get_size::<Size>().unwrap().encode::<Size>()));
                    $(
                        bytes.extend_from_slice(&try!(
                            $_elem.get_size::<Size>().unwrap().encode::<Size>()));
                    )*
                    bytes.extend_from_slice(&try!($elem.encode::<Size>()));
                    $(
                        bytes.extend_from_slice(&try!($_elem.encode::<Size>()));
                    )*
                    Ok(bytes)
                } else {
                    Err(ByteVecError::OverflowError)
                }
            }
        }

        #[allow(unused_assignments)]
        impl<$t, $($_t,)*> ByteDecodable for ($t, $($_t),*)
            where $t: ByteDecodable, $($_t: ByteDecodable),*
        {
            fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<($t, $($_t),*)>
                where Size: BVSize + ByteDecodable
            {
                let mut index = 0;
                let mut sizes = ::std::collections::HashMap::new();

                if bytes.len() >= Size::get_size_of().as_usize() {
                    sizes.insert(stringify!($elem),
                        try!(Size::decode::<Size>(&bytes[..Size::get_size_of().as_usize()])));
                    index += Size::get_size_of().as_usize();
                }
                else {
                    return Err(ByteVecError::BadSizeDecodeError {
                        expected: BVExpectedSize::MoreThan(Size::get_size_of().as_usize() + index),
                        actual: bytes.len()
                    });
                }
                $(
                    if bytes[index..].len() >= Size::get_size_of().as_usize() {
                        sizes.insert(stringify!($_elem),
                            try!(Size::decode::<Size>(
                                &bytes[index..index + Size::get_size_of().as_usize()])));
                        index += Size::get_size_of().as_usize();
                    }
                    else {
                        return Err(ByteVecError::BadSizeDecodeError {
                            expected: BVExpectedSize::MoreThan(Size::get_size_of().as_usize() + index),
                            actual: bytes.len()
                        });
                    }
                )*

                let body_size = sizes.values().fold(0, |acc, ref size| acc + size.as_usize());
                if body_size == bytes[index..].len() {
                    Ok((
                        {
                            let elem = try!($t::decode::<Size>(
                                &bytes[index..index + sizes[stringify!($elem)].as_usize()]));
                            index += sizes[stringify!($elem)].as_usize();
                            elem
                        },
                        $({
                            let elem = try!($_t::decode::<Size>(
                                &bytes[index..index + sizes[stringify!($_elem)].as_usize()]));
                            index += sizes[stringify!($_elem)].as_usize();
                            elem
                        }),*
                    ))
                } else {
                    Err(ByteVecError::BadSizeDecodeError {
                        expected: BVExpectedSize::EqualTo(
                            Size::get_size_of().as_usize() * sizes.len() + body_size),
                        actual: bytes.len()
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
    fn get_size<Size>(&self) -> Option<Size>
        where Size: BVSize + ByteEncodable
    {
        Some(Size::from_usize(0))
    }

    fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>>
        where Size: BVSize + ByteEncodable
    {
        // Send only size of 0
        Size::from_usize(0).encode::<Size>()
    }
}

impl ByteDecodable for () {
    fn decode<Size>(_: &[u8]) -> BVDecodeResult<()>
        where Size: BVSize + ByteDecodable
    {
        Ok(())
    }
}
