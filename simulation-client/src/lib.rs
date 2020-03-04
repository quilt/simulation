use simulation_args;
use snafu::{Backtrace, Snafu};
use reqwest::Error as ReqwestError;
use url::ParseError;

mod client;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("unable to decode return value from server"))]
    Decode,

    #[snafu(display("error parsing http request"))]
    Parse { backtrace: Backtrace, source: ParseError },

    #[snafu(display("error in underlying reqwest library"))]
    Reqwest { backtrace: Backtrace, source: ReqwestError },

    // #[snafu(display("error with HTTP request"))]
    // HTTP,
}

/// Shorthand for result types returned by this library
pub type Result<V, E = Error> = std::result::Result<V, E>;

pub use client::SimulationClient;
