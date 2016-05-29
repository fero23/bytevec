use {BVEncodeResult, BVDecodeResult};

pub trait ByteEncodable {
    fn get_size(&self) -> Option<u32>;
    fn encode(&self) -> BVEncodeResult<Vec<u8>>;
}

pub trait ByteDecodable: Sized {
    fn decode(bytes: &[u8]) -> BVDecodeResult<Self>;
}