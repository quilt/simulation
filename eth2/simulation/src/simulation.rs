use crate::errors::{
    Result,
    WhatBound,
};
use crate::store::Store;
use types::eth_spec::{EthSpec};
use types::slot_epoch_root::{
    EEIndex,
    Root,
    Shard,
    ShardSlot,
};
use types::execution_environment::ExecutionEnvironment;
use types::shard_block::ShardBlock;
use types::shard_state::ShardState;
use types::shard_transaction::ShardTransaction;

pub struct Simulation<T>
where
    T: EthSpec,
{
    store: Store<T>,
}

impl<T: EthSpec> Simulation<T> {
    /// Add a new execution environment, return EE index
    fn create_execution_environment(a: args::CreateExecutionEnvironment) -> EEIndex {
        unimplemented!();
    }

    /// Add a new shard block containing a list of transactions that need to be executed
    /// Execute all transactions on the appropriate shards / EEs, return ShardBlock index
    fn create_shard_block(args: args::CreateShardBlock) -> ShardSlot {
        unimplemented!();
    }

    /// Get an EE that was previously added
    fn get_execution_environment(args: args::GetExecutionEnvironment) -> Result<ExecutionEnvironment> {
        unimplemented!();
    }

    /// Get the current state of an execution environment on a shard
    fn get_execution_environment_state(args: args::GetExecutionEnvironmentState) -> Result<Root> {
        unimplemented!();
    }

    /// Get a shard block that was previously added
    fn get_shard_block(args: args::GetShardBlock) -> Result<ShardBlock> {
        unimplemented!();
    }

    /// Get the specified ShardState, will contain EE states
    fn get_shard_state(args: args::GetShardState) -> ShardState {
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
        wasm_code: Vec<u8>,
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
