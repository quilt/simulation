use base64::decode;
use std::default;
use std::collections::{HashMap};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct EthMagicManager {
    beacon_chain: BeaconChain,
    shard_chains: Vec<ShardChain>,
}

impl EthMagicManager {
    pub fn new() -> Self {
        Self {
            beacon_chain: BeaconChain::new(),
            shard_chains: Vec::new(),
        }
    }

    // Creates a new execution environment on the BeaconChain and returns the
    // index of the created execution environment
    pub fn add_new_execution_environment(&mut self, ee: CreateEeArgs) -> u32 {
        let wasm_code = decode(&ee.base64_wasm_code).unwrap();
        let execution_environment = ExecutionEnvironment {
            wasm_code,
        };
        self.beacon_chain.execution_environments.push(execution_environment);
        (self.beacon_chain.execution_environments.len() - 1) as u32
    }

    // Returns the index of the newly added shard chain
    // Longer-term, can accept a config here
    pub fn add_new_shard_chain(&mut self) -> u32 {
        let shard_chain = ShardChain::new();
        self.shard_chains.push(shard_chain);
        (self.shard_chains.len() - 1) as u32
    }

    // Returns the index of the newly added shard block
    pub fn append_shard_block(&mut self, shard_index: usize, block: CreateShardBlockArgs) -> u32 {
        // Worth noting that in a real-world use case "sub-transactions" may be merged
        // into one "combined" transaction before being executed / committed to a block
//        let &mut shard_chain = &mut self.shard_chains[shard_index];
//
//        // Sam to implement: create the transactions and the shard block and run the transactions
//        let shard_block = ShardBlock::new(Vec::new());
//
//        shard_chain.shard_blocks.push(shard_block);
//        (shard_chain.shard_blocks.len() - 1) as u32

        unimplemented!();

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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CreateEeArgs {
    base64_wasm_code: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
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
