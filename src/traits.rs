use {BVEncodeResult, BVDecodeResult, BVSize};
use errors::{ByteVecError, BVExpectedSize};

pub trait ByteEncodable {
    fn get_size<Size>(&self) -> Option<Size> where Size: BVSize + ByteEncodable;
    fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>> where Size: BVSize + ByteEncodable;
}

pub trait ByteDecodable: Sized {
    fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<Self> where Size: BVSize + ByteDecodable;
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