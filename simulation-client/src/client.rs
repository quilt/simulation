use crate::{Result, Parse, Reqwest};
use reqwest::Client as HttpClient;
use serde::ser::Serialize;
use snafu::{ResultExt};
use url::Url;

#[derive(Debug)]
pub struct SimulationClient {
    /// IP address and port of simulation-server for sending API requests.
    base_url: Url,
    http_client: HttpClient,
}

impl SimulationClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            http_client: HttpClient::new(),
        }
    }

    // fn send_request(&self, args_struct: S, path: String) -> Result<Response> {
    //
    // }

    pub async fn create_execution_environment(&self, a: simulation_args::CreateExecutionEnvironment) -> Result<u64> {
        let url = self.base_url.join("/create-execution-environment").context(Parse)?;

        let res = self.http_client.post(url)
            .json(&a)
            .send()
            .await.context(Reqwest)?
            .json::<u64>()
            .await.context(Reqwest)?;

        Ok(res)
    }
}
