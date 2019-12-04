use crate::ethereum::Handle;
use crate::Notion;

use rocket::config;
use rocket::{get, routes, State};

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
        .mount("/", routes![hello])
        .manage(handle)
        .launch();

    Ok(())
}

#[get("/")]
fn hello(_simulation: State<Handle>) -> String {
    // TODO: This is where we can interact with the simulation.

    "Hello, World".into()
}
