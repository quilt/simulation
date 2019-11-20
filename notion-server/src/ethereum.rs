use std::collections::HashMap;

#[derive(Debug)]
pub struct EthereumSimulation {
    beacon_chain: BeaconChain,
    shard_chains: Vec<ShardChain>,
}

impl EthereumSimulation {
    pub fn new() -> Self {
        Self {
            beacon_chain: BeaconChain::new(),
            shard_chains: Vec::new(),
        }
    }

    /// Creates a new execution environment on the BeaconChain and returns the
    /// index of the created execution environment
    pub fn create_execution_environment(
        &mut self,
        ee_args: args::CreateExecutionEnvironment,
    ) -> u32 {
        let execution_environment = ExecutionEnvironment {
            wasm_code: ee_args.wasm_code,
        };
        let EeIndex(ee_index) = self
            .beacon_chain
            .add_execution_environment(execution_environment);
        ee_index
    }

    /// Returns the index of the newly added shard chain
    /// Longer-term, can accept a config here
    pub fn create_shard_chain(&mut self, sc_args: args::CreateShardChain) -> u32 {
        let shard_chain = ShardChain::new();
        self.shard_chains.push(shard_chain);
        (self.shard_chains.len() - 1) as u32
    }

    /// Creates a new shard block and returns the
    /// index of the created shard block
    pub fn create_shard_block(
        &mut self,
        shard_index: u32,
        block_args: args::CreateShardBlock,
    ) -> u32 {
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

pub mod args {
    #[derive(Debug, Default)]
    pub struct CreateExecutionEnvironment {
        // TODO @gregt: switch this to be base64 encoded in the next diff
        // Also add conversion function using From to go from this to an internal EE representation (and same for structs below)
        // (not adding here to avoid huge PRs with too many purposes)
        pub wasm_code: Vec<u8>,
    }

    #[derive(Debug, Default)]
    pub struct CreateShardChain {}

    #[derive(Debug, Default)]
    pub struct CreateShardBlock {}
}

#[derive(Debug, Default)]
struct BeaconChain {
    // There are an unbounded number of EEs that can "exist" on the beacon chain
    execution_environments: Vec<ExecutionEnvironment>,
}

impl BeaconChain {
    fn new() -> Self {
        Self {
            execution_environments: Vec::new(),
        }
    }

    // Adds a new execution environment, returns the index of that new EE
    fn add_execution_environment(
        &mut self,
        execution_environment: ExecutionEnvironment,
    ) -> EeIndex {
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
        Self { transactions }
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
