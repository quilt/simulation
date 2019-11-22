#![allow(unused)]

use snafu::{OptionExt, Snafu};

use std::collections::HashMap;

use tokio::sync::mpsc::{channel, Receiver, Sender};

/// Shorthand for result types returned from the Simulation simulation.
pub type Result<V, E = Error> = std::result::Result<V, E>;

/// Errors arising from the simulation.
#[derive(Debug, Snafu)]
pub enum Error {
    /// Operation was cancelled because the simulation is shutting down.
    Terminated,
}

#[derive(Debug)]
enum Operation {
    CreateExecutionEnvironment(args::CreateExecutionEnvironment, Sender<EeIndex>),
    CreateShardChain(args::CreateShardChain, Sender<u32>),
    CreateShardBlock(args::CreateShardBlock, Sender<u32>),
    GetShardBlock(args::GetShardBlock, Sender<ShardBlock>),
}

#[derive(Debug, Clone)]
pub struct Handle(Sender<Operation>);

impl Handle {
    pub async fn create_shard_chain(&mut self, arg: args::CreateShardChain) -> Result<u32> {
        let (sender, mut receiver) = channel(1);

        self.0.send(Operation::CreateShardChain(arg, sender)).await;

        receiver.recv().await.context(Terminated)
    }

    pub async fn create_shard_block(&mut self, arg: args::CreateShardBlock) -> Result<u32> {
        let (sender, mut receiver) = channel(1);

        self.0.send(Operation::CreateShardBlock(arg, sender)).await;

        receiver.recv().await.context(Terminated)
    }

    pub async fn create_execution_environment(
        &mut self,
        arg: args::CreateExecutionEnvironment,
    ) -> Result<EeIndex> {
        let (sender, mut receiver) = channel(1);

        self.0
            .send(Operation::CreateExecutionEnvironment(arg, sender))
            .await;

        receiver.recv().await.context(Terminated)
    }

    pub async fn shard_block(&mut self, arg: args::GetShardBlock) -> Result<ShardBlock> {
        let (sender, mut receiver) = channel(1);

        self.0.send(Operation::GetShardBlock(arg, sender)).await;

        receiver.recv().await.context(Terminated)
    }
}

#[derive(Debug)]
pub struct Simulation {
    beacon_chain: BeaconChain,
    shard_chains: Vec<ShardChain>,
    receiver: Receiver<Operation>,
}

impl Simulation {
    pub fn new() -> (Self, Handle) {
        let (sender, receiver) = channel(1);

        let sim = Self {
            beacon_chain: BeaconChain::new(),
            shard_chains: Vec::new(),
            receiver,
        };

        let handle = Handle(sender);

        (sim, handle)
    }

    pub async fn run(mut self) -> Result<()> {
        eprintln!("Simulation Running: {:?}", std::thread::current().id());
        while let Some(op) = self.receiver.recv().await {
            match op {
                Operation::CreateExecutionEnvironment(args, mut reply) => {
                    let res = self.create_execution_environment(args);
                    reply.send(EeIndex(res)).await;
                }
                Operation::CreateShardBlock(args, mut reply) => {
                    let res = self.create_shard_block(args);
                    reply.send(res).await;
                }
                Operation::CreateShardChain(args, mut reply) => {
                    let res = self.create_shard_chain(args);
                    reply.send(res).await;
                }
                Operation::GetShardBlock(args, mut reply) => {
                    let res = self.get_shard_block(args);
                    reply.send(res).await;
                }
            }
        }

        Ok(())
    }

    /// Creates a new execution environment on the BeaconChain and returns the
    /// index of the created execution environment
    fn create_execution_environment(&mut self, ee_args: args::CreateExecutionEnvironment) -> u32 {
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
    fn create_shard_chain(&mut self, _: args::CreateShardChain) -> u32 {
        let shard_chain = ShardChain::new();
        self.shard_chains.push(shard_chain);
        (self.shard_chains.len() - 1) as u32
    }

    /// Creates a new shard block and returns the
    /// index of the created shard block
    fn create_shard_block(&mut self, _: args::CreateShardBlock) -> u32 {
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

    fn get_shard_block(&self, _: args::GetShardBlock) -> ShardBlock {
        unimplemented!()
    }
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
    pub struct CreateShardBlock {
        shard_index: u32,
    }

    #[derive(Debug, Default)]
    pub struct GetShardBlock {
        shard_index: u32,
        block_number: u32,
    }
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
pub struct EeIndex(u32);

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
pub struct ShardBlock {
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
        Simulation::new();
    }
}
