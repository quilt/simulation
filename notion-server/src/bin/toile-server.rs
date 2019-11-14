use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "b", long = "bind", default_value = "127.0.0.1:0")]
    /// IP address and port to listen on for API requests.
    bind: SocketAddr,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
