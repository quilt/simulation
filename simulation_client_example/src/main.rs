/// An example binary that demonstrates how one can use just the `simulation_client` library to
/// make requests to an already-running `simulation_server` instance. That server instance could
/// be running in another process on your local machine, or could be exposed on the web somewhere.
/// This library does NOT care about the internals of `eth2::simulation::Simulation`, and only
/// requires the `simulation_args` library in its Cargo.toml.
/// Note: `simulation_client` works by sending HTTP requests to `simulation_server`, but has
/// the exact same interface as if you had called methods on a `eth2::simulation::Simulation`
/// instance directly.
use hex::FromHex;
use simulation_args::{ToBytes32};
use simulation_client::{Result, SimulationClient};
use structopt::StructOpt;
use url::Url;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(
    short = "b",
    long = "base_url",
    default_value = "http://127.0.0.1:8999"
    )]
    /// IP address and port to listen on for API requests.
    base_url: Url,
}

/// Runs the "bazaar" example from the scout repo
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client
    let opt = Opt::from_args();
    let simulation_client = SimulationClient::new(opt.base_url);

    let initial_state = "22ea9b045f8792170b45ec629c98e1b92bc6a19cd8d0e9f37baaadf2564142f4";
    let initial_state = Vec::from_hex(initial_state).unwrap().to_bytes32().unwrap();
    let expected_post_state =
        "29505fd952857b5766c759bcb4af58eb8df5a91043540c1398dd987a503127fc";
    let expected_post_state = Vec::from_hex(expected_post_state)
        .unwrap()
        .to_bytes32()
        .unwrap();

    let data: Vec<u8> = Vec::from_hex("5c0000005000000001000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000001010101010101010101010101010101010101010101010101010101010101010400000000000000").unwrap();
    let shard_index = 0;

    run_block_and_verify_state(
        simulation_client,
        include_bytes!("../wasm_code/phase2_bazaar.wasm"),
        initial_state,
        data,
        expected_post_state,
        shard_index,
    ).await?;

    Ok(())
}

// Note that this looks very similar to `test_block_with_single_transaction`, but uses
// SimulationClient instead of Simulation
async fn run_block_and_verify_state(
    simulation_client: SimulationClient,
    wasm_code: &[u8],
    initial_state: [u8; 32],
    data: Vec<u8>,
    expected_post_state: [u8; 32],
    shard_index: u64,
) -> Result<()> {
    // Create EE with the specified code and initial state
    let ee = simulation_args::ExecutionEnvironment {
        initial_state,
        wasm_code: wasm_code.to_vec(),
    };
    let create_ee_args = simulation_args::CreateExecutionEnvironment { ee };
    let ee_index = simulation_client
        .create_execution_environment(create_ee_args)
        .await?;
    assert_eq!(ee_index, 0);

    // Set up a shard transaction with the specified data
    let shard_transaction = simulation_args::ShardTransaction { data, ee_index };

    // Create a shard block with the one transaction in it
    let shard_block = simulation_args::ShardBlock {
        transactions: vec![shard_transaction],
    };
    let create_shard_block_args = simulation_args::CreateShardBlock {
        shard_index,
        shard_block,
    };
    // This creates the block and runs all the transactions inside it
    simulation_client
        .create_shard_block(create_shard_block_args)
        .await?;

    // Get back the EE state to make sure it matches the expected_post_state
    let get_ee_state_args = simulation_args::GetExecutionEnvironmentState {
        ee_index,
        shard_index,
    };
    let ee_post_state = simulation_client
        .get_execution_environment_state(get_ee_state_args)
        .await?;
    assert_eq!(
        ee_post_state, expected_post_state,
        "actual post state root should match expected post state root"
    );

    Ok(())
}
