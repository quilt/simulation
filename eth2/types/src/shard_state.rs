use crate::slot_epoch_root::{Root};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode as DeriveDecode, Encode as DeriveEncode};
use ssz_types::{VariableList};
use crate::eth_spec::EthSpec;

#[derive(Default, Debug, PartialEq, Clone, Deserialize, Serialize, DeriveDecode, DeriveEncode)]
pub struct ShardState<T>
where
    T: EthSpec,
{
    //    slot: Slot,
    //    gasprice: Gwei,
    //    data: Root,
    //    latest_block_root: Root,

    // Unspecced fields
    pub execution_environment_states: VariableList<Root, T::MaxExecutionEnvironments>,
}

impl<T: EthSpec> ShardState<T> {
    pub fn new() -> Self {
        Self {
            execution_environment_states: VariableList::empty(),
        }
    }
}
