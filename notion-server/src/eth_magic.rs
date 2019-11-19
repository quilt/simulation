use std::default;
use std::collections::{HashMap};

const NUM_SHARDS: usize = 32;

#[derive(Debug)]
pub struct EthMagicManager {
    beacon_chain: BeaconChain,
    shard_chains: [ShardChain; NUM_SHARDS],
}

impl EthMagicManager {
    pub fn new() -> Self {
        Self {
            beacon_chain: BeaconChain::new(),
            shard_chains: Default::default(),
        }
    }

    // Creates a new execution environment on the BeaconChain and returns the
    // index of the created execution environment
    pub fn add_new_execution_environment(&self, ee: CreateEeArgs) -> u32 {
        // Create EE
        // Longer-term, accept a config here
        unimplemented!()
    }

    // Returns the index of the newly added shard chain
    pub fn add_new_shard_chain(&self, sc: CreateShardChainArgs) -> u32 {
        // Create new shard chain
        // Longer-term, accept a config here
        unimplemented!()
    }

    pub fn append_shard_block(&self, shard_index: usize, block: CreateShardBlockArgs) {
        // Worth noting that in a real-world use case "sub-transactions" may be merged
        // into one "combined" transaction before being executed / committed to a block

        unimplemented!()

//        let transactions = block.transactions
//
//        for transaction in block.transactions {
//            // This executes everything and presumably also updates the EE State on the shard
//            let ee = transaction.execution_environment;
//            let input_data = transaction.data;
//
//            let code = self.beacon_chain.get(ee);
//            let runtime = RootRuntime::new(&code, shard_ee_state_or_something_similar);
//            runtime.execute(input_data);
//        }

    }

    /* Getter methods still needed
        Beacon State
        Shard State
        Transactions (do we want to store EE state per shard before / after each transaction?

    */
}

#[derive(Debug, Default)]
pub struct CreateEeArgs {

}

#[derive(Debug, Default)]
pub struct CreateShardChainArgs {

}

#[derive(Debug, Default)]
pub struct CreateShardBlockArgs {

}


#[derive(Debug, Default)]
struct BeaconChain {
    // There are an unbounded number of EEs that can "exist" on the beacon chain
    execution_environments: Vec<ExecutionEnvironment>,
}

impl BeaconChain {
    fn new() -> Self {
        return Self {
            execution_environments: Vec::new(),
        };
    }

    // Adds a new execution environment, returns the index of that new EE
    fn add_execution_environment(&mut self, execution_environment: ExecutionEnvironment) -> EeIndex {
        self.execution_environments.push(execution_environment);
        EeIndex((self.execution_environments.len() - 1) as u32)
    }
}

#[derive(Default, Debug)]
struct ShardChain {
    // Longer-term, we may need to worry about rollbacks / "staging" changes to these before committing
    // (maybe not, but worth keeping in mind that could be an issue)
    execution_environment_state: HashMap<EeIndex, ExecutionEnvironmentState>,
    shard_blocks: Vec<ShardBlock>,
}

impl ShardChain {
    fn new() -> Self {
        Self {
            execution_environment_state: HashMap::new(),
            shard_blocks: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Hash, Clone, Copy, Eq, PartialEq)]
struct EeIndex(u32);

// The execution environment data that lives on the beacon chain
// Does NOT include shard-specific EE state
#[derive(Debug)]
struct ExecutionEnvironment {
    wasm_code: Vec<u8>,
}

// The execution environment state that lives on each shard chain
#[derive(Debug)]
struct ExecutionEnvironmentState {
    data: [u8; 32],
}

#[derive(Debug)]
struct ShardBlock {
    transactions: Vec<ShardTransaction>,
}

impl ShardBlock {
    fn new(transactions: Vec<ShardTransaction>) -> Self {
        Self {
            transactions,
        }
    }
    fn add_transaction(&mut self, transaction: ShardTransaction) {
        self.transactions.push(transaction);
    }
}

#[derive(Default, Debug)]
struct ShardTransaction {
    data: Vec<u8>,
    ee_index: EeIndex,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn basics() {
        let eth_magic = EthMagicManager::new();
    }
}
