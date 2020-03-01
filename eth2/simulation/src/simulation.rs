use crate::store::Store;
use crate::{Error, Result, WhatBound};
use ewasm::{Execute, RootRuntime};
use std::convert::TryFrom;
use ssz_types::VariableList;
use types::eth_spec::EthSpec;
use types::execution_environment::ExecutionEnvironment;
use types::shard_block::ShardBlock;
use types::shard_state::ShardState;
use types::shard_transaction::ShardTransaction;
use types::slot_epoch_root::{EeIndex, Root, Shard, ShardSlot};

#[derive(Debug)]
pub struct Simulation<T>
where
    T: EthSpec,
{
    store: Store<T>,
}

impl<T: EthSpec> Simulation<T> {
    pub fn new() -> Self {
        Self {
            store: Store::new(),
        }
    }

    /// Add a new execution environment, return EE index
    pub fn create_execution_environment(
        &mut self,
        a: args::CreateExecutionEnvironment,
    ) -> Result<u64> {
        // Create internal EE struct from args
        let ee = ExecutionEnvironment::try_from(a.ee)?;
        let cloned_initial_state = ee.initial_state.clone();

        // Add EE code to beacon chain
        self.store
            .current_beacon_state
            .execution_environments
            .push(ee)
            .map_err(|_| Error::MaxLengthExceeded {
                what: format!("number of execution environments"),
            })?;

        // For each shard, add the initial state to the shard
        for shard_state in self.store.current_beacon_state.shard_states.iter_mut() {
            // Set the initial state of the EE on each ShardState
            shard_state
                .execution_environment_states
                .push(cloned_initial_state)
                .map_err(|_| Error::MaxLengthExceeded {
                    what: format!("number of execution environment states"),
                })?;

            // Each shard should have the same # of ee states as there are EEs
            assert_eq!(
                shard_state.execution_environment_states.len(),
                self.store.current_beacon_state.execution_environments.len()
            );
        }

        let ee_index = self.store.current_beacon_state.execution_environments.len() - 1;
        Ok(ee_index as u64)
    }

    /// Add a new shard block containing a list of transactions that need to be executed
    /// Execute all transactions on the appropriate shards / EEs, return ShardBlock index
    pub fn create_shard_block(&mut self, a: args::CreateShardBlock) -> Result<u64> {
        // Get the specified ShardState (if it exists)
        let shard_index = a.shard_index as usize;
        let shard = Shard::new(a.shard_index);
        let shard_state = self
            .store
            .current_beacon_state
            .shard_states
            .get_mut(shard_index)
            .ok_or(Error::OutOfBounds {
                what: WhatBound::Shard,
                index: shard_index,
            })?;

        // Create the internal shard block from args
        let shard_block: ShardBlock<T> = ShardBlock::try_from(a.shard_block)?;

        // Execute transactions and update shard state for all transactions
        for transaction in shard_block.transactions.iter() {
            // Get the specified EE (if it exists)
            let ee_index: usize = transaction.ee_index.into();
            let execution_environment = self
                .store
                .current_beacon_state
                .execution_environments
                .get(ee_index)
                .ok_or(Error::OutOfBounds {
                    what: WhatBound::ExecutionEnvironment,
                    index: ee_index,
                })?;

            // Get the current EE state
            let pre_state = shard_state
                .execution_environment_states
                .get(ee_index)
                .ok_or(Error::OutOfBounds {
                    what: WhatBound::ExecutionEnvironmentState,
                    index: ee_index,
                })?;

            // Create a new runtime with the EE code, transaction data, and pre state root
            let wasm_code: &[u8] = &*execution_environment.wasm_code;
            let data: &[u8] = &*transaction.data;
            let pre_state: [u8; 32] = pre_state.clone().into();
            let mut runtime = RootRuntime::new(wasm_code, data, pre_state);
            let post_root = runtime.execute();
            drop(runtime);

            // Update shard state with new root
            shard_state.execution_environment_states[ee_index] = Root::from(post_root);
        }

        // Add shard block to store for later access
        let shard_blocks_for_shard =
            self.store
                .shard_blocks_by_shard
                .get_mut(&shard)
                .ok_or(Error::OutOfBounds {
                    index: shard_index,
                    what: WhatBound::Shard,
                })?;
        shard_blocks_for_shard.push(shard_block);

        // Return the slot of the newly added shard block
        Ok((shard_blocks_for_shard.len() - 1) as u64)
    }

