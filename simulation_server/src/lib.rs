//! `simulation_server` is a server that simulates Ethereum 2.0's second phase,
//! with a particular focus on evaluating execution environments.

#![feature(proc_macro_hygiene, decl_macro)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod api;
mod dispatch;

use futures_util::future::{self, FutureExt};
use futures_util::pin_mut;
use simulation::Simulation;
use snafu::{Backtrace, ResultExt, Snafu};
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use types::eth_spec::EthSpec;

mod error {
    use super::*;

    /// Errors arising from the simulation or from underlying OS errors.
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub enum Error {
        /// Errors returned by the API.
        Api {
            /// The underlying error as returned by the API.
            source: api::Error,
        },

        /// Errors returned by the Dispatch logic
        Dispatch {
            /// The underlying error as returned by the dispatch mod
            source: dispatch::Error,
        },

        /// Errors returned by tokio.
        Tokio {
            /// The underlying error as returned by tokio.
            source: tokio::io::Error,

            /// The location the error was captured.
            backtrace: Backtrace,
        },
    }
}

pub use error::Error;

/// Shorthand type for results with this crate's error type.
pub type Result<V, E = Error> = std::result::Result<V, E>;

/// Configuration options for starting a `SimulationServer` instance.
#[derive(Debug, Clone)]
pub struct SimulationServerBuilder<T: EthSpec> {
    bind: SocketAddr,
    // #PhantomDataExplanation
    // Required to be able to write SimulationServerBuilder<T: EthSpec> without actually
    // using the T value anywhere in the SimulationServerBuilder implementation, which is
    // in turn required because we want to be able to specify Simulation<T> in the
    // initialization of SimulationServer and SimulationServerBuilder
    phantom: PhantomData<T>,
}

impl<T: EthSpec> SimulationServerBuilder<T> {
    /// Create a new `SimulationServer` instance from the configuration in this builder.
    pub fn build(self) -> SimulationServer<T> {
        SimulationServer {
            bind: self.bind,
            phantom: PhantomData,
        }
    }

    /// Set the local address the server will listen to.
    ///
    /// Binding to port zero will attempt to automatically assign a port.
    pub fn bind(mut self, bind: SocketAddr) -> Self {
        self.bind = bind;
        self
    }
}

impl<T: EthSpec> Default for SimulationServerBuilder<T> {
    fn default() -> Self {
        SimulationServerBuilder {
            bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            phantom: PhantomData,
        }
    }
}

/// An HTTP/JSON server wrapper around eth2::simulation::Simulation
#[derive(Debug)]
pub struct SimulationServer<T: EthSpec> {
    bind: SocketAddr,
    // See #PhantomDataExplanation
    phantom: PhantomData<T>,
}

impl<T: EthSpec> SimulationServer<T> {
    /// Create a builder for a `SimulationServer`.
    pub fn builder() -> SimulationServerBuilder<T> {
        SimulationServerBuilder::default()
    }

    /// Start the simulation server and wait for it to finish.
    pub fn run(self) -> Result<()> {
        self.async_run()
    }

    #[tokio::main]
    async fn async_run(self) -> Result<()> {
        //        let simulation = ethereum::Simulation::new();
        let simulation: Simulation<T> = Simulation::new();
        let (dispatch, handle) = dispatch::Dispatch::new(simulation);

        let eth_run = tokio::spawn(dispatch.run().map(|x| x.context(error::Dispatch)));
        let api_run =
            tokio::task::spawn_blocking(move || api::run(&self, handle).context(error::Api));

        pin_mut!(eth_run);
        pin_mut!(api_run);

        future::try_select(eth_run, api_run)
            .await
            .unwrap()
            .factor_first()
            .0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
