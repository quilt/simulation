pub mod simulation;
mod store;

use snafu::Snafu;
use std::fmt;

/// Shorthand for result types returned from the Simulation simulation.
pub type Result<V, E = Error> = std::result::Result<V, E>;

#[derive(Debug)]
pub enum WhatBound {
    ExecutionEnvironment,
    ExecutionEnvironmentState,
    ShardBlock(usize),
    Shard,
}

impl fmt::Display for WhatBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WhatBound::ExecutionEnvironment => write!(f, "execution environment"),
            WhatBound::ExecutionEnvironmentState => write!(f, "execution environment state"),
            WhatBound::Shard => write!(f, "shard"),
            WhatBound::ShardBlock(shard) => write!(f, "block on shard {}", shard),
        }
    }
}

/// Errors arising from the simulation.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{} exceeds max allowable length", what))]
    MaxLengthExceeded { what: String },
    #[snafu(display("no {} exists at index: {}", what, index))]
    OutOfBounds { what: WhatBound, index: usize },
}

pub use simulation::Simulation;
pub use simulation::args;
