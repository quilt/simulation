use crate::ethereum::Handle;

use rocket::{get, routes, State};

use snafu::Snafu;

/// Shorthand for result types returned from the API server.
pub type Result<V, E = Error> = std::result::Result<V, E>;

/// Errors arising from the API server.
#[derive(Debug, Snafu)]
pub enum Error {}

pub fn run(handle: Handle) -> Result<()> {
    // TODO: Get some configuration options from command line.

    rocket::ignite()
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
