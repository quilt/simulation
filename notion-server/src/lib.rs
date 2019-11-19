//! `notion-server` is a server that simulates Ethereum 2.0's second phase, with
//! a particular focus on evaluating execution environments.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod ethereum;

mod error {
    use super::*;

    /// Errors arising from the simulation or from underlying OS errors.
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub enum Error {
        /// Errors returned by the simulation.
        Ethereum {
            /// The underlying error as returned by the simulation.
            source: ethereum::Error,
        },
    }
}

use crate::ethereum::Simulation;

pub use self::error::Error;

use snafu::{ResultExt, Snafu};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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
        Notion {}
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
pub struct Notion {}

impl Notion {
    /// Create a builder for a `Notion` server.
    pub fn builder() -> NotionBuilder {
        NotionBuilder::default()
    }

    /// Start the simulation server and wait for it to finish.
    pub fn run(&self) -> Result<()> {
        let eth = Simulation::spawn().context(error::Ethereum)?;

        // TODO: Fill this part in

        eth.join().context(error::Ethereum)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
