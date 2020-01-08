#![allow(unused)]

mod dispatch;

use base64;

pub use self::dispatch::{Dispatch, Handle};

use snafu::{Backtrace, OptionExt, ResultExt, Snafu};

use ewasm::{Execute, RootRuntime};

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

/// Shorthand for result types returned from the Simulation simulation.
pub type Result<V, E = Error> = std::result::Result<V, E>;

#[derive(Debug)]
pub enum WhatBound {
    BeaconBlock,
    ExecutionEnvironment,
    ShardChain,
    ShardBlock(u32),
}

impl fmt::Display for WhatBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WhatBound::BeaconBlock => write!(f, "beacon block"),
            WhatBound::ExecutionEnvironment => write!(f, "execution environment"),
            WhatBound::ShardChain => write!(f, "shard chain"),
            WhatBound::ShardBlock(shard) => write!(f, "block on shard {}", shard),
        }
    }
}

/// Errors arising from the simulation.
#[derive(Debug, Snafu)]
pub enum Error {
    Decode {
        backtrace: Backtrace,
        source: base64::DecodeError,
    },

    #[snafu(display("no {} exists at index: {}", what, index))]
    OutOfBounds {
        what: WhatBound,
        index: usize,
    },

    /// Operation was cancelled because the simulation is shutting down.
    Terminated,
    InvalidBytes32,
}

#[derive(Debug)]
pub struct Simulation {
    beacon_chain: BeaconChain,
    shard_chains: Vec<ShardChain>,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            beacon_chain: BeaconChain::new(),
            shard_chains: Vec::new(),
        }
    }

    pub fn simulation_state(&self) -> args::SimulationState {
        args::SimulationState {
            num_execution_environments: self.beacon_chain.execution_environments.len() as u32,
            num_shard_chains: self.shard_chains.len() as u32,
        }
    }

    pub fn create_beacon_block(
        &mut self,
        args: args::CreateBeaconBlock,
    ) -> Result<u32> {
        let beacon_block = BeaconBlock::from(args.beacon_block);
        let BeaconBlockIndex(beacon_block_index) = self
            .beacon_chain
            .add_beacon_block(beacon_block);
        Ok(beacon_block_index)
    }

    pub fn get_beacon_block(
        &self,
        args: args::GetBeaconBlock,
    ) -> Result<args::BeaconBlock> {
        let beacon_block = self
            .beacon_chain
            .beacon_blocks
            .get(args.beacon_block_index as usize)
            .context(OutOfBounds {
                what: WhatBound::BeaconBlock,
                index: args.beacon_block_index as usize,
            })?;

        Ok(args::BeaconBlock::from(beacon_block))
    }

    /// Creates a new execution environment on the BeaconChain and returns the
    /// index of the created execution environment
    pub fn create_execution_environment(
        &mut self,
        args: args::CreateExecutionEnvironment,
    ) -> Result<u32> {
        let execution_environment = ExecutionEnvironment::from(args.execution_environment);
        let EeIndex(ee_index) = self
            .beacon_chain
            .add_execution_environment(execution_environment);
        Ok(ee_index)
    }

    pub fn get_execution_environment(
        &self,
        args: args::GetExecutionEnvironment,
    ) -> Result<args::ExecutionEnvironment> {
        let execution_environment = self
            .beacon_chain
            .execution_environments
            .get(args.execution_environment_index as usize)
            .context(OutOfBounds {
                what: WhatBound::ExecutionEnvironment,
                index: args.execution_environment_index as usize,
            })?;

        Ok(args::ExecutionEnvironment::from(execution_environment))
    }

    /// Returns the index of the newly added shard chain
    /// Longer-term, can accept a config here
    pub fn create_shard_chain(&mut self, _: args::CreateShardChain) -> u32 {
        let shard_chain = ShardChain::new();
        self.shard_chains.push(shard_chain);
        (self.shard_chains.len() - 1) as u32
    }

    /// Creates a new shard block and returns the
    /// index of the created shard block
    fn create_shard_block(&mut self, args: args::CreateShardBlock) -> Result<u32> {
        let mut shard_chain = self
            .shard_chains
            .get_mut(args.shard_chain_index as usize)
            .context(OutOfBounds {
                what: WhatBound::ShardChain,
                index: args.shard_chain_index as usize,
            })?;

        let mut transactions = Vec::with_capacity(args.shard_block.transactions.len());

        for transaction in args.shard_block.transactions {
            let execution_environment = self
                .beacon_chain
                .execution_environments
                .get(transaction.ee_index as usize)
                .context(OutOfBounds {
                    what: WhatBound::ExecutionEnvironment,
                    index: transaction.ee_index as usize,
                })?;
            let code = &execution_environment.wasm_code;

            let pre_state = shard_chain
                .execution_environment_state
                .get(&EeIndex(transaction.ee_index))
                .unwrap_or(&execution_environment.initial_shard_state);
            let data = base64::decode(&transaction.base64_encoded_data).context(Decode)?;
            let mut runtime = RootRuntime::new(&code, &data, pre_state.data);
            let post_root = runtime.execute();
            drop(runtime);

            shard_chain.execution_environment_state.insert(
                EeIndex(transaction.ee_index),
                ExecutionEnvironmentState { data: post_root },
            );

            transactions.push(ShardTransaction {
                data,
                ee_index: EeIndex(transaction.ee_index),
            });
        }

        let shard_block = ShardBlock::new(transactions);

        shard_chain.shard_blocks.push(shard_block);
        Ok((shard_chain.shard_blocks.len() - 1) as u32)
    }

    /* Getter methods still needed
        Beacon State
        Transactions (do we want to store EE state per shard before / after each transaction?

    */

    pub fn shard_chain(&self, arg: args::GetShardChain) -> Result<args::ShardChain> {
        // TODO: Add useful information to the output. Maybe count of blocks?

        let idx = arg.shard_chain_index as usize;
        let chain = self.shard_chains.get(idx).context(OutOfBounds {
            what: WhatBound::ShardChain,
            index: idx,
        })?;

        Ok(args::ShardChain {})
    }

    pub fn get_shard_block(&self, args: args::GetShardBlock) -> Result<args::ShardBlock> {
        let shard_chain = self
            .shard_chains
            .get(args.shard_chain_index as usize)
            .context(OutOfBounds {
                what: WhatBound::ShardChain,
                index: args.shard_chain_index as usize,
            })?;

        let shard_block = shard_chain
            .shard_blocks
            .get(args.shard_block_index as usize)
            .context(OutOfBounds {
                index: args.shard_block_index as usize,
                what: WhatBound::ShardBlock(args.shard_chain_index),
            })?;

        Ok(args::ShardBlock::from(shard_block))
    }
}

