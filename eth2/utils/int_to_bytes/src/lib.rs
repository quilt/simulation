// Copied from sigp/lighthouse with some modifications and deletions
use bytes::{BufMut, BytesMut};

/// Returns `int` as little-endian bytes with a length of 32.
pub fn int_to_bytes32(int: u64) -> Vec<u8> {
    let mut bytes = BytesMut::with_capacity(32);
    bytes.put_u64_le(int);
    bytes.resize(32, 0);
    bytes.to_vec()
}
