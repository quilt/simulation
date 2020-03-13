use crate::eth_spec::EthSpec;
use crate::shard_transaction::ShardTransaction;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode as DeriveDecode, Encode as DeriveEncode};
use ssz_types::VariableList;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, DeriveDecode, DeriveEncode)]
pub struct ShardBlock<T>
where
    T: EthSpec,
{
    pub transactions: VariableList<ShardTransaction, T::MaxTransactionsPerBlock>,
}
