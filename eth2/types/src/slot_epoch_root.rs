// Many patterns copied from the slot_epoch.rs file in the
// https://github.com/sigp/lighthouse repo with some small modifications.  Specifically, added new
// types, and removed code not needed for this repo.
// The analog of the `Root` struct in this repo is H256 in lighthouse (and in some other
// ethereum-focused crates). Decided to use `Root` in this repo instead of `H256` because the spec
// refers to the data type as a `Root`.  This can be changed in the future if necessary.
use fixed_hash::construct_fixed_hash;

// Necessary for impl_common macro
use serde::{Deserialize, Serialize};
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

#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EEIndex(u64);

impl_common!(Slot);
impl_common!(Epoch);
impl_common!(Shard);
impl_common!(ShardSlot);
impl_common!(EEIndex);

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
    pub fn new(epoch: u64) -> Self {
        Self(epoch)
    }
}

impl EEIndex {
    pub fn new(ee_index: u64) -> Self { Self(ee_index) }
}

impl Shard {
    pub fn new(shard: u64) -> Self {
        Self(shard)
    }
}

impl ShardSlot {
    pub fn new(slot: u64) -> Self {
        Self(slot)
    }
}

impl Root {

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
