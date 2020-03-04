/// Holds all the types necessary to interact with the `Simulation` struct
/// These public interface values do not hold "internal" types, and instead only use "basic" Rust
/// types.
use base64;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use snafu::Snafu;

mod internal_types {
    pub use ssz_types::VariableList;
    pub use types::eth_spec::EthSpec;
    pub use types::execution_environment::ExecutionEnvironment;
    pub use types::shard_block::ShardBlock;
    pub use types::shard_state::ShardState;
    pub use types::shard_transaction::ShardTransaction;
    pub use types::slot_epoch_root::Root;
}

/// Shorthand for result types returned from the Simulation simulation.
pub type Result<V, E = Error> = std::result::Result<V, E>;

#[derive(Debug)]
pub enum WhatBound {
    ExecutionEnvironment,
    ExecutionEnvironmentState,
    ShardBlock(usize),
    Shard,
}

impl fmt::Display for WhatBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WhatBound::ExecutionEnvironment => write!(f, "execution environment"),
            WhatBound::ExecutionEnvironmentState => write!(f, "execution environment state"),
            WhatBound::Shard => write!(f, "shard"),
            WhatBound::ShardBlock(shard) => write!(f, "block on shard {}", shard),
        }
    }
}

/// Errors arising from the simulation.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{} exceeds max allowable length", what))]
    MaxLengthExceeded { what: String },
    #[snafu(display("no {} exists at index: {}", what, index))]
    OutOfBounds { what: WhatBound, index: usize },
    InvalidBytes32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateExecutionEnvironment {
    pub ee: ExecutionEnvironment,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateShardBlock {
    pub shard_index: u64,
    pub shard_block: ShardBlock,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct GetExecutionEnvironment {
    pub ee_index: u64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct GetExecutionEnvironmentState {
    pub ee_index: u64,
    pub shard_index: u64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct GetShardBlock {
    pub shard_index: u64,
    pub shard_slot_index: u64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct GetShardState {
    pub shard_index: u64,
}

/// Defines custom serialization for basic return types
/// If serialization is required, appropriate basic types returned from the Simulation can be
/// wrapped in the appropriate enum entry to tell Serde how to custom-serialize the type.
// TODO: Possibly there's a more elegant way to achieve this same result?
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomSerializedReturnTypes {
    #[serde(with = "base64_arr")]
    Base64EncodedRoot([u8; 32]),
}

// Interface structs

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExecutionEnvironment {
    #[serde(with = "base64_arr")]
    pub initial_state: [u8; 32],

    #[serde(with = "base64_vec")]
    pub wasm_code: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ShardTransaction {
    pub data: Vec<u8>,
    pub ee_index: u64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ShardBlock {
    pub transactions: Vec<ShardTransaction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ShardState {
    #[serde(serialize_with = "vec_base64_arrs::serialize")]
    pub execution_environment_states: Vec<[u8; 32]>,
}


// Conversions to/from interface structs <--> internal structs

impl<T: internal_types::EthSpec> From<internal_types::ExecutionEnvironment<T>> for ExecutionEnvironment {
    fn from(value: internal_types::ExecutionEnvironment<T>) -> Self {
        let initial_state: [u8; 32] = value.initial_state.into();
        let wasm_code: Vec<u8> = value.wasm_code.into();
        Self {
            initial_state,
            wasm_code,
        }
    }
}
impl<T: internal_types::EthSpec> TryFrom<ExecutionEnvironment> for internal_types::ExecutionEnvironment<T> {
    type Error = crate::Error;
    fn try_from(value: ExecutionEnvironment) -> Result<Self, Self::Error> {
        let initial_state: internal_types::Root = internal_types::Root::from(value.initial_state);
        // TODO(gregt): Switch this to wrap the underlying error
        let wasm_code = internal_types::VariableList::new(value.wasm_code).map_err(|_| Error::MaxLengthExceeded {
            what: format!("wasm_code"),
        })?;

        Ok(Self {
            initial_state,
            wasm_code,
        })
    }
}

impl From<internal_types::ShardTransaction> for ShardTransaction {
    fn from(value: internal_types::ShardTransaction) -> Self {
        let data: Vec<u8> = value.data.into();
        let ee_index: u64 = value.ee_index.into();
        Self {
            data,
            ee_index,
        }
    }
}
impl TryFrom<ShardTransaction> for internal_types::ShardTransaction {
    type Error = crate::Error;
    fn try_from(value: ShardTransaction) -> Result<Self, Self::Error> {
        let ee_index = value.ee_index.into();
        // TODO(gregt): Switch this to wrap the underlying error
        let data = internal_types::VariableList::new(value.data).map_err(|_| Error::MaxLengthExceeded {
            what: format!("data"),
        })?;
        Ok(Self {
            data,
            ee_index,
        })
    }
}

impl<T: internal_types::EthSpec> From<internal_types::ShardBlock<T>> for ShardBlock {
    fn from(value: internal_types::ShardBlock<T>) -> Self {
        let transactions: Vec<ShardTransaction> = value.transactions.into_iter().map(|t| -> ShardTransaction {
            t.clone().into()
        }).collect();
        Self {
            transactions,
        }
    }
}
impl<T: internal_types::EthSpec> TryFrom<ShardBlock> for internal_types::ShardBlock<T> {
    type Error = crate::Error;
    fn try_from(value: ShardBlock) -> Result<Self, Self::Error> {
        let mut transactions: Vec<internal_types::ShardTransaction> = Vec::new();
        for t in value.transactions.into_iter() {
            let transaction = t.try_into()?;
            transactions.push(transaction);
        }
        // TODO(gregt): Switch this to wrap the underlying error
        let transactions = internal_types::VariableList::new(transactions).map_err(|_| Error::MaxLengthExceeded {
            what: format!("transactions per shard block"),
        })?;
        Ok(Self {
            transactions,
        })
    }
}

impl<T: internal_types::EthSpec> From<internal_types::ShardState<T>> for ShardState {
    fn from(value: internal_types::ShardState<T>) -> Self {
        let execution_environment_states: Vec<[u8; 32]> = value.execution_environment_states.into_iter().map(|t| -> [u8; 32] {
            t.0
        }).collect();
        Self {
            execution_environment_states,
        }
    }
}
impl<T: internal_types::EthSpec> TryFrom<ShardState> for internal_types::ShardState<T> {
    type Error = crate::Error;
    fn try_from(value: ShardState) -> Result<Self, Self::Error> {
        let execution_environment_states: Vec<internal_types::Root>  = value.execution_environment_states.into_iter().map(|t| -> internal_types::Root {
            internal_types::Root::from(t)
        }).collect();
        // TODO(gregt): Switch this to wrap the underlying error
        let execution_environment_states = internal_types::VariableList::new(execution_environment_states).map_err(|_| Error::MaxLengthExceeded {
            what: format!("execution environment states"),
        })?;
        Ok(Self {
            execution_environment_states,
        })
    }
}

// Serialization helpers

pub trait ToBytes32 {
    fn to_bytes32(&self) -> Result<[u8; 32]>;
}

impl ToBytes32 for Vec<u8> {
    fn to_bytes32(&self) -> Result<[u8; 32]> {
        if self.len() == 32 {
            let mut ret: [u8; 32] = [0; 32];
            ret.copy_from_slice(&self[..]);
            Ok(ret)
        } else {
            Err(Error::InvalidBytes32)
        }
    }
}

mod vec_base64_arrs {
    use serde::ser::{Serialize, Serializer, SerializeSeq};
    use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor, Unexpected};
    use std::fmt;
    use super::ToBytes32;

    pub fn serialize<S>(vec: &Vec<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(vec.len()))?;
        for bytes_arr in vec {
            let txt = base64::encode(bytes_arr.as_ref());
            seq.serialize_element(&txt)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 32]>, D::Error>
        where
            D: Deserializer<'de>,
    {
        // Convert to vector of strings first
        let string_vec: Vec<&str> = Vec::deserialize(deserializer)?;

        // Then attempt to convert to Vec<[u8; 32]>
        let result_vec: Result<Vec<[u8; 32]>, D::Error> = string_vec.into_iter().map(|s| -> Result<[u8; 32], D::Error> {
            // TODO: Some duplicated code between this deserialize and the deserialize methods below
            // There's probably a better way to do this without repeating that logic.
            let vec_u8 = base64::decode(s)
                .map_err(|_| D::Error::invalid_value(Unexpected::Str(s), &"base64 encoded bytes"))?;

            vec_u8.to_bytes32().map_err(|_| {
                D::Error::invalid_value(Unexpected::Bytes(&vec_u8), &"exactly 32 base64 encoded bytes")
            })
        }).collect();

        result_vec
    }
}

mod base64_vec {
    use serde::de::{Deserialize, Deserializer, Error as _, Unexpected};
    use serde::Serializer;

    pub fn serialize<T, S>(bytes: T, serializer: S) -> Result<S::Ok, S::Error>
        where
            T: AsRef<[u8]>,
            S: Serializer,
    {
        let txt = base64::encode(bytes.as_ref());
        serializer.serialize_str(&txt)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let txt = String::deserialize(deserializer)?;

        base64::decode(&txt)
            .map_err(|_| D::Error::invalid_value(Unexpected::Str(&txt), &"base64 encoded bytes"))
    }
}

mod base64_arr {
    use serde::de::{Deserialize, Deserializer, Error as _, Unexpected};
    use serde::Serializer;

    use super::ToBytes32;

    pub use super::base64_vec::serialize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
        where
            D: Deserializer<'de>,
    {
        let vec = super::base64_vec::deserialize(deserializer)?;

        vec.to_bytes32().map_err(|_| {
            D::Error::invalid_value(Unexpected::Bytes(&vec), &"exactly 32 base64 encoded bytes")
        })
    }
}