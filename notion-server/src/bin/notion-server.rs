use notion_server::{Notion, NotionBuilder, Result};

use std::net::SocketAddr;

use structopt::StructOpt;
use types::eth_spec::MainnetEthSpec;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "b", long = "bind", default_value = "127.0.0.1:0")]
    /// IP address and port to listen on for API requests.
    bind: SocketAddr,
}

impl Into<NotionBuilder<MainnetEthSpec>> for Opt {
    fn into(self) -> NotionBuilder<MainnetEthSpec> {
        Notion::builder().bind(self.bind)
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let builder: NotionBuilder<MainnetEthSpec> = opt.into();
    let notion = builder.build();

    notion.run()
}
