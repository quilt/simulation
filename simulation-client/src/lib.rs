use simulation_args;
use snafu::Snafu;

mod client;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("error with HTTP request"))]
    HTTPError,

    #[snafu(display("error in underlying simulation"))]
    SimulationError,
}

/// Shorthand for result types returned by this library
pub type Result<V, E = Error> = std::result::Result<V, E>;

pub use client::SimulationClient;
