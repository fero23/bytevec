# Version 0.2.0 (2016-06-06)
- Rename the `bytevec_impls` macro to `bytevec_decl`. This macro now accepts a pub 
  access modifier on the declared `struct` and on its fields. 
- Reintroduce the `bytevec_impls` macro. This macro now can do a partial or full implementation
  of the serialization and deserialization operations for an externally declared `struct`. Partial
  implementation in this context meaning that it can be implemented for only a subset of the 
  actual fields of the `struct` rather than for all the fields. The remaining fields will be 
  obtained from the value returned from [`Default::default()`] on deserialization.
- Change the fixed u32 type for the size indicators used in the byte representation of a complex
  structure. The methods of `ByteEncodable` and `ByteDecodable` now use an integral type parameter
  `Size` constrained by the `BVSize` trait. This trait is implemented for `u8`, `u16`, `u32` and `u64`.
  This now lifts the 4GB fixed limit of byte buffer, as the user now can use `u64` for the size indicator.
- Add a `decode_max` method to the `ByteDecodable` trait, so users are now able to set a limit to the
  length of byte buffer on deserialization. If the limit is less or equal than the buffer length, it will
  call and return the value returned from `decode`, otherwise it will return a `BadSizeDecodeError`.
- Change endianness from big endian to little endian.

[`Default::default()`]: http://doc.rust-lang.org/stable/std/default/trait.Default.html#tymethod.default

# Version 0.1.1 (2016-05-30)
- Minor fix to correct wrong usage of a `BVExpectedSize` value.

# Version 0.1.0 (2016-05-29)
- Introduce the `ByteEncodable` and `ByteDecodable` traits that provide methods for serialization
  and deserialization respectively.
- Implementations of the `ByteEncodable` and `ByteDecodable` traits for most base types.
- Introduce the `bytevec_impls` macro for custom `struct` declarations that automatically implement
  the `ByteEncodable` and `ByteDecodable` traits.
- `bytevec_impls` can only define private structs.
- Big endian encoding for primitive data types.
- u32 fixed length for the size indicators.
- Max 4GB fixed limit.