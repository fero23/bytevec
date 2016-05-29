mod collections;
mod primitives;

/* // Uncomment when specialization becomes stable
use {ByteEncodable, BVEncodeResult};

impl<'a, E: ByteEncodable>  ByteEncodable for &'a E {
    fn get_size(&self) -> Option<u32> {
        (**self).get_size()
    }

    fn encode(&self) -> BVEncodeResult<Vec<u8>> {
        (**self).encode()
    }
}
*/