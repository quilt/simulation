use crate::ethereum::{args, Handle, Result as EthResult};
use crate::Notion;

use rocket::response::status;
use rocket::{config, uri};
use rocket::{get, post, routes, State};

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
        .mount(
            "/",
            routes![
                simulation_state,
                show_execution_environment,
                create_execution_environment
            ],
        )
        .manage(handle)
        .launch();

    Ok(())
}

#[tokio::main] // TODO: Check efficiency of tokio::main. Does it create or reuse thread pools?
#[get("/")]
async fn simulation_state(simulation: State<Handle>) -> EthResult<Json<args::SimulationState>> {
    let state = simulation.clone().simulation_state().await?;
    Ok(Json(state))
}

#[tokio::main]
#[get("/beacon/execution-environments/<index>")]
async fn show_execution_environment(
    index: u32,
    simulation: State<Handle>,
) -> EthResult<Json<args::ExecutionEnvironment>> {
    let arg = args::GetExecutionEnvironment {
        execution_environment_index: index,
    };

    let ee = simulation.clone().execution_environment(arg).await?;
    Ok(Json(ee))
}

#[tokio::main]
#[post("/beacon/execution-environments", data = "<ee>")]
async fn create_execution_environment(
    ee: Json<args::ExecutionEnvironment>,
    simulation: State<Handle>,
) -> EthResult<status::Created<()>> {
    let ee = ee.into_inner();

    let arg = args::CreateExecutionEnvironment {
        execution_environment: ee.clone(),
    };

    let idx = simulation.clone().create_execution_environment(arg).await?;
    let location = uri!(show_execution_environment: idx);

    Ok(status::Created(location.to_string(), None))
}
