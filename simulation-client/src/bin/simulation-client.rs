use simulation_client::{Result, SimulationClient};
use structopt::StructOpt;
use url::Url;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "b", long = "base_url", default_value = "http://127.0.0.1:8999")]
    /// IP address and port to listen on for API requests.
    base_url: Url,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client
    let opt = Opt::from_args();
    let client = SimulationClient::new(opt.base_url);

    // Create EE
    let ee = simulation_args::ExecutionEnvironment {
        initial_state: [5; 32],
        wasm_code: Vec::new(),
    };
    let create_ee_args = simulation_args::CreateExecutionEnvironment {
        ee,
    };
    let result = client.create_execution_environment(create_ee_args).await?;
    println!("created with index: {}", result);

    // Get Shard State
    let get_shard_state_args = simulation_args::GetShardState {
        shard_index: 10,
    };
    let result = client.get_shard_state(get_shard_state_args).await?;
    println!("Shard state debug: {:?}", result);

    Ok(())
}
