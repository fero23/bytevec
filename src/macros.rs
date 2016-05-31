/// ###The `bytevec_impls` macro
/// This macro allows the user to declare an arbitrary number of structures that
/// automatically implement both the `ByteEncodable` and `ByteDecodable` traits,
/// as long as all of the fields also implement both traits.
/// 
/// ```rust
/// # #[macro_use]
/// # extern crate bytevec;
/// # 
/// # use bytevec::{ByteEncodable, ByteDecodable};
/// # 
/// bytevec_impls! {
///     #[derive(PartialEq, Eq, Debug)]
///     struct Point {
///         x: u32,
///         y: u32
///     }
/// }
/// 
/// fn main() {
///     let p1 = Point {x: 32, y: 436};
///     let bytes = p1.encode().unwrap();
///     let p2 = Point::decode(&bytes).unwrap();
///     assert_eq!(p1, p2);
/// }
/// ```
#[macro_export]
macro_rules! bytevec_impls {
    {$($(#[$attr:meta])* struct $name:ident {$($field:ident : $t:ty),*})*} => {
        $(
            $(#[$attr])*
            struct $name {
                $($field: $t),*
            }

            impl $crate::ByteEncodable for $name {
                fn get_size(&self) -> Option<u32> {
                    let mut size = Some(0);
                    $(
                        size = size.and_then(|size: u32|
                            self.$field.get_size().and_then(|field_size|
                                size.checked_add(field_size).and_then(
                                    |acc_size| acc_size.checked_add(4)
                                )
                            )
                        );
                    )*
                    size
                }

                fn encode(&self) -> $crate::BVEncodeResult<Vec<u8>> {
                    if self.get_size().is_some() {
                        let mut bytes = Vec::new();
                        $(
                            bytes.extend_from_slice(&try!(
                                self.$field.get_size().unwrap().encode()));
                        )*
                        $(
                            bytes.extend_from_slice(&try!(self.$field.encode()));
                        )*
                        Ok(bytes)
                    } else {
                        Err($crate::errors::ByteVecError::OverflowError)
                    }
                }
            }

            #[allow(dead_code, unused_assignments)]
            impl $crate::ByteDecodable for $name {
                fn decode(bytes: &[u8]) -> $crate::BVDecodeResult<$name> {
                    let mut index = 0;
                    let mut sizes = ::std::collections::HashMap::new();
                    $(
                        if bytes[index..].len() >= 4 {
                            sizes.insert(stringify!($field),
                                try!(u32::decode(&bytes[index..index + 4])));
                            index += 4;
                        }
                        else {
                            return Err($crate::errors::ByteVecError::BadSizeDecodeError {
                                wanted: $crate::errors::BVWantedSize::MoreThan(4 + index as u32),
                                actual: bytes.len() as u32
                            });
                        }
                    )*

                    let body_size = sizes.values().fold(0, |acc, &size| acc + size);
                    if body_size as usize == bytes[index..].len() {
                        Ok($name {
                            $(
                                $field: {
                                    let field = try!(<$t as $crate::ByteDecodable>::decode(
                                        &bytes[index..index + sizes[stringify!($field)] as usize]));
                                    index += sizes[stringify!($field)] as usize;
                                    field
                                }
                            ),*
                        })
                    } else {
                        Err($crate::errors::ByteVecError::BadSizeDecodeError {
                            wanted: $crate::errors::BVWantedSize::EqualTo(
                                4 * sizes.len() as u32 + body_size),
                            actual: bytes.len() as u32
                        })
                    }
                }
            }
        )*
    };
}
