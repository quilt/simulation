use crate::ethereum::{args, Handle, Result as EthResult};
use crate::Notion;

use rocket::config;
use rocket::{get, routes, State};

use rocket_contrib::json::Json;

use snafu::{ResultExt, Snafu};

/// Shorthand for result types returned from the API server.
pub type Result<V, E = Error> = std::result::Result<V, E>;

/// Errors arising from the API server.
#[derive(Debug, Snafu)]
pub enum Error {
    Config { source: config::ConfigError },
}

pub fn run(notion: &Notion, handle: Handle) -> Result<()> {
    let config = config::Config::build(config::Environment::Development)
        .address(format!("{}", notion.bind.ip()))
        .port(notion.bind.port())
        .finalize()
        .context(Config)?;

    rocket::custom(config)
        .mount("/", routes![simulation_state, show_execution_environment,])
        .manage(handle)
        .launch();

    Ok(())
}

#[tokio::main] // TODO: Check efficiency of tokio::main. Does it create or reuse thread pools?
#[get("/")]
async fn simulation_state(handle: State<Handle>) -> EthResult<Json<args::SimulationState>> {
    let state = handle.clone().simulation_state().await?;
    Ok(Json(state))
}

#[tokio::main]
#[get("/beacon/execution-environments/<index>")]
async fn show_execution_environment(
    index: u32,
    handle: State<Handle>,
) -> EthResult<Json<args::ExecutionEnvironment>> {
    let arg = args::GetExecutionEnvironment {
        execution_environment_index: index,
    };

    let ee = handle.clone().execution_environment(arg).await?;
    Ok(Json(ee))
}
