bytevec: A Rust serialization library that uses byte vectors
============================================================

[![Build Status](https://travis-ci.org/fero23/bytevec.svg?branch=master)](https://travis-ci.org/fero23/bytevec)
[![](https://img.shields.io/crates/v/bytevec.svg)](https://crates.io/crates/bytevec)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
[![Apache 2.0 licensed](https://img.shields.io/badge/license-APACHE%202.0-blue.svg)](./LICENSE-APACHE)

bytevec takes advantage of Rust's concise and stable type system to
serialize data objects to a byte vector (`Vec<u8>`) and back.

What does it do?
----------------
Rust has a very powerful type system with predictable sizes for most
types, starting with the primitive types, so it's fairly easy to convert
any type to a collection of bytes and convert it back. This library intends
to give the user the means of converting a given type instance to a byte vector
and store it or send it through the wire to another device, with the possibility
of getting the value back from the byte vector anytime using the library traits.

Of course, Rust isn't magical enough to implement the traits to serialize
the functions automatically, as every type has its quirks. This library
uses two traits to give a type the functionality it needs to do that: 
`ByteEncodable` and `ByteDecodable`.

###The `ByteEncodable` trait
A type that implements this trait is able to use the `encode` method that 
yields a `Vec<u8>` byte sequence. Seems prone to failure right? Of course it is,
internally it uses `unsafe` blocks to extract the bytes of a given type, so 
it can be pretty unsafe. That's why it always checks for any possible error and
returns the vector wrapped around a `BVEncodeResult` instance. If everything
goes `Ok`, we will be able to get a byte vector value that represents the 
original data structure.

bytevec doesn't actually do a 1:1 conversion of the bytes of the original
type instance, as not every Rust type is stored on the stack. For any type
that wraps a heap stored value, it will give a representation of the 
underlying value.

bytevec implements `ByteEncodable` out of the box for the following types:
- The integral types: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`
- The floating point types: `f32` and `f64`
- `char`, `str` and `String`
- [`Vec`](http://doc.rust-lang.org/stable/std/vec/struct.Vec.html)
- [`&[T]`](http://doc.rust-lang.org/stable/std/primitive.slice.html)
- [`HashMap`](http://doc.rust-lang.org/stable/std/collections/struct.HashMap.html)
- [`HashSet`](http://doc.rust-lang.org/stable/std/collections/struct.HashSet.html)
- Tuples with up to 12 elements
- Custom `struct`s

For collections and other structures, automatic implementation of bytevec
requires that all of its underlying elements implement the `ByteEncodable`
trait.

###The bytevec serialization format
bytevec doesn't follow any particular serialization format. It follows simple
rules when translating some type value to bytes:
- For a primitive type such as the integral types, floating points
or char that have fixed size, it will just grab the bytes and put them 
on a `u8` buffer of the same length as the size of the type through 
[`std::mem::transmute`]. These types are converted to and from big endian on
serialization and deserialization respectively.
- String and str don't store their byte count, it's up to their container (if any)
to store the size of the byte buffer of the string.
- For structures with defined fields such as a custom `struct` or a tuple,
it will store the size of each field on an `u32` value in order at the start
of the slice segment for the structure, followed by the actual bytes of 
the values of the fields.
- For any collection with variable length, it will first store the length
(in elements, not byte count) on an `u32` value, followed by the byte count
(yes, in `u32`) of each element, and then the actual values of the elements.
All of this done in order, order is important, the same order of serialization
is the order of deserialization.
- All serializable values can be nested, so any structure that implements 
`ByteEncodable` containing a `Vec`, `String`, or another structure that also implements
`ByteEncodable` will be serialized along all its fields.

###The `ByteDecodable` trait
Given a byte vector retrieved from memory, a file, or maybe a TCP connection,
the user will be able to pass the vector to the `decode` method of
a type that implements the `ByteDecodable` trait. `decode` will do a few checks 
on the byte vector and if the required sizes matches, it will yield a type instance wrapped 
in a `BVDecodeResult`. If the size doesn't match, or if some other conversion problem 
arises, it will yield a `ByteVecError` detailing the failure.

Almost all of the out of the box implementations of `ByteEncodable` also
implement `ByteDecodable`, but some of them, particularly the slices and 
the tuple references don't make sense when deserialized, as they can't
point to the original data they were referencing. This is usually a problem
that requires some tweaking, but bytevec allows data conversion from byte
buffers that were originally referenced data to a new instance of an owned data type,
as long as the size requirements are the same. This way, slice data can
be assigned to a `Vec` instance for example, as long as they share the same 
type of the underlying elements.

###Example: Serialization and deserialization of a slice

```rust
let slice = &["Rust", "Is", "Awesome!"];
let bytes = slice.encode().unwrap();
let vec = <Vec<String>>::decode(&bytes).unwrap();
assert_eq!(vec, slice);
```

###The `bytevec_decl` macro
This macro allows the user to declare an arbitrary number of structures that
automatically implement both the `ByteEncodable` and `ByteDecodable` traits,
as long as all of the fields also implement both traits.

```rust
#[macro_use]
extern crate bytevec;

use bytevec::{ByteEncodable, ByteDecodable};

bytevec_decl! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct Point {
        x: u32,
        y: u32
    }
}

fn main() {
    let p1 = Point {x: 32, y: 436};
    let bytes = p1.encode().unwrap();
    let p2 = Point::decode(&bytes).unwrap();
    assert_eq!(p1, p2);
}
```

###The `bytevec_impls` macro

This macro implements both the `ByteEncodable` and `ByteDecodable` traits
for the given `struct` definitions. This macro does not declare the `struct`
definitions, the user should either declare them separately or use the
`bytevec_decl` trait.

This trait also allows the user to create a partial implementation of the
serialization operations for a select number of the fields of the 
structure. If the actual definition of the `struct` has more fields than
the one provided to the macro, only the listed fields in the macro invocation
will be serialized and deserialized. In the deserialization process, the
rest of the fields of the `struct` will be initialized using the value
returned from the [`Default::default()`] method.

```rust
#[macro_use]
extern crate bytevec;

use bytevec::{ByteEncodable, ByteDecodable};

#[derive(PartialEq, Eq, Debug, Default)]
struct Vertex3d {
    x: u32,
    y: u32,
    z: u32
}

bytevec_impls! {
    struct Vertex3d {
        x: u32,
        y: u32
    }
}

fn main() {
    let p1 = Vertex3d {x: 32, y: 436, z: 0};
    let bytes = p1.encode().unwrap();
    let p2 = Vertex3d::decode(&bytes).unwrap();
    assert_eq!(p1, p2);
}
```

####This all sounds like your usual serialization library, but why bother with bytes?
bytevec certainly isn't for everyone. It isn't a full serialization library like
[rustc_serialize] or [serde], nor is it trying to become one. This is for the people
that for any reason can't handle text based serializing and just need 
to get some bytes fast and recreate an object out of them with low overhead through the use
of a small crate with no dependencies.

##License
This library is distributed under both the MIT license and the Apache License (Version 2.0).
You are free to use any of them as you see fit.

[`Default::default()`]: http://doc.rust-lang.org/stable/std/default/trait.Default.html#tymethod.default
[`std::mem::transmute`]: http://doc.rust-lang.org/stable/std/mem/fn.transmute.html
[rustc_serialize]: https://github.com/rust-lang-nursery/rustc-serialize
[serde]: https://github.com/serde-rs/serde