use crate::dispatch::{args, Handle, Result as DispatchResult, simulation_args};
use crate::Notion;

use rocket::response::status;
use rocket::{config, uri};
use rocket::{get, post, routes, State};
use rocket_contrib::json::Json;
use snafu::{ResultExt, Snafu};
use types::eth_spec::EthSpec;

/// Shorthand for result types returned from the API server.
pub type Result<V, E = Error> = std::result::Result<V, E>;

/// Errors arising from the API server.
#[derive(Debug, Snafu)]
pub enum Error {
    Config { source: config::ConfigError },
}

pub fn run<T: EthSpec>(notion: &Notion<T>, handle: Handle) -> Result<()> {
    let config = config::Config::build(config::Environment::Development)
        .address(format!("{}", notion.bind.ip()))
        .port(notion.bind.port())
        .finalize()
        .context(Config)?;

    rocket::custom(config)
        .mount(
            "/",
            routes![
               create_execution_environment,
               create_shard_block,
               get_execution_environment,
               get_execution_environment_state,
               get_shard_block,
               get_shard_state,
            ],
        )
        .manage(handle)
        .launch();

    Ok(())
}

#[tokio::main] // TODO: Check efficiency of tokio::main. Does it create or reuse thread pools?
#[post("/create-execution-environment", data = "<args>")]
async fn create_execution_environment(
   args: Json<simulation_args::CreateExecutionEnvironment>,
   handle: State<Handle>,
) -> DispatchResult<Json<u64>> {
    let args = args.into_inner();

    let ee_index = handle.clone().create_execution_environment(args).await?;

    Ok(Json(ee_index))
}

#[tokio::main]
#[post("/create-shard-block", data = "<args>")]
async fn create_shard_block(
    args: Json<simulation_args::CreateShardBlock>,
    handle: State<Handle>,
) -> DispatchResult<Json<u64>> {
    let args = args.into_inner();
    let shard_block_index = handle.clone().create_shard_block(args).await?;
    Ok(Json(shard_block_index))
}

#[tokio::main]
#[post("/get-execution-environment", data = "<args>")]
async fn get_execution_environment(
   args: Json<simulation_args::GetExecutionEnvironment>,
   handle: State<Handle>,
) -> DispatchResult<Json<simulation_args::ExecutionEnvironment>> {
    let args = args.into_inner();
    let ee = handle.clone().get_execution_environment(args).await?;
    Ok(Json(ee))
}

#[tokio::main]
#[post("/get-execution-environment-state", data = "<args>")]
async fn get_execution_environment_state(
    args: Json<simulation_args::GetExecutionEnvironmentState>,
    handle: State<Handle>,
) -> DispatchResult<Json<simulation_args::CustomSerializedReturnTypes>> {
    let args = args.into_inner();
    let ee_state_root = handle.clone().get_execution_environment_state(args).await?;
    let encodeable_ee_state_root = simulation_args::CustomSerializedReturnTypes::Base64EncodedRoot(ee_state_root);
    Ok(Json(encodeable_ee_state_root))
}

#[tokio::main]
#[post("/get-shard-block", data = "<args>")]
async fn get_shard_block(
    args: Json<simulation_args::GetShardBlock>,
    handle: State<Handle>,
) -> DispatchResult<Json<simulation_args::ShardBlock>> {
    let args = args.into_inner();
    let shard_block = handle.clone().get_shard_block(args).await?;
    Ok(Json(shard_block))
}

#[tokio::main]
#[post("/get-shard-state", data = "<args>")]
async fn get_shard_state(
    args: Json<simulation_args::GetShardState>,
    handle: State<Handle>,
) -> DispatchResult<Json<simulation_args::ShardState>> {
    let args = args.into_inner();
    let shard_state = handle.clone().get_shard_state(args).await?;
    Ok(Json(shard_state))
}
