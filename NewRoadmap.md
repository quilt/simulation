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
# Most of this stuff should be SSZ serializable
Simulation {
    current_beacon_state: <BeaconState>
    
}

# This is internal state -- outside interfaces should not know or care how this stuff is stored internally
# this stuff shouldn't necessarily be SSZ serializable
# (though collections may contain SSZ-compatible values)
# All this is just an in-memory store for all the objects the simulation needs to track
# All "spec" objects will follow spec and will be SSZ-serializable, but will truncated
# to include only fields that make sense for the simulation (these list of fields may grow
# as the simulation grows in scope)
Store {
    # Lighthouse *effectively* does this as a Hashmap, in that they look it up by root hash
    # (beacon_node/store/src/impls/beacon_state.rs)
    # In actuality they're not actually storing it as a hashmap, but...
    # How much do we care about copying their approach here?
    # Don't think we even need to store past beacon states for the current version
    # of the simulation...
    beacon_states: HashMap<root, BeaconState>

    beacon_blocks: HashMap<root, BeaconBlock>

    # Store this however we like until we come up with a spec (and possibly even after for ease of access)
    execution_environments: VectorOrHashMap<EE>

    pending_shard_transactions: PendingShardTransactionsPool
}
```

```
BeaconState {
    slot: Slot

    # History
    # list of roots of recent-past beacon block headers
    block_roots: Vector<Root, SLOTS_PER_HISTORICAL_ROOT>
    # list of roots of recent-past beacon states
    state_roots: Vector<Root, SLOTS_PER_HISTORICAL_ROOT>
    # believe this is a list of the roots of recent HistoricalBatch objects
    historical_roots: List<Root, HISTORICAL_ROOTS_LIMIT>

    # Removed everything to do with slashing, eth1, registry, randomness, attestations, finality

    # PHASE 1 (added by Greg)
    # index is shard number
    # I believe this would hold only the most recent crosslinks
    crosslinks: Vector<CrossLink>

    # PHASE 2 (added by Greg)
    # Suppose we can add this here bc no spec yet for how it's stored,
    # can always move to beacon block if necessary in future
    # ...can also just store these in the simulation store until we have a better
    # sense of the spec for these...
    # execution_environments: Vector<ExecutionEnvironment, MAX_EES>
}

BeaconBlock {
    slot: Slot
    parent_root: Root
    state_root: Root
    
    cross_links: Vector<CrossLink>
}

BeaconBlockHeader {
    # TODO: include the relevant shit here from the spec
}

// This is a "snapshot" of the EE states for the given shard as of 
// the creation of this CrossLink
CrossLink {
    execution_environment_states: HashMap<ee_index: u32, ExecutionEnvironmentState>
}

ExecutionEnvironment {
    wasm_code: Vec<u8>
}

// In a stateless world, "EE State" is represented just by 32 bytes,
// which is expected to represent the root hash of some
// cryptographic data structure
// TODO: Once spec for all this crap is defined, adhere to it here.
ExecutionEnvironmentState {
    data: [u8; 32]
}

ShardChain {
    execution_environment_state: ExecutionEnvironmentState 
    shard_blocks: Vec<ShardBlock>
}

ShardBlock {
    shard: Shard
    slot: ShardSlot
    beacon_block_root: Root
    parent_root: Root
    state_root: Root
    body: List[byte, MAX_SHARD_BLOCK_SIZE - SHARD_HEADER_SIZE]
    block_size_sum: uint64

    # Question: how are these stored on the block? presumably in the body, but do we
    # have hard requirements / guidelines on that?
    transactions: Vec<ShardTransaction>
}

class ShardBlockHeader(Container):
    shard: Shard
    slot: ShardSlot
    beacon_block_root: Root
    parent_root: Root
    state_root: Root
    body_root: Root
    block_size_sum: uint64

# Same question as above re: how transactions are stored
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
