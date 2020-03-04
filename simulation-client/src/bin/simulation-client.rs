use simulation_client::SimulationClient;
use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "b", long = "bind", default_value = "127.0.0.1:8999")]
    /// IP address and port to listen on for API requests.
    bind: SocketAddr,
}


fn main() {
    let opt = Opt::from_args();
    let client = SimulationClient::new(opt.bind);
    println!("hi greg: {:?}", opt);
}