mod base64_vec {
    use serde::de::{Deserialize, Deserializer, Error as _, Unexpected};
    use serde::Serializer;

    pub fn serialize<T, S>(bytes: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: AsRef<[u8]>,
        S: Serializer,
    {
        let txt = base64::encode(bytes.as_ref());
        serializer.serialize_str(&txt)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let txt = String::deserialize(deserializer)?;

        base64::decode(&txt)
            .map_err(|_| D::Error::invalid_value(Unexpected::Str(&txt), &"base64 encoded bytes"))
    }
}

mod base64_arr {
    use serde::de::{Deserialize, Deserializer, Error as _, Unexpected};
    use serde::Serializer;

    use super::ToBytes32;

    pub use super::base64_vec::serialize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = super::base64_vec::deserialize(deserializer)?;

        vec.to_bytes32().map_err(|_| {
            D::Error::invalid_value(Unexpected::Bytes(&vec), &"exactly 32 base64 encoded bytes")
        })
    }
}

/// Incoming arguments and return values.
pub mod args {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Default)]
    pub struct CreateBeaconBlock {
        pub beacon_block: BeaconBlock,
    }

    #[derive(Debug, Default)]
    pub struct GetBeaconBlock {
        pub beacon_block_index: u32,
    }

    #[derive(Debug, Default)]
    pub struct CreateExecutionEnvironment {
        pub execution_environment: ExecutionEnvironment,
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct GetExecutionEnvironment {
        pub execution_environment_index: u32,
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct CreateShardChain {}

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct GetShardChain {
        pub shard_chain_index: u32,
    }

    #[derive(Debug, Default)]
    pub struct CreateShardBlock {
        pub shard_chain_index: u32,
        pub shard_block: ShardBlock,
    }
    #[derive(Debug, Default)]
    pub struct GetShardBlock {
        pub shard_chain_index: u32,
        pub shard_block_index: u32,
    }

    // Return values AND/OR sub-components of incoming argument values

    #[derive(Debug, Default)]
    pub struct BeaconBlock {
        pub cross_links: HashMap<u32, CrossLink>
    }
    impl From<&super::BeaconBlock> for BeaconBlock {
        fn from(beacon_block: &super::BeaconBlock) -> Self {
            let mut cross_links: HashMap<u32, CrossLink> = HashMap::new();
            for (k, v) in &beacon_block.cross_links {
                let super::ShardChainIndex(shard_chain_index) = k;
                let cross_link: CrossLink = CrossLink::from(v);
                cross_links.insert(*shard_chain_index, cross_link);
            }
            Self {
                cross_links,
            }
        }
    }

    #[derive(Default, Debug)]
    pub struct CrossLink {
        pub execution_environment_states: HashMap<u32, Vec<u8>>
    }
    impl From<&super::CrossLink> for CrossLink {
        fn from(cross_link: &super::CrossLink) -> Self {
            let mut execution_environment_states: HashMap<u32, Vec<u8>> = HashMap::new();
            for (k, v) in &cross_link.execution_environment_states {
                let super::EeIndex(k) = k;
                execution_environment_states.insert(*k, v.clone());
            }
            Self {
                execution_environment_states,
            }
        }
    }

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct ExecutionEnvironment {
        #[serde(with = "super::base64_vec")]
        pub wasm_code: Vec<u8>,

        #[serde(with = "super::base64_arr")]
        pub initial_state: [u8; 32],
    }

    impl From<&super::ExecutionEnvironment> for ExecutionEnvironment {
        fn from(ee: &super::ExecutionEnvironment) -> Self {
            Self {
                wasm_code: ee.wasm_code.clone(),
                initial_state: ee.initial_shard_state.data,
            }
        }
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct SimulationState {
        pub num_execution_environments: u32,
        pub num_shard_chains: u32,
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct ShardChain {}

    #[derive(Debug, Default, Eq, PartialEq)]
    pub struct ShardBlock {
        pub transactions: Vec<ShardTransaction>,
    }

    impl From<&super::ShardBlock> for ShardBlock {
        fn from(sb: &super::ShardBlock) -> Self {
            let transactions: Vec<ShardTransaction> = sb
                .transactions
                .iter()
                .map(|st| -> ShardTransaction { ShardTransaction::from(st) })
                .collect();
            ShardBlock { transactions }
        }
    }

    #[derive(Debug, Default, Eq, PartialEq)]
    pub struct ShardTransaction {
        pub base64_encoded_data: String,
        pub ee_index: u32,
    }

    impl From<&super::ShardTransaction> for ShardTransaction {
        fn from(st: &super::ShardTransaction) -> Self {
            let base64_encoded_data = base64::encode(&st.data);
            let super::EeIndex(ee_index) = st.ee_index;
            Self {
                base64_encoded_data,
                ee_index,
            }
        }
    }
}

#[derive(Debug, Default)]
struct BeaconChain {
    beacon_blocks: Vec<BeaconBlock>,
    // There are an unbounded number of EEs that can "exist" on the beacon chain
    execution_environments: Vec<ExecutionEnvironment>,
}

impl BeaconChain {
    fn new() -> Self {
        Self {
            beacon_blocks: Vec::new(),
            execution_environments: Vec::new(),
        }
    }

    fn add_beacon_block(
        &mut self,
        beacon_block: BeaconBlock,
    ) -> BeaconBlockIndex {
        self.beacon_blocks.push(beacon_block);
        BeaconBlockIndex((self.beacon_blocks.len() - 1) as u32)
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

trait ToBytes32 {
    fn to_bytes32(&self) -> Result<[u8; 32]>;
}

impl ToBytes32 for Vec<u8> {
    fn to_bytes32(&self) -> Result<[u8; 32]> {
        if self.len() == 32 {
            let mut ret: [u8; 32] = [0; 32];
            ret.copy_from_slice(&self[..]);
            Ok(ret)
        } else {
            Err(Error::InvalidBytes32)
        }
    }
}

#[derive(Debug, Default, Hash, Clone, Copy, Eq, PartialEq)]
pub struct EeIndex(u32);
#[derive(Debug, Default, Hash, Clone, Copy, Eq, PartialEq)]
pub struct BeaconBlockIndex(u32);
#[derive(Debug, Default, Hash, Clone, Copy, Eq, PartialEq)]
pub struct ShardChainIndex(u32);

// TODO: Add beacon block / shard block index types

// The execution environment data that lives on the beacon chain
// Does NOT include shard-specific EE state
#[derive(Debug)]
struct ExecutionEnvironment {
    wasm_code: Vec<u8>,
    initial_shard_state: ExecutionEnvironmentState,
}

impl From<args::ExecutionEnvironment> for ExecutionEnvironment {
    fn from(ee_args: args::ExecutionEnvironment) -> Self {
        Self {
            wasm_code: ee_args.wasm_code,
            initial_shard_state: ExecutionEnvironmentState {
                data: ee_args.initial_state,
            },
        }
    }
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
impl TryFrom<args::ShardBlock> for ShardBlock {
    type Error = Error;
    fn try_from(sb_args: args::ShardBlock) -> Result<Self, Self::Error> {
        let transactions: Result<Vec<ShardTransaction>> = sb_args
            .transactions
            .iter()
            .map(|sbt_args| -> Result<ShardTransaction> { ShardTransaction::try_from(sbt_args) })
            .collect();
        match transactions {
            Err(e) => Err(e),
            Ok(transactions) => Ok(ShardBlock { transactions }),
        }
    }
}

#[derive(Default, Debug)]
struct ShardTransaction {
    data: Vec<u8>,
    ee_index: EeIndex,
}
impl TryFrom<&args::ShardTransaction> for ShardTransaction {
    type Error = Error;
    fn try_from(sbt_args: &args::ShardTransaction) -> Result<Self, Self::Error> {
        let data = base64::decode(&sbt_args.base64_encoded_data).context(Decode)?;
        let ee_index = EeIndex(sbt_args.ee_index);
        Ok(Self { data, ee_index })
    }
}

#[derive(Default, Debug)]
struct BeaconBlock {
    cross_links: HashMap<ShardChainIndex, CrossLink>
}
impl From<args::BeaconBlock> for BeaconBlock {
    fn from(beacon_block_args: args::BeaconBlock) -> Self {
        let mut cross_links: HashMap<ShardChainIndex, CrossLink> = HashMap::new();
        for (k, v) in beacon_block_args.cross_links {
            let k = ShardChainIndex(k);
            let v = CrossLink::from(v);
            cross_links.insert(k, v);
        }
        Self {
            cross_links,
        }
    }
}


#[derive(Default, Debug, Clone)]
struct CrossLink {
    execution_environment_states: HashMap<EeIndex, Vec<u8>>
}
impl From<args::CrossLink> for CrossLink {
    fn from(cross_link_args: args::CrossLink) -> Self {
        let mut execution_environment_states: HashMap<EeIndex, Vec<u8>> = HashMap::new();
        for (k, v) in cross_link_args.execution_environment_states {
            let k = EeIndex(k);
            execution_environment_states.insert(k, v);
        }
        Self {
            execution_environment_states,
        }
    }
}

#[derive(Default, Debug)]
/// Contains all un-executed transactions that have yet to be included in a block
struct PendingTransactionPool {
    shard_transactions: Vec<HashMap<u32, Vec<ShardTransaction>>>
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::FromHex;
    use std::path::Path;

    #[test]
    fn can_create_and_get_beacon_blocks() {
        let mut eth = Simulation::new();
        let beacon_block = BeaconBlock {
            cross_links: vec![CrossLink::default()],
        };
        let args = args::CreateBeaconBlock {
            beacon_block: args::BeaconBlock {
                cross_links: HashMap::new(),
            }
        };
        let beacon_block_index = eth.create_beacon_block(args);
        assert_eq!(
            beacon_block_index, 0,
            "The first beacon block created should have an index of 0"
        );
    }
    #[test]
    fn can_create_and_get_execution_environments() {
        let mut eth = Simulation::new();

        // Can create a new EE
        let example_wasm_code = b"some wasm code here";
        let ee_args = args::ExecutionEnvironment {
            wasm_code: example_wasm_code.to_vec(),
            initial_state: [0; 32],
        };
        let create_ee_args = args::CreateExecutionEnvironment {
            execution_environment: ee_args,
        };
        let result = eth.create_execution_environment(create_ee_args).unwrap();
        assert_eq!(
            result, 0,
            "The first execution environment created should have an index of 0"
        );

        // Can retrieve the newly-created EE
        let get_ee_args = args::GetExecutionEnvironment {
            execution_environment_index: result,
        };
        let ee_args_retrieved = eth.get_execution_environment(get_ee_args).unwrap();
        assert_eq!(
            ee_args_retrieved.wasm_code, example_wasm_code,
            "EE wasm code retrieved should match the EE wasm code that was created"
        );

        // Can create and retrieve a second EE
        let example_wasm_code = b"some other wasm code here";
        let ee_args = args::ExecutionEnvironment {
            wasm_code: example_wasm_code.to_vec(),
            initial_state: [0; 32],
        };
        let create_ee_args = args::CreateExecutionEnvironment {
            execution_environment: ee_args,
        };
        let result = eth.create_execution_environment(create_ee_args).unwrap();
        assert_eq!(
            result, 1,
            "The second execution environment created should have an index of 1"
        );
        let get_ee_args = args::GetExecutionEnvironment {
            execution_environment_index: result,
        };
        let ee_args_retrieved = eth.get_execution_environment(get_ee_args).unwrap();
        assert_eq!(
            ee_args_retrieved.wasm_code, example_wasm_code,
            "EE wasm code retrieved should match the EE wasm code that was created"
        );
    }
    #[test]
    fn getting_ee_at_incorrect_index_should_return_err() {
        let mut eth = Simulation::new();
        let get_ee_args = args::GetExecutionEnvironment {
            execution_environment_index: 155512,
        };
        let ee_args_retrieved = eth.get_execution_environment(get_ee_args);
        assert!(ee_args_retrieved.is_err());
    }
    #[test]
    fn can_create_shard_chains() {
        let mut eth = Simulation::new();
        let sc_args = args::CreateShardChain {};
        let result = eth.create_shard_chain(sc_args);
        assert_eq!(
            result, 0,
            "The first shard chain created should have an index of 0"
        );

        let sc_args = args::CreateShardChain {};
        let result = eth.create_shard_chain(sc_args);
        assert_eq!(
            result, 1,
            "The second shard chain created should have an index of 1"
        );
    }
    #[test]
    fn can_get_simulation_state() {
        let mut eth = Simulation::new();

        let general_state = eth.simulation_state();
        assert_eq!(0, general_state.num_shard_chains);
        assert_eq!(0, general_state.num_execution_environments);

        let sc_args = args::CreateShardChain {};
        eth.create_shard_chain(sc_args);

        let general_state = eth.simulation_state();
        assert_eq!(1, general_state.num_shard_chains);
        assert_eq!(0, general_state.num_execution_environments);

        let ee_args = args::ExecutionEnvironment {
            wasm_code: b"wasm msaw".to_vec(),
            initial_state: [0; 32],
        };
        let create_ee_args = args::CreateExecutionEnvironment {
            execution_environment: ee_args,
        };
        eth.create_execution_environment(create_ee_args);
        let general_state = eth.simulation_state();
        assert_eq!(1, general_state.num_shard_chains);
        assert_eq!(1, general_state.num_execution_environments);
    }

    fn create_example_shard_block_args(ee_index: u32) -> args::ShardBlock {
        // Create transaction arguments
        let transaction_args1 = args::ShardTransaction {
            base64_encoded_data: base64::encode("some data"),
            ee_index,
        };
        let transaction_args2 = args::ShardTransaction {
            base64_encoded_data: base64::encode("some other data"),
            ee_index,
        };

        // Create shard block arguments
        let sb_args = args::ShardBlock {
            transactions: vec![transaction_args1, transaction_args2],
        };

        sb_args
    }
    #[test]
    fn can_create_and_get_shard_blocks() {
        let mut eth = Simulation::new();

        // Add EE
        let example_wasm_code: &[u8] = include_bytes!("../../tests/do_nothing.wasm");
        let ee_args = args::ExecutionEnvironment {
            wasm_code: example_wasm_code.to_vec(),
            initial_state: [0; 32],
        };
        let create_ee_args = args::CreateExecutionEnvironment {
            execution_environment: ee_args,
        };
        let ee_index = eth.create_execution_environment(create_ee_args).unwrap();

        // Add Shard Chain
        let sc_args = args::CreateShardChain {};
        let sc_index = eth.create_shard_chain(sc_args);

        // Create shard block args
        let sb_args1 = create_example_shard_block_args(ee_index);
        let sb_args2 = create_example_shard_block_args(ee_index);

        // Add shard blocks and assert that indices look correct
        let create_shard_block_args1 = args::CreateShardBlock {
            shard_chain_index: sc_index,
            shard_block: sb_args1,
        };
        let create_shard_block_args2 = args::CreateShardBlock {
            shard_chain_index: sc_index,
            shard_block: sb_args2,
        };
        let block_index1 = eth.create_shard_block(create_shard_block_args1).unwrap();
        let block_index2 = eth.create_shard_block(create_shard_block_args2).unwrap();
        assert_eq!(
            block_index1, 0,
            "first shard block added should have index of 0"
        );
        assert_eq!(
            block_index2, 1,
            "second shard block added should have index of 1"
        );

        // Get back shard blocks and make sure they look the same as originally
        let get_shard_block_args1 = args::GetShardBlock {
            shard_chain_index: sc_index,
            shard_block_index: block_index1,
        };
        let mut sb_args_returned = eth.get_shard_block(get_shard_block_args1).unwrap();
        assert_eq!(
            sb_args_returned,
            create_example_shard_block_args(ee_index),
            "value saved should match initial args passed in"
        );

        let get_shard_block_args2 = args::GetShardBlock {
            shard_chain_index: sc_index,
            shard_block_index: block_index2,
        };
        let mut sb_args_returned = eth.get_shard_block(get_shard_block_args2).unwrap();
        assert_eq!(
            sb_args_returned,
            create_example_shard_block_args(ee_index),
            "value saved should match initial args passed in"
        );
    }

    fn produce_block(wasm_code: &[u8], initial_state: &str, data: &str, expected_end_state: &str) {
        let mut simulation = Simulation::new();

        let execution_environment = args::ExecutionEnvironment {
            wasm_code: wasm_code.to_vec(),
            initial_state: Vec::from_hex(initial_state).unwrap().to_bytes32().unwrap(),
        };
        assert_eq!(
            0,
            simulation
                .create_execution_environment(args::CreateExecutionEnvironment {
                    execution_environment,
                })
                .unwrap()
        );

        assert_eq!(0, simulation.create_shard_chain(args::CreateShardChain {}));

        let shard_block = args::ShardBlock {
            transactions: vec![args::ShardTransaction {
                base64_encoded_data: base64::encode(&Vec::from_hex(data).unwrap()),
                ee_index: 0,
            }],
        };
        assert_eq!(
            0,
            simulation
                .create_shard_block(args::CreateShardBlock {
                    shard_chain_index: 0,
                    shard_block,
                })
                .unwrap()
        );

        let post_state = simulation.shard_chains[0]
            .execution_environment_state
            .get(&EeIndex(0))
            .unwrap();
        let expected_state: [u8; 32] = FromHex::from_hex(expected_end_state).unwrap();
        assert_eq!(expected_state, post_state.data);
    }

    #[test]
    fn run_scout_helloworld() {
        produce_block(
            include_bytes!("../../tests/phase2_helloworld.wasm"),
            "0000000000000000000000000000000000000000000000000000000000000000",
            "",
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
    }

    #[test]
    fn run_scout_bazaar() {
        produce_block(
            include_bytes!("../../tests/phase2_bazaar.wasm"),
            "22ea9b045f8792170b45ec629c98e1b92bc6a19cd8d0e9f37baaadf2564142f4",
            "5c0000005000000001000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000001010101010101010101010101010101010101010101010101010101010101010400000000000000",
            "29505fd952857b5766c759bcb4af58eb8df5a91043540c1398dd987a503127fc",
        );
    }
}
