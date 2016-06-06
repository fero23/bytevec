use {BVEncodeResult, BVDecodeResult, BVSize};
use errors::{ByteVecError, BVExpectedSize};

/// Provides serialization functionality for the implementing types.
pub trait ByteEncodable {
    /// Returns the total length of the byte buffer 
    /// than can be obtained through the `encode` method  
    fn get_size<Size>(&self) -> Option<Size> where Size: BVSize + ByteEncodable;
    /// Returs a byte representation of the original data object
    fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>> where Size: BVSize + ByteEncodable;
}

/// Provides deserialization functionality for the implementing types.
pub trait ByteDecodable: Sized {
    /// Returns an instance of `Self` obtained from the deserialization of the provided byte buffer.
    fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<Self> where Size: BVSize + ByteDecodable;
    /// Returns the result of `decode` if `bytes.len()` is less or equal than `limit`
    fn decode_max<Size>(bytes: &[u8], limit: Size) -> BVDecodeResult<Self>
        where Size: BVSize + ByteDecodable
    {
        if bytes.len() <= limit.as_usize() {
            Self::decode::<Size>(bytes)
        } else {
            Err(ByteVecError::BadSizeDecodeError {
                expected: BVExpectedSize::LessOrEqualThan(limit.as_usize()),
                actual: bytes.len(),
            })
        }
    }
}