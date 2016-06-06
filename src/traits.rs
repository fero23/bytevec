use {BVEncodeResult, BVDecodeResult, BVSize};

pub trait ByteEncodable {
    fn get_size<Size>(&self) -> Option<Size> where Size: BVSize + ByteEncodable;
    fn encode<Size>(&self) -> BVEncodeResult<Vec<u8>> where Size: BVSize + ByteEncodable;
}

pub trait ByteDecodable: Sized {
    fn decode<Size>(bytes: &[u8]) -> BVDecodeResult<Self> where Size: BVSize + ByteDecodable;
}