    /// Get an EE that was previously added
    pub fn get_execution_environment(
        &self,
        a: args::GetExecutionEnvironment,
    ) -> Result<args::ExecutionEnvironment> {
        let ee_index= a.ee_index as usize;
        let ee = self
            .store
            .current_beacon_state
            .execution_environments
            .get(ee_index)
            .ok_or(Error::OutOfBounds {
                what: WhatBound::ExecutionEnvironment,
                index: ee_index,
            })?;
        Ok(ee.clone().into())
    }

    /// Get the current state of an execution environment on a shard
    pub fn get_execution_environment_state(
        &self,
        a: args::GetExecutionEnvironmentState,
    ) -> Result<[u8; 32]> {
        let ee_index = a.ee_index as usize;
        let shard_index = a.shard_index as usize;
        let shard_state = self
            .store
            .current_beacon_state
            .shard_states
            .get(shard_index)
            .ok_or(Error::OutOfBounds {
                what: WhatBound::Shard,
                index: shard_index,
            })?;
        let ee_state_root = shard_state
            .execution_environment_states
            .get(ee_index)
            .ok_or(Error::OutOfBounds {
                what: WhatBound::ExecutionEnvironmentState,
                index: ee_index,
            })?;
        Ok(ee_state_root.clone().into())
    }

    /// Get a shard block that was previously added
    pub fn get_shard_block(&self, a: args::GetShardBlock) -> Result<args::ShardBlock> {
        let shard_index = a.shard_index as usize;
        let shard_slot_index = a.shard_slot_index as usize;
        let shard = Shard::new(a.shard_index);
        let shard_blocks = self
            .store
            .shard_blocks_by_shard
            .get(&shard)
            .ok_or(Error::OutOfBounds {
                what: WhatBound::Shard,
                index: shard_index,
            })?;
        let shard_block = shard_blocks.get(shard_slot_index).ok_or(Error::OutOfBounds {
            what: WhatBound::ShardBlock(shard_index),
            index: shard_slot_index,
        })?;
        Ok(shard_block.clone().into())
    }

    /// Get the specified ShardState, will contain EE states
    pub fn get_shard_state(&self, a: args::GetShardState) -> Result<args::ShardState> {
        let shard_index = a.shard_index as usize;
        let shard_state = self
            .store
            .current_beacon_state
            .shard_states
            .get(shard_index)
            .ok_or(Error::OutOfBounds {
                what: WhatBound::Shard,
                index: shard_index,
            })?;
        Ok(shard_state.clone().into())
    }
}

trait ToBytes32 {
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

/// Holds all the types necessary to interact with the `Simulation` struct
// TODO: Longer-term, we *may* not want to directly return internal representations of state from
// `Simulation` methods.  If/when that time comes, we will add the external-facing return values
// to this mod.  For now, however, we'll just directly return the internal state of the Simulation.
// (eg. a `Simulation.get_execution_environment_state` will return an internal `Root` object,
// instead of the more generic `[u8; 32]`)
pub mod args {
    // TODO: can remove this??
    use serde::{Deserialize, Serialize};
    use std::convert::{TryFrom, TryInto};
    use super::*;

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

    // Interface structs

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ExecutionEnvironment {
        #[serde(with = "super::base64_arr")]
        pub initial_state: [u8; 32],

        #[serde(with = "super::base64_vec")]
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

        pub execution_environment_states: Vec<[u8; 32]>,
    }

    // Conversions to/from interface structs <--> internal structs

