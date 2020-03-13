use crate::eth_spec::EthSpec;
use crate::slot_epoch_root::Root;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::VariableList;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ExecutionEnvironment<T>
where
    T: EthSpec,
{
    pub initial_state: Root,
    pub wasm_code: VariableList<u8, T::MaxEEByteCodeSize>,
}
