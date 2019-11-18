//! `notion-server` (roughly pronounced twaal) is a server that simulates Ethereum
//! 2.0's second phase, with a particular focus on evaluating execution
//! environments.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use snafu::Snafu;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Errors arising from the simulation or from underlying OS errors.
#[derive(Debug, Snafu)]
pub enum Error {}

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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
