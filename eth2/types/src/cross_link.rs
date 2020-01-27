use crate::newtypes::{Root, Slot};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::VariableList;
// TODO: Replace this with the spec-defined max # of slots back each crosslink can hold
use typenum::U64;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize, Encode, Decode)]
pub struct CrossLink {
    shard_roots: VariableList<(Slot, Root), U64>,
}
