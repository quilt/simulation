use simulation_server::{Result, SimulationServerBuilder, SimulationServer};
use std::net::SocketAddr;
use structopt::StructOpt;
use types::eth_spec::MainnetEthSpec;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "b", long = "bind", default_value = "127.0.0.1:8999")]
    /// IP address and port to listen on for API requests.
    bind: SocketAddr,
}

impl Into<SimulationServerBuilder<MainnetEthSpec>> for Opt {
    fn into(self) -> SimulationServerBuilder<MainnetEthSpec> {
        SimulationServer::builder().bind(self.bind)
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let builder: SimulationServerBuilder<MainnetEthSpec> = opt.into();
    let simulation_server = builder.build();

    simulation_server.run()
}
