use crate::newtypes::EEIndex;
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_derive::{Decode as DeriveDecode, Encode as DeriveEncode};
use ssz_types::VariableList;
// TODO: Replace this with the actual max # of bytes a ShardTransaction can include
// Currently this is arbitrarily set to 256KB max size
use typenum::U262144;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, DeriveDecode, DeriveEncode)]
pub struct ShardTransaction {
    data: VariableList<u8, U262144>,
    ee_index: EEIndex,
}
