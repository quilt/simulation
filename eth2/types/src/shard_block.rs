use crate::shard_transaction::ShardTransaction;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode as DeriveDecode, Encode as DeriveEncode};
use ssz_types::VariableList;
// TODO: Replace this with the actual max # of transactions a block can contain
use typenum::U1024;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, DeriveDecode, DeriveEncode)]
pub struct ShardBlock {
    pub transactions: VariableList<ShardTransaction, U1024>,
}
