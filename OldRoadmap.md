# Simulation Repository Roadmap (subject to change)

**Overall Goals:**
* Provide an easy-to-use way for users to create and test execution environments in an eth2-like environment
* Give a concrete example of what the various parts of Eth2 might look like and how they might interact
* Provide a framework for protocol devs / users to test out new ideas and demonstrate concepts via concrete examples
    * eg. how does this approach to statelessness look "in real life"
    * eg. here's a basic demonstration that suggests approach A to proof verification might be viable
    * eg. how might a state provider interact with users in eth2?
    * etc.

## Goal 1: Provide a basic interface for interacting with an Eth2-like simulation

### Proposed Interface

#### Structs
```
BeaconChain {
    beacon_blocks: Vec<BeaconBlock>
    execution_environments: Vec<ExecutionEnvironment>
}

BeaconBlock {
    cross_links: HashMap<shard_chain_index: u32, CrossLink>
}

// This is a "snapshot" of the EE states for the given shard as of 
// the creation of this CrossLink
CrossLink {
    execution_environment_states: HashMap<ee_index: u32, Vec<u8>>
}

ExecutionEnvironment {
    wasm_code: Vec<u8>
}

// In a stateless world, "EE State" is represented just by 32 bytes,
// which is expected to represent the root hash of some
// cryptographic data structure
ExecutionEnvironmentState {
    data: [u8; 32]
}

ShardChain {
    execution_environment_state: ExecutionEnvironmentState 
    shard_blocks: Vec<ShardBlock>
}

ShardBlock {
    transactions: Vec<ShardTransaction>
}

ShardTransaction {
    data: Vec<u8>
    ee_index: u32
}

// A pool of un-executed shard transactions that should eventually
// be executed and added to a shard block
PendingTransactionsPool {
    transactions: HashMap<shard_chain_index: u32, Vec<ShardTransaction>>
}

// Provides general data about the simulation state overall
SimulationState {
    num_execution_environments: u32
    num_shard_chains: u32
}
```

### Functions
```
// Create simulation, initialize base values
// (eg. create a beacon chain with zero blocks / zero EEs)
// (Possibly in future initialize with a "default EE" for users
// that just want to create contracts on that default EE)
Simulation.new()

// ***********
// EEs
// ***********

// Create the EE and add it to the beacon chain, possibly initialize
// shards to some initial state
CreateExecutionEnvironment(ExecutionEnvironment) -> ee_index: u32
// Get back the EE struct
GetExecutionEnvironment(ee_index:  u32) -> ExecutionEnvironment

// ***********
// Beacon Chain
// ***********

// Create Beacon Block and add it to the beacon chain
CreateBeaconBlock(BeaconBlock) -> beacon_block_index: u32
// Get back the BeaconBlock struct based on index
GetBeaconBlock(beacon_block_index: u32) -> BeaconBlock

// ***********
// Shard Chain
// ***********

// No arguments currently for this method, since ShardChains don't
// have any customizability on initialization
CreateShardChain() -> shard_chain_index: u32
// Create ShardBlock and add it to the ShardChain
// This method ALSO runs all transactions on the given block,
// allowing the EEs involved to modify their state according to their
// internal (arbitrary) wasm code 
CreateShardBlock(shard_chain_index: u32, ShardBlock) -> shard_block_index: u32
// Get back the ShardBlock struct based on index
GetShardBlock(shard_chain_index: u32, shard_block_index: u32) -> ShardBlock

// ***********
// High-Level Controls
// ***********

// Add the ShardTransaction to the PendingTransactionsPool
// This pool will be pulled from and executed according to the next function
AddTransactionToPool(shard_chain_index: u32, ShardTransaction)

// Puts the simulation in an "automated" mode which auto-creates blocks
// from transactions in the PendingTransactionsPool
// This is an alternative way to have transactions / blocks be created without doing
// it manually via the methods above (such as CreateShardBlock)
// This mode is meant to simulate how the system might work in a more "true-to-life"
// context vs. every piece of data being manually added to the simulation "by hand"
// The configuration passed to this function will likely be refined / added to as the
// "auto creation" mode is iterated upon
StartAutoCreateBlocks(
    // Create a block every <secondsBetweenBlocks> seconds
    secondsBetweenBlocks: u32,
    // Add <transactionsPerBlock> transactions to each block, if available 
    transactionsPerBlock: u32,
    // Create a BeaconBlock with crosslinks to the shards' state with a delay
    // of <crossLinkDelayInBlocks> blocks
    crossLinkDelayInBlocks: u32,
)
// Get some general info about the simulation state
GetSimulationState() -> SimulationState

```
## Goal 2: Provide one or more examples of ExecutionEnvironments

The simulation repo already has some very basic EEs that can be used with the simulation
(eg. users can create the EE in the simulation, run transactions on that EE, see the state update, etc). 

However, the current proposed "next step" after Goal 1 will be to create a more full-featured
EE that includes some of the features expected in "production" Eth2 EEs, such as:
* Stateless-ness
    * EE virtually "stores" state, but still represents that state via only the 32-bytes
    * Users wishing to run transactions against the stateless EE would need to submit a proof with their transaction for it to be valid.
        * (the proof basically is a means of the EE verifying that the "virtual" EE state matches the state the user is claiming in their transaction)
* Proof verification
    * Discussed above, the EE would include code that can validate incoming user proofs
* Reproducibility of state by outside users
    * Since the state is "virtual" and not stored anywhere, it is non-trivial to track the current "virtual" state of the EE.
    * The example EE would need to provide some mechanism by which outside observers (eg. state providers) could reproduce the virtual state tree by running all past transactions
    * One likely solution here is to have the EE make use of a host function that eg. logs out every state change as the transactions are executed.
    * In this way, an outside observer can replay all transactions, making note of the log statements, to observe the internal virtual state of the EE.
    
## Goal 3+: ??

Likely a lot will change between now and the completion of Goal 2 and even of Goal 1.  Roadmap beyond this is TBD.
