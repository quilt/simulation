use crate::store::Store;
use crate::{Result, Error, WhatBound};
use ewasm::{Execute, RootRuntime};
use ssz_types::{VariableList};
use types::eth_spec::EthSpec;
use types::execution_environment::ExecutionEnvironment;
use types::shard_block::ShardBlock;
use types::shard_state::ShardState;
use types::shard_transaction::ShardTransaction;
use types::slot_epoch_root::{EEIndex, Root, Shard, ShardSlot};

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
    pub fn create_execution_environment(&mut self, a: args::CreateExecutionEnvironment) -> Result<EEIndex> {
        // Create EE struct from args
        let vl_wasm_code = VariableList::new(a.wasm_code).map_err(|_| {
            Error::MaxLengthExceeded {
                what: format!("wasm_code"),
            }
        })?;
        let ee = ExecutionEnvironment {
            initial_state: a.initial_state,
            wasm_code: vl_wasm_code,
        };

        // Add EE code to beacon chain
        self.store.current_beacon_state.execution_environments.push(ee).map_err(|_| {
            Error::MaxLengthExceeded {
                what: format!("number of execution environments"),
            }
        })?;

        // For each shard, add the initial state to the shard
        for shard_state in self.store.current_beacon_state.shard_states.iter_mut() {
            // Set the initial state of the EE on each ShardState
            shard_state.execution_environment_states.push(a.initial_state.clone()).map_err(|_| {
                Error::MaxLengthExceeded {
                    what: format!("number of execution environment states"),
                }
            })?;

            // Each shard should have the same # of ee states as there are EEs
            assert_eq!(shard_state.execution_environment_states.len(), self.store.current_beacon_state.execution_environments.len());
        }

        let ee_index = self.store.current_beacon_state.execution_environments.len() - 1;
        Ok(EEIndex::new(ee_index as u64))
    }

    /// Add a new shard block containing a list of transactions that need to be executed
    /// Execute all transactions on the appropriate shards / EEs, return ShardBlock index
    pub fn create_shard_block(&mut self, a: args::CreateShardBlock) -> Result<ShardSlot> {
        // Get the specified ShardState (if it exists)
        let shard_index: usize = a.shard.into();
        let mut shard_state = self.store.current_beacon_state.shard_states.get_mut(shard_index).ok_or(Error::OutOfBounds {
            what: WhatBound::Shard,
            index: shard_index,
        })?;

        // Create the shard block from args
        let transactions = VariableList::new(a.shard_transactions).map_err(|_| {
            Error::MaxLengthExceeded {
                what: format!("number of shard transactions per block"),
            }
        })?;
        let shard_block = ShardBlock {
            transactions,
        };

        // Execute transactions and update shard state for all transactions
        for transaction in shard_block.transactions.iter() {
            // Get the specified EE (if it exists)
            let ee_index: usize = transaction.ee_index.into();
            let execution_environment = self.store.current_beacon_state.execution_environments.get(ee_index).ok_or(Error::OutOfBounds {
                what: WhatBound::ExecutionEnvironment,
                index: ee_index,
            })?;

            // Get the current EE state
            let pre_state = shard_state.execution_environment_states.get(ee_index).ok_or(Error::OutOfBounds {
                what: WhatBound::ExecutionEnvironmentState,
                index: ee_index,
            })?;

            // Create a new runtime with the EE code, transaction data, and pre state root
            let wasm_code: Vec<u8> = execution_environment.wasm_code.clone().into();
            let data: Vec<u8> = transaction.data.clone().into();
            let pre_state: [u8; 32] = pre_state.clone().into();
            let mut runtime = RootRuntime::new(&wasm_code, &data, pre_state);
            let post_root = runtime.execute();
            drop(runtime);

            // Update shard state with new root
            shard_state.execution_environment_states[ee_index] = Root::from(post_root);

        }

        // Add shard block to store for later access
        let shard_blocks_for_shard = self.store.shard_blocks_by_shard.get_mut(&a.shard).ok_or(Error::OutOfBounds {
            index: shard_index,
            what: WhatBound::Shard,
        })?;
        shard_blocks_for_shard.push(shard_block);

        // Return the slot of the newly added shard block
        Ok(ShardSlot::new((shard_blocks_for_shard.len() -1 ) as u64))
    }

    /// Get an EE that was previously added
    pub fn get_execution_environment(
        &self,
        a: args::GetExecutionEnvironment,
    ) -> Result<ExecutionEnvironment> {
        let ee_index: usize = a.ee_index.into();
        let ee = self.store.current_beacon_state.execution_environments.get(ee_index).ok_or(Error::OutOfBounds {
            what: WhatBound::ExecutionEnvironment,
            index: ee_index,
        })?;
        Ok(ee.clone())
    }

    /// Get the current state of an execution environment on a shard
    pub fn get_execution_environment_state(&self, a: args::GetExecutionEnvironmentState) -> Result<Root> {
        let ee_index: usize = a.ee_index.into();
        let shard_index: usize = a.shard.into();
        let shard_state = self.store.current_beacon_state.shard_states.get(shard_index).ok_or(Error::OutOfBounds {
            what: WhatBound::Shard,
            index: shard_index,
        })?;
        let ee_state_root = shard_state.execution_environment_states.get(ee_index).ok_or(Error::OutOfBounds {
            what: WhatBound::ExecutionEnvironmentState,
            index: ee_index,
        })?;
        Ok(ee_state_root.clone())
    }

    /// Get a shard block that was previously added
    pub fn get_shard_block(&self, a: args::GetShardBlock) -> Result<ShardBlock> {
        unimplemented!();
    }

    /// Get the specified ShardState, will contain EE states
    pub fn get_shard_state(&self, a: args::GetShardState) -> ShardState<T> {
        unimplemented!();
    }
}

/// Holds all the types necessary to interact with the `Simulation` struct
// TODO: Longer-term, we *may* not want to directly return internal representations of state from
// `Simulation` methods.  If/when that time comes, we will add the external-facing return values
// to this mod.  For now, however, we'll just directly return the internal state of the Simulation.
// (eg. a `Simulation.get_execution_environment_state` will return an internal `Root` object,
// instead of the more generic `[u8; 32]`)
mod args {
    use super::*;

    pub struct CreateExecutionEnvironment {
        pub initial_state: Root,
        pub wasm_code: Vec<u8>,
    }
    pub struct CreateShardBlock {
        pub shard: Shard,
        pub shard_transactions: Vec<ShardTransaction>,
    }
    pub struct GetExecutionEnvironment {
        pub ee_index: EEIndex,
    }
    pub struct GetExecutionEnvironmentState {
        pub ee_index: EEIndex,
        pub shard: Shard,
    }
    pub struct GetShardBlock {
        pub shard: Shard,
        pub shard_slot: ShardSlot,
    }
    pub struct GetShardState {
        pub shard: Shard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_new() {

    }
}
