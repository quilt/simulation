use crate::errors::{Result, WhatBound};
use crate::store::Store;
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
    pub fn create_execution_environment(&mut self, a: args::CreateExecutionEnvironment) -> EEIndex {
        // Add EE code to beacon chain
        let ee = ExecutionEnvironment {
            initial_state: a.initial_state,
            wasm_code: VariableList::new(a.wasm_code),
        };
        self.store.current_beacon_state.execution_environments.push(ee).unwrap();

        // For each shard, add the initial state
//        self.store.current_beacon_state.shard_states


        EEIndex(self.store.current_beacon_state.execution_environments.len() as u64)
    }

    /// Add a new shard block containing a list of transactions that need to be executed
    /// Execute all transactions on the appropriate shards / EEs, return ShardBlock index
    pub fn create_shard_block(&self, args: args::CreateShardBlock) -> ShardSlot {
        unimplemented!();
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
        shard: Shard,
        shard_transactions: Vec<ShardTransaction>,
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
