## Concepts we will support (and their status)
* Beacon
    * Chain
    * Block
    * Crosslink
        * No spec yet for what crosslinks will look like
* Shard
    * Chain
    * Block
* 

Maybe have 2 sets of state:
* Virtual
    * Can include virtual state that we want to cache / track AND can include state that will actually exist in the blockchain that we just haven't implemented yet.
    * eg.
    ```
    
    // This is the internal state of the simulation, but the interface methods will remain static
    // eg. get_execution_environment
    // eg. add_transaction
    // Goal 1: get the interface methods right for the simulation as a whole
    // Goal 2: start moving over the internal state such that it starts to look more like the actual blockchain will look.
    struct Simulation {
        struct Virtual {
            // TODO: once cross_links defined in spec, move to "actual"
            cross_links: Vec<CrossLinks>
            // TODO: move execution environments to "actual"
            execution_environments: Vec<EE>
            pending_transactions_pool: Vec<PendingTransactions>
        }
        struct Actual {
            beacon_blocks: Vec<BeaconBlock>
        }  
    }
    ```
* Actual

## Misc Notes
* Lighthouse testnet uses HTTP + JSON for API methods, doesn't appear to use SSZ
* FixedVector, which seems to be what's being suggested, is a data-structure with finite # of values, and does not support add/remove, only modifying existing elements

## Open Questions
* What spec should I use?  The existing spec is almost exclusively for Phase 0, with some Phase 1 sprinkled in
    * Seems premature to worry overmuch about internal storage of Crosslinks, EEs, other Phase 2 stuff until a spec for these is up.
    * If we want to shape the spec, then what's the goal here?  I think it makes sense to start with something then go from there.
* Many of the beacon-related spec entries are for consensus (eth balance, validator groupings, etc)
    * I assume we do NOT have these in scope for the roadmap of simulation?
* Do we want to support things like epochs in the near term?
    * my original plan was just have a chain of blocks, since epoch is related more to finality, which I don't think the simulation should care about for a while (if ever)
* Where / why do we care about SSZ support?
    * What will we be able to do with SSZ support that we won't be able to do otherwise?
    * eg. what is a user story start-to-finish that requires SSZ support
    * HAPPY to support this, but need to know where in the simulation the hook-in needs to happen.  "Support SSZ" is meaningless to me without more specifics.  IMO there is no need to have EVERY struct (de)serialize to SSZ.
* Who are my users?
    * Matt is one -> I'll make sure I build his ideal simulation.
    * Ansgar / Sam testing is another -> have reached out to them for thoughts on what they need.
    * Who else?
        * Let's write a couple user stories for the other future folks that we want to use this.
* Need to enumerate virtual vs. actual state, "true" blockchain won't actually track virtual state
    * Also, may be useful in the simulation to have convenience methods that return the virtual state
    * eg. get_execution_environment still is a great method, maybe it just grabs the EE from the chain under the hood.

    
## Goals (from Matt / Will meeting)
### Long Term
* Be able to start up a node 
* "Ganache-like"
    * Action item: make sure we're on the same page about what this means for now.
    * Quote from their site:
        * "Quickly fire up a personal Ethereum blockchain which you can use to run tests, execute commands, and inspect state while controlling how the chain operates."
    * My interpretation:
        * Should look and act like the actual Eth2 blockchain, but can mock out the unimportant stuff (eg. maybe I'll mock out EVERYTHING other than the EE / crosslink stuff for starters)
        * Should be able to examine the beacon and shard blocks via access methods
        * Should be able to create + use execution environments
        * Should be able to submit transactions and see the results of those transactions.
### Short Term
* See it working with Sam + Ansgar stuff
    * what is their stuff?  What do they require?
    * Have reached out to them, waiting on response.
* Support Matt's work
    * Does Matt require anything other than what we have so far?
* Build something that makes testing EEs easy, but also have the low-level structs be spec- and ssz-compatible