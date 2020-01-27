struct Simulation {
    store: Store,
}
struct Store {
    // All internal storage of the simulation goes here
    // This part need NOT be spec-compliant
}
impl Simulation {
    // 5 methods discussed go here
    // 5 methods can just access stuff on the store for ease of use
    // And...that's it?
}

// TODO:
/*
    * Add 5 methods on the simulation implementation (blank at first)
    * Write up all the spec structs and make sure they're SSZ compliant
    * For the 5 methods, fill in the implementations and add data structures to the store as necessary

    Organizationally, let's move everything into separate files
    * spec_types.rs
    * simulation.rs (includes Store + Args)
    * dispatch.rs
*/
