use crate::store::Store;
use crate::{Result, Error};
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
            shard_state.execution_environment_states.push(a.initial_state.clone());

            // Each shard should have the same # of ee states as there are EEs
            assert_eq!(shard_state.execution_environment_states.len(), self.store.current_beacon_state.execution_environments.len());
        }

        let ee_index = self.store.current_beacon_state.execution_environments.len() - 1;
        Ok(EEIndex::new(ee_index as u64))
    }

    /// Add a new shard block containing a list of transactions that need to be executed
    /// Execute all transactions on the appropriate shards / EEs, return ShardBlock index
    pub fn create_shard_block(&mut self, a: args::CreateShardBlock) -> Result<ShardSlot> {
        // Create shard block
        let transactions = VariableList::new(a.shard_transactions).map_err(|_| {
            Error::MaxLengthExceeded {
                what: format!("number of shard transactions per block"),
            }
        });
        let shard_block = ShardBlock {
            transactions,
        };

        // Execute transactions and update shard state for all transactions
        // TODO: Handle errors
//        for transaction in a.shard_transactions {
//            let execution_environment = self
//                .store.current_beacon_state
//                .execution_environments.get(transaction.ee_index)
//                .get(transaction.ee_index as usize)
//                .context(OutOfBounds {
//                    what: WhatBound::ExecutionEnvironment,
//                    index: transaction.ee_index as usize,
//                })?;
//            let code = &execution_environment.wasm_code;
//
//            let pre_state = shard_chain
//                .execution_environment_state
//                .get(&EeIndex(transaction.ee_index))
//                .unwrap_or(&execution_environment.initial_shard_state);
//            let data = base64::decode(&transaction.base64_encoded_data).context(Decode)?;
//            let mut runtime = RootRuntime::new(&code, &data, pre_state.data);
//            let post_root = runtime.execute();
//            drop(runtime);
//
//            shard_chain.execution_environment_state.insert(
//                EeIndex(transaction.ee_index),
//                ExecutionEnvironmentState { data: post_root },
//            );
//
//            transactions.push(ShardTransaction {
//                data,
//                ee_index: EeIndex(transaction.ee_index),
//            });
//        }

        // Update shard state for all transactions
        // Store transactions
        // TODO: Make this return error if shard index is out of bounds
//        self.store.shard_blocks_by_shard[a.shard].push(shard_block);

    }

    /// Get an EE that was previously added
    pub fn get_execution_environment(
        &self,
        args: args::GetExecutionEnvironment,
    ) -> Result<ExecutionEnvironment> {
        unimplemented!();
    }

    /// Get the current state of an execution environment on a shard
    pub fn get_execution_environment_state(&self, args: args::GetExecutionEnvironmentState) -> Result<Root> {
        unimplemented!();
    }

    /// Get a shard block that was previously added
    pub fn get_shard_block(&self, args: args::GetShardBlock) -> Result<ShardBlock> {
        unimplemented!();
    }

    /// Get the specified ShardState, will contain EE states
    pub fn get_shard_state(&self, args: args::GetShardState) -> ShardState<T> {
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
        ee_index: EEIndex,
    }
    pub struct GetExecutionEnvironmentState {
        ee_index: EEIndex,
        shard: Shard,
    }
    pub struct GetShardBlock {
        shard: Shard,
        shard_slot: ShardSlot,
    }
    pub struct GetShardState {
        shard: Shard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_new() {

    }
}
