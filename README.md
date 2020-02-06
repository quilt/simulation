# Simulation Repository Roadmap (subject to change)

**Overall Goals:**
* Provide an easy-to-use way for users to create and test execution environments in an eth2-like environment
* Give a concrete example of what the various parts of Eth2 might look like and how they might interact
* Provide a framework for protocol devs / users to test out new ideas and demonstrate concepts via concrete examples
    * eg. how does this approach to statelessness look "in real life"
    * eg. here's a basic demonstration that suggests approach A to proof verification might be viable
    * eg. how might a state provider interact with users in eth2?
    * etc.

## Goal 1: Provide a basic interface for interacting with an Eth2-like simulation, specifically for creating and testing EEs.

### Proposed Initial Interface

#### High-level Simulation Methods
We anticipate adding many more methods in the future, but here are the functions we think are most necessary for the initial goal of testing EEs in an Eth2-like environment.
```rust
// Add a new execution environment, return EE index
fn create_execution_environment(a: args::CreateExecutionEnvironment) -> u32 {}

// Add a new shard block containing a list of transactions that need to be executed
// Execute all transactions on the appropriate shards / EEs, return ShardBlock index
fn create_shard_block(args: args::CreateShardBlock) -> u32 {}

// Get an EE that was previously added
fn get_execution_environment(args: args::GetExecutionEnvironment) -> ExecutionEnvironment {}

// Get a shard block that was previously added
fn get_shard_block(args: args::GetShardBlock) -> ShardBlock {}

// Get the specified ShardState, will contain EE states
fn get_shard_state(args: args::GetShardState) -> ShardState {}
```

#### Types / Structs
All types that are defined in the [Phase 0](https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md) and [Phase 1](https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase1/beacon-chain.md) spec documents will look the same in this simulation, with a couple exceptions:
1) Where appropriate, the simulation will either "mock" or neglect to include fields that are not relevant for the current goals of the simulation.
    * eg. initially, the simulation will not attempt to simulate actual consensus, so things like attestations, validator shuffling, etc. will not be included in the first versions.
    * These extra fields/concepts can be added in future versions if necessary.
2) This simulation will attempt to simulate several "unspecced" topics (eg. Phase 2 execution environments)
    * Obviously, we can't match the spec if the spec doesn't exist, so in these cases we'll do our best to match our best guess of what the spec might look like.
    * If and when any "unspecced" types get finalized, we'll update this simulation to match.

##### Unspecced Stuff
The following "unspecced" types will be included in the simulation:

```rust
struct ExecutionEnvironment {
    // arbitrary eWASM byte code
    wasm_code: Vec<u8>,
}

// Represents a transaction on a specific shard for a specific execution environment
struct ShardTransaction {
    // Arbitrary-length bytes included with the transaction
    // Could include arguments, witness data, or anything else this specific EE might require
    data: Vec<u8>,
    // The index of the execution environment in which this transaction will run
    ee_index: u32,
}
```

`ExecutionEnvironment` will be stored in the simulation on [the spec's](https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md) `BeaconState` struct as follows:
*Note: "Crosslinked" shard state was recently "specced", and is stored on `BeaconState.shard_states`*
```rust
struct BeaconState {
    // ...(fields from the spec here)
    // ...(minus any unneeded fields)
    
    execution_environments: Vec<ExecutionEnvironment>
}
```
`ShardTransaction` will be passed to the simulation as part of `ShardBlock`, as follows: 
```rust
struct ShardBlock {
    transactions: Vec<ShardTransaction>,
}
```
Note: this may not follow the spec exactly, but is useful for being able to execute a pre-bundled set of transactions on an EE via the `create_shard_block` function above.
If necessary, this interface (and the associated structs) may change in the future.

***

`ExecutionEnvironments` will have shard-specific state, which will be stored as 32 bytes of arbitrary data (in most EEs, this 32 bytes is expected to be used to store the root hash of the EE's state tree) 

This shard-specific state will be stored on the [the spec's](https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase1/shard-data-chains.md) `ShardState` struct as follows:
```rust
struct ShardState {
    // ...(fields from the spec here)
    // ...(minus any unneeded fields)
    
    // One 32-byte chunk of data for each of the execution environments
    execution_environment_states: Vec<[u8; 32]>,
}
```
##### Function Arguments
```rust
mod args {
    struct CreateExecutionEnvironment {
        wasm_code: Vec<u8>,
    }
    
    struct CreateShardBlock {
        shard_index: u32,
        transactions: Vec<ShardTransactions>,
    }
    
    struct GetExecutionEnvironment {
        execution_environment_index: u32,
    }
    
    struct GetShardBlock {
        shard_chain_index: u32,
        shard_block_index: u32,
    }
    
    struct GetShardState {
        // Initially, this method will only return the latest shard state, so it only requires the shard index
        // If necessary, this interface can be modified in the future to allow access to "older" copies of ShardState
        // for a given shard. 
        shard_index: u32,
    }
}
```

## Goal 2: Support existing efforts to provide one or more examples of ExecutionEnvironments

The simulation repo already has some very basic EEs that can be used with the simulation
(eg. users can create the EE in the simulation, run transactions on that EE, see the state update, etc). 

However, one effort currently being worked on in parallel is to create one or more full-featured EEs that includes some of the features expected in "production" Eth2 EEs, such as:
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
* etc.

The simulation should add whatever features are necessary to be compatible with this work. 

## Goal 2+: Keep up-to-date with the spec and add features as-necessary to support users


