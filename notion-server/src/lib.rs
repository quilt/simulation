//! `notion-server` is a server that simulates Ethereum 2.0's second phase,
//! with a particular focus on evaluating execution environments.

#![feature(proc_macro_hygiene, decl_macro)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod api;
mod ethereum;

use futures_util::future::{self, FutureExt};
use futures_util::pin_mut;

use snafu::{Backtrace, ResultExt, Snafu};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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

        /// Errors returned by the simulation.
        Ethereum {
            /// The underlying error as returned by the simulation.
            source: ethereum::Error,
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

/// Configuration options for starting a `Notion` server instance.
#[derive(Debug, Clone)]
pub struct NotionBuilder {
    bind: SocketAddr,
}

impl NotionBuilder {
    /// Create a new `Notion` instance from the configuration in this builder.
    pub fn build(self) -> Notion {
        Notion { bind: self.bind }
    }

    /// Set the local address the server will listen to.
    ///
    /// Binding to port zero will attempt to automatically assign a port.
    pub fn bind(mut self, bind: SocketAddr) -> Self {
        self.bind = bind;
        self
    }
}

impl Default for NotionBuilder {
    fn default() -> Self {
        NotionBuilder {
            bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        }
    }
}

/// A server that simulates Ethereum 2.0's phase two.
#[derive(Debug)]
pub struct Notion {
    bind: SocketAddr,
}

impl Notion {
    /// Create a builder for a `Notion` server.
    pub fn builder() -> NotionBuilder {
        NotionBuilder::default()
    }

    /// Start the simulation server and wait for it to finish.
    pub fn run(self) -> Result<()> {
        self.async_run()
    }

    #[tokio::main]
    async fn async_run(self) -> Result<()> {
        let simulation = ethereum::Simulation::new();
        let (dispatch, handle) = ethereum::Dispatch::new(simulation);

        let eth_run = tokio::spawn(dispatch.run().map(|x| x.context(error::Ethereum)));
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