    impl<T: EthSpec> From<super::ExecutionEnvironment<T>> for ExecutionEnvironment {
        fn from(value: super::ExecutionEnvironment<T>) -> Self {
            let initial_state: [u8; 32] = value.initial_state.into();
            let wasm_code: Vec<u8> = value.wasm_code.into();
            Self {
                initial_state,
                wasm_code,
            }
        }
    }
    impl<T: EthSpec> TryFrom<ExecutionEnvironment> for super::ExecutionEnvironment<T> {
        type Error = crate::Error;
        fn try_from(value: ExecutionEnvironment) -> Result<Self, Self::Error> {
            let initial_state: super::Root = Root::from(value.initial_state);
            // TODO(gregt): Switch this to wrap the underlying error
            let wasm_code = VariableList::new(value.wasm_code).map_err(|_| Error::MaxLengthExceeded {
                what: format!("wasm_code"),
            })?;

            Ok(Self {
                initial_state,
                wasm_code,
            })
        }
    }

    impl From<super::ShardTransaction> for ShardTransaction {
        fn from(value: super::ShardTransaction) -> Self {
            let data: Vec<u8> = value.data.into();
            let ee_index: u64 = value.ee_index.into();
            Self {
                data,
                ee_index,
            }
        }
    }
    impl TryFrom<ShardTransaction> for super::ShardTransaction {
        type Error = crate::Error;
        fn try_from(value: ShardTransaction) -> Result<Self, Self::Error> {
            let ee_index = value.ee_index.into();
            // TODO(gregt): Switch this to wrap the underlying error
            let data = VariableList::new(value.data).map_err(|_| Error::MaxLengthExceeded {
                what: format!("data"),
            })?;
            Ok(Self {
                data,
                ee_index,
            })
        }
    }

    impl<T: EthSpec> From<super::ShardBlock<T>> for ShardBlock {
        fn from(value: super::ShardBlock<T>) -> Self {
            let transactions: Vec<ShardTransaction> = value.transactions.into_iter().map(|t| -> ShardTransaction {
                t.clone().into()
            }).collect();
            Self {
                transactions,
            }
        }
    }
    impl<T: EthSpec> TryFrom<ShardBlock> for super::ShardBlock<T> {
        type Error = crate::Error;
        fn try_from(value: ShardBlock) -> Result<Self, Self::Error> {
            let mut transactions: Vec<super::ShardTransaction> = Vec::new();
            for t in value.transactions.into_iter() {
                let transaction = t.try_into()?;
                transactions.push(transaction);
            }
            // TODO(gregt): Switch this to wrap the underlying error
            let transactions = VariableList::new(transactions).map_err(|_| Error::MaxLengthExceeded {
                what: format!("transactions per shard block"),
            })?;
            Ok(Self {
                transactions,
            })
        }
    }

