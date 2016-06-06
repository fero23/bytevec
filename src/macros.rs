/// Implements the byte serialization traits for the given structures.
///
/// This macro implements both the `ByteEncodable` and `ByteDecodable` traits
/// for the given `struct` definitions. This macro does not declare the `struct`
/// definitions, the user should either declare them separately or use the
/// `bytevec_decl` trait.
///
/// This trait also allows the user to create a partial implementation of the
/// serialization operations for a select number of the fields of the
/// structure. If the actual definition of the `struct` has more fields than
/// the one provided to the macro, only the listed fields in the macro invocation
/// will be serialized and deserialized. In the deserialization process, the
/// rest of the fields of the `struct` will be initialized using the value
/// returned from the [`Default::default()`][1] method.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate bytevec;
/// #
/// # use bytevec::{ByteEncodable, ByteDecodable};
/// #
/// #[derive(PartialEq, Eq, Debug, Default)]
/// struct Vertex3d {
///     x: u32,
///     y: u32,
///     z: u32
/// }
///
/// bytevec_impls! {
///     struct Vertex3d {
///         x: u32,
///         y: u32
///     }
/// }
///
/// fn main() {
///     let p1 = Vertex3d {x: 32, y: 436, z: 0};
///     let bytes = p1.encode::<u32>().unwrap();
///     let p2 = Vertex3d::decode::<u32>(&bytes).unwrap();
///     assert_eq!(p1, p2);
/// }
/// ```
/// [1]: http://doc.rust-lang.org/stable/std/default/trait.Default.html#tymethod.default
#[macro_export]
macro_rules! bytevec_impls {
    {$(struct $name:ident {$($field:ident : $t:ty),*})*} => {
        $(
            impl $crate::ByteEncodable for $name
            {
                fn get_size<Size>(&self) -> Option<Size>
                    where Size: $crate::BVSize + $crate::ByteEncodable
                {
                    let mut size = Some(Size::from_usize(0));
                    $(
                        size = size.and_then(|size: Size|
                            self.$field.get_size::<Size>().and_then(|field_size|
                                size.checked_add(field_size).and_then(
                                    |acc_size| acc_size.checked_add(
                                        Size::get_size_of())
                                )
                            )
                        );
                    )*
                    size
                }

                fn encode<Size>(&self) -> $crate::BVEncodeResult<Vec<u8>>
                    where Size: $crate::BVSize + $crate::ByteEncodable
                {
                    if self.get_size::<Size>().is_some() {
                        let mut bytes = Vec::new();
                        $(
                            let field_size: Option<Size> = self.$field.get_size::<Size>();
                            bytes.extend_from_slice(&try!(
                                field_size.unwrap().encode::<Size>()));
                        )*
                        $(
                            bytes.extend_from_slice(&try!(self.$field.encode::<Size>()));
                        )*
                        Ok(bytes)
                    } else {
                        Err($crate::errors::ByteVecError::OverflowError)
                    }
                }
            }

            #[allow(dead_code, unused_assignments)]
            impl $crate::ByteDecodable for $name {
                fn decode<Size>(bytes: &[u8]) -> $crate::BVDecodeResult<$name>
                    where Size: $crate::BVSize + $crate::ByteDecodable
                {
                    let mut index = 0;
                    let mut sizes = ::std::collections::HashMap::new();
                    $(
                        if bytes[index..].len() >= Size::get_size_of().as_usize() {
                            sizes.insert(stringify!($field),
                                try!(Size::decode::<Size>(
                                    &bytes[index..index + Size::get_size_of().as_usize()])));
                            index += Size::get_size_of().as_usize();
                        }
                        else {
                            return Err($crate::errors::ByteVecError::BadSizeDecodeError {
                                wanted: $crate::errors::BVWantedSize::MoreThan(
                                    Size::get_size_of().as_usize() + index),
                                actual: bytes.len()
                            });
                        }
                    )*

                    let body_size = sizes.values().fold(0, |acc, ref size| acc + size.as_usize());
                    if body_size == bytes[index..].len() {
                        Ok($name {
                            $(
                                $field: {
                                    let size = sizes[stringify!($field)].as_usize();
                                    let field = try!(<$t as $crate::ByteDecodable>::decode::<Size>(
                                        &bytes[index..index + size]));
                                    index += size;
                                    field
                                },
                            )*
                            ..Default::default()
                        })
                    } else {
                        Err($crate::errors::ByteVecError::BadSizeDecodeError {
                            wanted: $crate::errors::BVWantedSize::EqualTo(
                                Size::get_size_of().as_usize() * sizes.len() + body_size),
                            actual: bytes.len()
                        })
                    }
                }
            }
        )*
    };
}


/// Declares the given structures and implements the byte serialization traits.
///
/// This macro allows the user to declare an arbitrary number of structures that
/// automatically implement both the `ByteEncodable` and `ByteDecodable` traits,
/// as long as all of the fields also implement both traits.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate bytevec;
/// #
/// # use bytevec::{ByteEncodable, ByteDecodable};
/// #
/// bytevec_decl! {
///     #[derive(PartialEq, Eq, Debug)]
///     pub struct Point {
///         x: u32,
///         y: u32
///     }
/// }
///
/// fn main() {
///     let p1 = Point {x: 32, y: 436};
///     let bytes = p1.encode::<u32>().unwrap();
///     let p2 = Point::decode::<u32>(&bytes).unwrap();
///     assert_eq!(p1, p2);
/// }
/// ```
#[macro_export]
macro_rules! bytevec_decl {
    {$($(#[$attr:meta])* struct $name:ident {$($field:ident : $t:ty),*})*} => {
        $(
            $(#[$attr])*
            #[derive(Default)]
            struct $name {
                $($field: $t),*
            }
            bytevec_impls!(struct $name {$($field:$t),*});
        )*
    };

    {$($(#[$attr:meta])* pub struct $name:ident {$(pub $field:ident : $t:ty),*})*} => {
        $(
            $(#[$attr])*
            #[derive(Default)]
            pub struct $name {
                $(pub $field: $t),*
            }
            bytevec_impls!(struct $name {$($field:$t),*});
        )*
    };

    {$($(#[$attr:meta])* pub struct $name:ident {$($field:ident : $t:ty),*})*} => {
        $(
            $(#[$attr])*
            #[derive(Default)]
            pub struct $name {
                $($field: $t),*
            }
            bytevec_impls!(struct $name {$($field:$t),*});
        )*
    };
}
