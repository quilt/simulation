## Beacon Chain
[Beacon Chain Spec](https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#beaconstate)

### Constants / Basic Types
```markdown
Name 	Value 	Unit 	Duration
SECONDS_PER_SLOT 	12 	seconds 	12 seconds
SLOTS_PER_EPOCH 	2**5 (= 32) 	slots 	6.4 minutes
/* ?? */ MIN_SEED_LOOKAHEAD 	2**0 (= 1) 	epochs 	6.4 minutes
/* ?? */ MAX_SEED_LOOKAHEAD 	2**2 (= 4) 	epochs 	25.6 minutes
/* # of slots to keep cached */ SLOTS_PER_HISTORICAL_ROOT 	2**13 (= 8,192) 	slots 	~27 hours
```
```
Name 	SSZ equivalent 	Description
Slot 	uint64 	a slot number
Epoch 	uint64 	an epoch number
Root 	Bytes32 	a Merkle root
Version 	Bytes4 	a fork version number
DomainType 	Bytes4 	a domain type
Domain 	Bytes8 	a signature domain
```

```python
def hash(data: bytes) -> Bytes32 is SHA256.
# Hash the object into a single root
def hash_tree_root(object: SSZSerializable) -> Root
```


###Types
```python
class BeaconBlock(Container):
    slot: Slot
    parent_root: Root
    state_root: Root
    body: BeaconBlockBody

    # PHASE 1 (added by Greg)
    # index is shard number
    # I believe this would hold the crosslinks for this particular beacon block
    crosslinks: Vector[CrossLink]
```

```python
class BeaconState(Container):
    # Versioning
    genesis_time: uint64
    slot: Slot
    fork: Fork

    # History
    latest_block_header: BeaconBlockHeader
    block_roots: Vector[Root, SLOTS_PER_HISTORICAL_ROOT]
    state_roots: Vector[Root, SLOTS_PER_HISTORICAL_ROOT]
    historical_roots: List[Root, HISTORICAL_ROOTS_LIMIT]

    # Removed everything to do with slashing, eth1, registry, randomness, attestations, finality

    # PHASE 1 (added by Greg)
    # index is shard number
    # I believe this would hold only the most recent crosslinks
    crosslinks: Vector[CrossLink]

    # PHASE 2 (added by Greg)
    # Recommend adding this here bc no spec yet for how it's stored,
    # can always move to beacon block if necessary in future
    execution_environments: Vector[ExecutionEnvironment, MAX_EES]
```

```markdown
class BeaconBlockBody(Container):
    # removed all 
    graffiti: Bytes32  # Arbitrary data
```

## Shard Chain

### Constants / Basic Types
```markdown
Name 	SSZ equivalent 	Description
Shard 	uint64 	a shard number
ShardSlot 	uint64 	a shard slot number
```
```
Name 	Value
SHARD_COUNT 	2**10 (= 1,024)
SHARD_HEADER_SIZE 	2**10 (= 1024)
SHARD_BLOCK_SIZE_TARGET 	2**14 (= 16,384)
MAX_SHARD_BLOCK_SIZE 	2**16 (= 65,536)

SHARD_SLOTS_PER_EPOCH 	2**7 (= 128) 	shard slots 	6.4 minutes
EPOCHS_PER_SHARD_PERIOD 	2**8 (= 256) 	epochs 	~27 hours

HISTORY_ACCUMULATOR_DEPTH 	2**6 (= 64)
```

###Types

// TODO: check whether the PR has more up-to-date crosslink info
```markdown
# Crosslink is a placeholder to appease the build script until phase 1 is reworked
# beacon_state.crosslinks[shard].shard_block_root <- example usage in this doc
class Crosslink(Container):
    shard: Shard
```

```markdown
class ShardBlock(Container):
    shard: Shard
    slot: ShardSlot
    beacon_block_root: Root
    parent_root: Root
    state_root: Root
    body: List[byte, MAX_SHARD_BLOCK_SIZE - SHARD_HEADER_SIZE]
    block_size_sum: uint64
```
```
class ShardBlockHeader(Container):
    shard: Shard
    slot: ShardSlot
    beacon_block_root: Root
    parent_root: Root
    state_root: Root
    body_root: Root
    block_size_sum: uint64
```
```
class ShardState(Container):
    shard: Shard
    slot: ShardSlot
    history_accumulator: Vector[Bytes32, HISTORY_ACCUMULATOR_DEPTH]
    latest_block_header: ShardBlockHeader
    block_size_sum: uint64
    
    # Note: consider using VariableList (or whatever the lighthouse folks do)
    ee_states: Vector<EEState(bytes32)>

```

What we need on the external facing interface:
* Some way to read blocks (shard and/or beacon)
    * Some way to read EE states
* ~~Some way to submit new transactions and have them created into blocks (pending transaction pool)~~
    * Will says we don't need this now.
    * 
* Some way to create a shard block straight up with transactions included.
    * Shard block will include anything it needs already
    * Most notably, will include list of transactions (and associated data)
    * Ok, sounds like existing fn / interface stuff can stick around for shard blocks + shard transactions
* Some way to create EEs

Incomplete spec item decisions

### ExecutionEnvironments
* We're gonna EE STATE on ShardState
* We're gonna EE CODE on BeaconState
* Existing interface for create / get is OK

### Crosslinks
* We're gonna store these on the beacon state


On same page now on internal storage, SSZ compatibility, crosslink and EE storage, etc.