    impl<T: EthSpec> From<super::ShardState<T>> for ShardState {
        fn from(value: super::ShardState<T>) -> Self {
            let execution_environment_states: Vec<[u8; 32]> = value.execution_environment_states.into_iter().map(|t| -> [u8; 32] {
                t.0
            }).collect();
            Self {
                execution_environment_states,
            }
        }
    }
    impl<T: EthSpec> TryFrom<ShardState> for super::ShardState<T> {
        type Error = crate::Error;
        fn try_from(value: ShardState) -> Result<Self, Self::Error> {
            let execution_environment_states: Vec<super::Root>  = value.execution_environment_states.into_iter().map(|t| -> Root {
                Root::from(t)
            }).collect();
            // TODO(gregt): Switch this to wrap the underlying error
            let execution_environment_states = VariableList::new(execution_environment_states).map_err(|_| Error::MaxLengthExceeded {
                what: format!("execution environment states"),
            })?;
            Ok(Self {
                execution_environment_states,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::FromHex;
    use std::convert::TryFrom;
    use typenum::Unsigned;
    use types::eth_spec::MainnetEthSpec;

    #[test]
    fn simulation_new() {
        let simulation: Simulation<MainnetEthSpec> = Simulation::new();
        let max_shards = <MainnetEthSpec as EthSpec>::MaxShards::to_usize();
        // Should have MaxShards shard states
        assert_eq!(
            simulation.store.current_beacon_state.shard_states.len(),
            max_shards
        );
        // Should have no ees initially
        assert_eq!(
            simulation
                .store
                .current_beacon_state
                .execution_environments
                .len(),
            0
        );
        // Should have no ee states initially
        for i in 0..max_shards {
            let shard_state = simulation
                .store
                .current_beacon_state
                .shard_states
                .get(i)
                .unwrap();
            assert_eq!(shard_state.execution_environment_states.len(), 0);
        }
        // Should have MaxShards shards, but no shard blocks
        assert_eq!(simulation.store.shard_blocks_by_shard.len(), max_shards);
        for i in 0..max_shards {
            let shard_blocks_for_shard_i = simulation
                .store
                .shard_blocks_by_shard
                .get(&Shard::new(i as u64))
                .unwrap();
            assert_eq!(shard_blocks_for_shard_i.len(), 0);
        }
    }

    #[test]
    fn can_create_and_get_ee() {
        let mut simulation: Simulation<MainnetEthSpec> = Simulation::new();

        // Set up args::CreateExecutionEnvironment
        let mut initial_state: [u8; 32] = [0; 32];
        initial_state[5] = 1;
        let example_wasm_code: &[u8] = include_bytes!("../tests/do_nothing.wasm");
        let example_wasm_code2: &[u8] = include_bytes!("../tests/phase2_bazaar.wasm");

        let interface_ee = args::ExecutionEnvironment {
            initial_state,
            wasm_code: example_wasm_code.to_vec(),
        };
        let create_ee_args = args::CreateExecutionEnvironment {
            ee: interface_ee,
        };

        let interface_ee2 = args::ExecutionEnvironment {
            initial_state: initial_state.clone(),
            wasm_code: example_wasm_code2.to_vec(),
        };
        let create_ee_args2 = args::CreateExecutionEnvironment {
            ee: interface_ee2,
        };

        // Calling create_execution_environment repeatedly should return an increasing EE index
        let ee_index = simulation
            .create_execution_environment(create_ee_args)
            .unwrap();
        assert_eq!(ee_index, 0);
        let ee_index2 = simulation
            .create_execution_environment(create_ee_args2)
            .unwrap();
        assert_eq!(ee_index2, 1);

        // Set up args::GetExecutionEnvironment
        let get_ee_args = args::GetExecutionEnvironment {
            ee_index,
        };
        let get_ee_args2 = args::GetExecutionEnvironment {
            ee_index: ee_index2,
        };

        // Make sure the retrieved EEs have the same wasm code originally passed in
        let ee = simulation.get_execution_environment(get_ee_args).unwrap();
        let ee2 = simulation.get_execution_environment(get_ee_args2).unwrap();
        assert_eq!(ee.wasm_code, example_wasm_code.to_vec());
        assert_eq!(ee2.wasm_code, example_wasm_code2.to_vec());

        // Make sure the EEs have the correct initial_state specified for every shard
        let max_shards = <MainnetEthSpec as EthSpec>::MaxShards::to_usize();
        for i in 0..max_shards as u64 {
            let get_ee_state_args = args::GetExecutionEnvironmentState {
                ee_index,
                shard_index: i,
            };
            let ee_state = simulation
                .get_execution_environment_state(get_ee_state_args)
                .unwrap();
            assert_eq!(ee_state, initial_state);
        }
    }

    fn test_block_with_single_transaction(
        wasm_code: &[u8],
        initial_state: [u8; 32],
        data: Vec<u8>,
        expected_post_state: [u8; 32],
        shard_index: u64,
    ) -> (
        Simulation<MainnetEthSpec>,
        args::ShardTransaction,
        ShardSlot,
        EeIndex,
    ) {
        let mut simulation: Simulation<MainnetEthSpec> = Simulation::new();

        // Create EE with the specified code and initial state
        let ee = args::ExecutionEnvironment {
            initial_state,
            wasm_code: wasm_code.to_vec(),
        };
        let create_ee_args = args::CreateExecutionEnvironment {
            ee,
        };
        let ee_index = simulation
            .create_execution_environment(create_ee_args)
            .unwrap();
        assert_eq!(ee_index, 0);

        // Set up a shard transaction with the specified data
        let shard_transaction = args::ShardTransaction {
            data,
            ee_index,
        };
        let shard_transaction_copy = shard_transaction.clone();

        // Create a shard block with the one transaction in it
        let shard_block = args::ShardBlock {
            transactions: vec![shard_transaction],
        };
        let create_shard_block_args = args::CreateShardBlock {
            shard_index,
            shard_block,
        };
        // This creates the block and runs all the transactions inside it
        let shard_slot_index = simulation
            .create_shard_block(create_shard_block_args)
            .unwrap();

        // Get back the EE state to make sure it matches the expected_post_state
        let get_ee_state_args = args::GetExecutionEnvironmentState { ee_index, shard_index };
        let ee_post_state = simulation
            .get_execution_environment_state(get_ee_state_args)
            .unwrap();
        assert_eq!(
            ee_post_state, expected_post_state,
            "actual post state root should match expected post state root"
        );

        (simulation, shard_transaction_copy, ShardSlot::new(shard_slot_index), EeIndex::new(ee_index))
    }

    #[test]
    fn run_scout_helloworld_and_get_shard_block_and_state() {
        let initial_state = [0; 32];
        let expected_post_state = [0; 32];
        let data: Vec<u8> = Vec::new();
        let shard_index = 0;
        let (simulation, shard_transaction, shard_slot, ee_index) =
            test_block_with_single_transaction(
                include_bytes!("../tests/phase2_helloworld.wasm"),
                initial_state,
                data,
                expected_post_state,
                shard_index,
            );

        let ee_index: u64 = ee_index.into();

        // Test that GetShardBlock is working as expected
        let shard_slot_index: u64 = shard_slot.into();
        let get_shard_block_args = args::GetShardBlock { shard_index, shard_slot_index };
        let shard_block = simulation.get_shard_block(get_shard_block_args).unwrap();

        // Make sure the transaction on the retrieved block matches the transaction
        // on the created block
        assert_eq!(
            shard_block.transactions.get(0).unwrap().clone(),
            shard_transaction
        );

        // Test that GetShardState is working as expected
        let get_shard_state_args = args::GetShardState { shard_index };
        let shard_state = simulation.get_shard_state(get_shard_state_args).unwrap();
        let ee_index = ee_index as usize;
        let ee_state = shard_state
            .execution_environment_states
            .get(ee_index)
            .unwrap();
        assert_eq!(ee_state, &expected_post_state);
    }
    #[test]
    fn run_scout_bazaar_test() {
        let initial_state = "22ea9b045f8792170b45ec629c98e1b92bc6a19cd8d0e9f37baaadf2564142f4";
        let initial_state = Vec::from_hex(initial_state).unwrap().to_bytes32().unwrap();
        let expected_post_state = "29505fd952857b5766c759bcb4af58eb8df5a91043540c1398dd987a503127fc";
        let expected_post_state = Vec::from_hex(expected_post_state).unwrap().to_bytes32().unwrap();
        let data: Vec<u8> = Vec::from_hex("5c0000005000000001000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000001010101010101010101010101010101010101010101010101010101010101010400000000000000").unwrap();
        let shard_index = 0;
        let (_simulation, _shard_transaction, _shard_slot, _ee_index) =
            test_block_with_single_transaction(
                include_bytes!("../tests/phase2_bazaar.wasm"),
                initial_state,
                data,
                expected_post_state,
                shard_index,
            );
    }
}
