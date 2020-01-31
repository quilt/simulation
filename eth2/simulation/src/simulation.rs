use types::slot_epoch_root::{
    EEIndex,
    Root,
    ShardSlot,
};
use types::execution_environment::ExecutionEnvironment;
use types::shard_block::ShardBlock;
use types::shard_state::ShardState;

struct Simulation {

}

impl Simulation {
    // Add a new execution environment, return EE index
    fn create_execution_environment(a: args::CreateExecutionEnvironment) -> EEIndex {
        unimplemented!();
    }

    // Add a new shard block containing a list of transactions that need to be executed
    // Execute all transactions on the appropriate shards / EEs, return ShardBlock index
    fn create_shard_block(args: args::CreateShardBlock) -> ShardSlot {
        unimplemented!();
    }

    // Get an EE that was previously added
    fn get_execution_environment(args: args::GetExecutionEnvironment) -> Option<ExecutionEnvironment> {
        unimplemented!();
    }

    // Get a shard block that was previously added
    fn get_shard_block(args: args::GetShardBlock) -> Option<ShardBlock> {
        unimplemented!();
    }

    // Get the specified ShardState, will contain EE states
    fn get_shard_state(args: args::GetShardState) -> ShardState {
        unimplemented!();
    }
}

/// Holds all the types necessary to interact with the `Simulation` struct
// TODO: Longer-term, we *may* not want to directly return internal representations of state from
// `Simulation` methods.  If/when that time comes, we will add the external-facing return values
// to this mod.  For now, however, we'll just directly return the internal state of the Simulation.
// (eg. a `Simulation.get_execution_environment_state` will return an internal `Root` object, instead
// of the more generic `[u8; 32]`)
mod args {
    pub struct CreateExecutionEnvironment {

    }
    pub struct CreateShardBlock {

    }
    pub struct GetExecutionEnvironment {

    }
    pub struct GetShardBlock {

    }
    pub struct GetShardState {

    }
}
