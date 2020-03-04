use crate::store::Store;
use crate::{ArgsError, Error, Result, WhatBound};
use ewasm::{Execute, RootRuntime};
use simulation_args;
use snafu::ResultExt;
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
        a: simulation_args::CreateExecutionEnvironment,
    ) -> Result<u64> {
        // Create internal EE struct from args
        let ee = ExecutionEnvironment::try_from(a.ee).context(
            ArgsError,
        )?;
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
    pub fn create_shard_block(&mut self, a: simulation_args::CreateShardBlock) -> Result<u64> {
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
        let shard_block: ShardBlock<T> = ShardBlock::try_from(a.shard_block).context(
            ArgsError,
        )?;;

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
        a: simulation_args::GetExecutionEnvironment,
    ) -> Result<simulation_args::ExecutionEnvironment> {
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
        a: simulation_args::GetExecutionEnvironmentState,
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
    pub fn get_shard_block(&self, a: simulation_args::GetShardBlock) -> Result<simulation_args::ShardBlock> {
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
    pub fn get_shard_state(&self, a: simulation_args::GetShardState) -> Result<simulation_args::ShardState> {
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

        let interface_ee = simulation_args::ExecutionEnvironment {
            initial_state,
            wasm_code: example_wasm_code.to_vec(),
        };
        let create_ee_args = simulation_args::CreateExecutionEnvironment {
            ee: interface_ee,
        };

        let interface_ee2 = simulation_args::ExecutionEnvironment {
            initial_state: initial_state.clone(),
            wasm_code: example_wasm_code2.to_vec(),
        };
        let create_ee_args2 = simulation_args::CreateExecutionEnvironment {
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

        // Set up simulation_args::GetExecutionEnvironment
        let get_ee_args = simulation_args::GetExecutionEnvironment {
            ee_index,
        };
        let get_ee_args2 = simulation_args::GetExecutionEnvironment {
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
            let get_ee_state_args = simulation_args::GetExecutionEnvironmentState {
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
        simulation_args::ShardTransaction,
        ShardSlot,
        EeIndex,
    ) {
        let mut simulation: Simulation<MainnetEthSpec> = Simulation::new();

        // Create EE with the specified code and initial state
        let ee = simulation_args::ExecutionEnvironment {
            initial_state,
            wasm_code: wasm_code.to_vec(),
        };
        let create_ee_args = simulation_args::CreateExecutionEnvironment {
            ee,
        };
        let ee_index = simulation
            .create_execution_environment(create_ee_args)
            .unwrap();
        assert_eq!(ee_index, 0);

        // Set up a shard transaction with the specified data
        let shard_transaction = simulation_args::ShardTransaction {
            data,
            ee_index,
        };
        let shard_transaction_copy = shard_transaction.clone();

        // Create a shard block with the one transaction in it
        let shard_block = simulation_args::ShardBlock {
            transactions: vec![shard_transaction],
        };
        let create_shard_block_args = simulation_args::CreateShardBlock {
            shard_index,
            shard_block,
        };
        // This creates the block and runs all the transactions inside it
        let shard_slot_index = simulation
            .create_shard_block(create_shard_block_args)
            .unwrap();

        // Get back the EE state to make sure it matches the expected_post_state
        let get_ee_state_args = simulation_args::GetExecutionEnvironmentState { ee_index, shard_index };
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
        let get_shard_block_args = simulation_args::GetShardBlock { shard_index, shard_slot_index };
        let shard_block = simulation.get_shard_block(get_shard_block_args).unwrap();

        // Make sure the transaction on the retrieved block matches the transaction
        // on the created block
        assert_eq!(
            shard_block.transactions.get(0).unwrap().clone(),
            shard_transaction
        );

        // Test that GetShardState is working as expected
        let get_shard_state_args = simulation_args::GetShardState { shard_index };
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
        use simulation_args::ToBytes32;

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
