#![allow(dead_code)] // TODO: Remove as the rest of the daemon takes shape.

use snafu::{Backtrace, ResultExt, Snafu};

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

/// Shorthand for result types returned from the Simulation simulation.
pub type Result<V, E = Error> = std::result::Result<V, E>;

/// Errors arising from the simulation.
#[derive(Debug, Snafu)]
pub enum Error {
    /// An input/output error, usually reported by the operating system.
    Io {
        /// Location of where the error occurred.
        backtrace: Backtrace,

        /// The underlying error as reported by the operating system.
        source: std::io::Error,
    },

    /// Attempted to join a simulation that isn't running.
    NotRunning,
}

#[derive(Debug)]
enum Status {
    Stopped,
    Started(JoinHandle<Result<()>>),
}

impl Status {
    pub fn is_started(&self) -> bool {
        match self {
            Status::Started(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
enum Operation {
    CreateExecutionEnvironment(args::CreateExecutionEnvironment, SyncSender<EeIndex>),
    CreateShardChain(args::CreateShardChain, SyncSender<u32>),
    CreateShardBlock(args::CreateShardBlock, SyncSender<u32>),
    GetShardBlock(args::GetShardBlock, SyncSender<ShardBlock>),
}

#[derive(Debug, Clone)]
pub struct Simulation {
    status: Arc<Mutex<Status>>,
    sender: Sender<Operation>,
}

impl Simulation {
    pub fn spawn() -> Result<Self> {
        let (op_send, op_recv) = mpsc::channel::<Operation>();

        let handle = thread::Builder::new()
            .name("ethereum".into())
            .spawn(move || Simulation::run(op_recv))
            .context(Io)?;

        let eth = Simulation {
            status: Arc::new(Mutex::new(Status::Started(handle))),
            sender: op_send,
        };

        Ok(eth)
    }

    fn status(&self) -> MutexGuard<Status> {
        self.status.lock().expect("ethereum status poisoned")
    }

    fn run(recv: Receiver<Operation>) -> Result<()> {
        let mut implementation = EthereumSimulation::new();

        while let Ok(op) = recv.recv() {
            match op {
                Operation::CreateExecutionEnvironment(args, reply) => {
                    let res = implementation.create_execution_environment(args);
                    reply.send(EeIndex(res)).ok();
                }
                Operation::CreateShardBlock(args, reply) => {
                    let res = implementation.create_shard_block(args);
                    reply.send(res).ok();
                }
                Operation::CreateShardChain(args, reply) => {
                    let res = implementation.create_shard_chain(args);
                    reply.send(res).ok();
                }
                Operation::GetShardBlock(args, reply) => {
                    let res = implementation.get_shard_block(args);
                    reply.send(res).ok();
                }
            }
        }

        Ok(())
    }

    pub fn create_execution_environment(&self, args: args::CreateExecutionEnvironment) -> EeIndex {
        let (reply_send, reply_recv) = mpsc::sync_channel(1);

        self.sender
            .send(Operation::CreateExecutionEnvironment(args, reply_send))
            .unwrap();

        reply_recv.recv().unwrap()
    }

    pub fn join(self) -> Result<()> {
        let handle = {
            let mut status = self.status();
            if !status.is_started() {
                return Err(Error::NotRunning);
            }

            let old = std::mem::replace(&mut *status, Status::Stopped);

            match old {
                Status::Started(handle) => handle,
                _ => unreachable!(),
            }
        };

        drop(self);
        handle.join().expect("simulation thread panicked")
    }
}

#[derive(Debug)]
struct EthereumSimulation {
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
    pub fn create_shard_chain(&mut self, _: args::CreateShardChain) -> u32 {
        let shard_chain = ShardChain::new();
        self.shard_chains.push(shard_chain);
        (self.shard_chains.len() - 1) as u32
    }

    pub fn get_shard_block(&self, _block_args: args::GetShardBlock) -> ShardBlock {
        unimplemented!()
    }

    /// Creates a new shard block and returns the
    /// index of the created shard block
    pub fn create_shard_block(&mut self, _block_args: args::CreateShardBlock) -> u32 {
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
        EthereumSimulation::new();
    }

    #[test]
    fn start_join() {
        let eth = Simulation::spawn().unwrap();
        eth.join().unwrap();
    }
}
