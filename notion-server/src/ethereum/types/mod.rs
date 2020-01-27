use ssz::{ssz_encode, Decode, DecodeError, Encode, SszDecoderBuilder, SszEncoder};
use ssz_derive::{Decode, Encode};
use serde::{Deserialize, Serialize};

const SHARD_COUNT: usize = 64;

// Needed by `macros.rs`
use slog;
use std::fmt;
use std::cmp::{Ord, Ordering};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, Sub, SubAssign};

// Imports basic macros to add SSZ (and other) methods onto u64 newtypes
#[macro_use]
mod macros;

impl_common!(Slot);
impl_common!(Epoch);
impl_common!(Shard);
impl_common!(ShardSlot);

#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
struct Slot(u64);
#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
struct Epoch(u64);
#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
struct Shard(u64);
#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
struct ShardSlot(u64);
#[derive(PartialEq, Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
struct Root([u8; 32]);

impl Encode for Root {
    fn is_ssz_fixed_len() -> bool {
        <[u8; 32] as Encode>::is_ssz_fixed_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.0.ssz_append(buf);
    }

    fn ssz_fixed_len() -> usize {
        <[u8; 32] as Encode>::ssz_fixed_len()
    }
}

impl Decode for Root {
    fn is_ssz_fixed_len() -> bool {
        <[u8; 32] as Decode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <[u8; 32] as Decode>::ssz_fixed_len()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(<[u8; 32] as Decode>::from_ssz_bytes(bytes)?))
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
struct BeaconBlock {
    slot: Slot,
    parent_root: Root,
    state_root: Root,
    body: BeaconBlockBody,

    // Many fields removed
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
struct BeaconState {
    slot: Slot,

    cross_links: [u8; SHARD_COUNT],
    execution_environments: Vec<ExecutionEnvironment>,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
struct BeaconBlockBody {}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
struct ShardBlock {
    shard: Shard,
    slot: ShardSlot,
    beacon_block_root: Root,
    parent_root: Root,
    state_root: Root,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
struct ShardBlockHeader {
    shard: Shard,
    slot: ShardSlot,
    beacon_block_root: Root,
    parent_root: Root,
    state_root: Root,
    body_root: Root,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
struct ShardState {
    shard: Shard,
    slot: ShardSlot,

    execution_environment_states: Vec<ExecutionEnvironmentState>,
}

// unspecced, but eventually will be in spec in some form
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
struct CrossLink {
    shard_roots: Vec<(Slot, Root)>
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
struct ExecutionEnvironment {
    wasm_code: Vec<u8>,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
struct ExecutionEnvironmentState {
    data: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_test() {
        assert_eq!(0, 1, "spec types tests are running");
    }
}
