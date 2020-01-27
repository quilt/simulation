// Many patterns here copied from sigp/lighthouse with some small modifications
use fixed_hash::construct_fixed_hash;

// Necessary for impl_common macro
use serde::{Deserialize, Serialize};
use slog;
use ssz::{ssz_encode, Decode, DecodeError, Encode};
use std::cmp::{Ord, Ordering};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, Sub, SubAssign};

#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Slot(u64);

#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Epoch(u64);

#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Shard(u64);

#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShardSlot(u64);

impl_common!(Slot);
impl_common!(Epoch);
impl_common!(Shard);
impl_common!(ShardSlot);

construct_fixed_hash! {
    #[derive(Serialize, Deserialize)]
    pub struct Root(32);
}

impl Slot {
    pub fn new(slot: u64) -> Self {
        Self(slot)
    }
}

impl Epoch {
    pub fn new(slot: u64) -> Self {
        Self(slot)
    }
}

impl Decode for Root {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();
        let expected = <Self as Decode>::ssz_fixed_len();

        if len != expected {
            Err(DecodeError::InvalidByteLength { len, expected })
        } else {
            Ok(Root::from_slice(bytes))
        }
    }
}

impl Encode for Root {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}
