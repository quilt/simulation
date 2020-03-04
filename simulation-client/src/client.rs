use crate::{Result};
use http::{Request, Response};
use std::net::SocketAddr;


pub struct SimulationClient {
    /// IP address and port of simulation-server for sending API requests.
    bind: SocketAddr,
}

impl SimulationClient {
    pub fn new(bind: SocketAddr) -> Self {
        Self {
            bind,
        }
    }

    fn create_execution_environment(self, a: simulation_args::CreateExecutionEnvironment) -> Result<u64> {
        unimplemented!();
    }
}
