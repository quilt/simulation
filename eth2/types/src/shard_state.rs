use crate::slot_epoch_root::{Root, Slot};
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_derive::{Decode as DeriveDecode, Encode as DeriveEncode};

#[derive(Default, Debug, PartialEq, Clone, Deserialize, Serialize, DeriveDecode, DeriveEncode)]
pub struct ShardState {
    //    slot: Slot,
//    gasprice: Gwei,
//    data: Root,
//    latest_block_root: Root,
}
