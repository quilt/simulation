use crate::ethereum::Handle;

use snafu::Snafu;

/// Shorthand for result types returned from the API server.
pub type Result<V, E = Error> = std::result::Result<V, E>;

/// Errors arising from the API server.
#[derive(Debug, Snafu)]
pub enum Error {}

pub fn run(handle: Handle) -> Result<()> {
    unimplemented!()
}
