use std::collections::HashMap;
use typenum::Unsigned;
use types::beacon_state::BeaconState;
use types::eth_spec::EthSpec;
use types::shard_block::ShardBlock;
use types::slot_epoch_root::Shard;

/// Contains arbitrary state stored by the simulation
/// This struct need not adhere to any official Eth2 spec -- it will store internal
/// simulation state in whatever manner is most convenient.
/// However, some types that ARE listed in a spec (eg. BeaconState) will still be spec-compliant
/// even if they happen to be stored in Store.
#[derive(Debug)]
pub struct Store<T>
where
    T: EthSpec,
{
    pub current_beacon_state: BeaconState<T>,

    // A mapping from shard to the shard blocks contained in the shard
    // HashMap<Shard, Vec<ShardBlock>> is used instead of Vec<Vec<ShardBlock>> because the former
    // is easier to read and immediately understand what is being stored.
    pub shard_blocks_by_shard: HashMap<Shard, Vec<ShardBlock<T>>>,
}

impl<T: EthSpec> Store<T> {
    pub fn new() -> Self {
        // Initialize shard blocks storage for all shards
        let mut shard_blocks_by_shard = HashMap::new();
        for shard in 0..T::MaxShards::to_u64() {
            let shard = Shard::new(shard);
            shard_blocks_by_shard.insert(shard, Vec::new());
        }
        Self {
            current_beacon_state: BeaconState::new(),
            shard_blocks_by_shard,
        }
    }
}
