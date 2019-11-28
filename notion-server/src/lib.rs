//! `notion-server` (roughly pronounced twaal) is a server that simulates Ethereum
//! 2.0's second phase, with a particular focus on evaluating execution
//! environments.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod ethereum;

use snafu::{Backtrace, ResultExt, Snafu};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio::runtime::{Runtime, Handle};
use tokio::sync::oneshot::channel as oneshot;

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
        let simulation = ethereum::Simulation::new();
        let handle = ethereum::Handle::new(simulation);

        let (sender, receiver) = oneshot();

        let mut spawner = Runtime::new().context(error::Tokio)?;

        spawner.spawn(async {
            let result = handle.run().await;
            sender.send(result).unwrap();
        });

        // TODO: Do other work here, after starting the simulation. For example,
        // start a web server.

        spawner
            .block_on(async { receiver.await })
            .unwrap()
            .context(error::Ethereum)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
